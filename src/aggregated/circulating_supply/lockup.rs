use actix::Addr;

use near_client::{Query, ViewClientActor};
use near_indexer::near_primitives::hash::CryptoHash;
use near_indexer::near_primitives::types::{BlockId, BlockReference};
use near_indexer::near_primitives::views::{QueryRequest, QueryResponseKind};
use near_sdk::borsh::BorshDeserialize;
use near_sdk::json_types::{U128, U64};

use crate::aggregated::circulating_supply::lockup_types::{
    LockupContract, TransfersInformation, VestingInformation, VestingSchedule, WrappedBalance, U256,
};

// The timestamp (nanos) when transfers were enabled in the Mainnet after community voting
// Tuesday, 13 October 2020 18:38:58.293
pub const TRANSFERS_ENABLED: u64 = 1602614338293769340;

pub(crate) async fn get_account_state(
    view_client: &Addr<ViewClientActor>,
    account_id: &str,
    block_height: u64,
) -> Result<LockupContract, String> {
    let block_reference = BlockReference::BlockId(BlockId::Height(block_height));
    let request = QueryRequest::ViewState {
        account_id: account_id
            .parse()
            .map_err(|_| "Failed to parse `account_id`")?,
        prefix: vec![].into(),
    };
    let query = Query::new(block_reference, request);

    let state_response = view_client
        .send(query)
        .await
        .map_err(|err| {
            format!(
                "Error while delivering ViewState for account {}, block_height {}: {}",
                account_id, block_height, err
            )
        })?
        .map_err(|_| {
            format!(
                "Invalid ViewState query for account {}, block_height {}",
                account_id, block_height
            )
        })?;

    let view_state_result = match state_response.kind {
        QueryResponseKind::ViewState(x) => x,
        _ => {
            return Err(format!(
                "ViewState result expected for account {}, block_height {}",
                account_id, block_height
            ))
        }
    };
    let view_state = view_state_result.values.get(0).ok_or(format!(
        "Encoded contract expected for account {}, block_height {}",
        account_id, block_height
    ))?;

    let mut state = LockupContract::try_from_slice(
        base64::decode(&view_state.value)
            .map_err(|err| {
                format!(
                    "Failed to decode `view_state` for the account {}: {}",
                    account_id, err
                )
            })?
            .as_slice(),
    )
    .map_err(|err| {
        format!(
            "Failed to construct LockupContract for the account {}: {}",
            account_id, err
        )
    })?;

    // If owner of the lockup account didn't call the
    // `check_transfers_vote` contract method we won't be able to
    // get proper information based on timestamp, that's why we inject
    // the `transfer_timestamp` which is phase2 timestamp
    state.lockup_information.transfers_information = TransfersInformation::TransfersEnabled {
        transfers_timestamp: U64(TRANSFERS_ENABLED),
    };
    Ok(state)
}

// The lockup contract implementation had a bug that affected lockup start date.
// For each contract, we should choose the logic based on the binary version of the contract
pub(crate) fn is_bug_inside_contract(code_hash: &CryptoHash, acc_id: &str) -> Result<bool, String> {
    match &*code_hash.to_string() {
        // The first realization, with the bug
        "3kVY9qcVRoW3B5498SMX6R3rtSLiCdmBzKs7zcnzDJ7Q" => Ok(true),
        // We have 6 lockups created at 6th of April 2021, assume it's buggy
        "DiC9bKCqUHqoYqUXovAnqugiuntHWnM3cAc7KrgaHTu" => Ok(true),
        // Another 5 lockups created in May/June 2021, assume they are OK
        "Cw7bnyp4B6ypwvgZuMmJtY6rHsxP2D4PC8deqeJ3HP7D" => Ok(false),
        // The most fresh one
        "4Pfw2RU6e35dUsHQQoFYfwX8KFFvSRNwMSNLXuSFHXrC" => Ok(false),
        other => Err(format!(
            "Unable to recognise the version of contract {}, code hash {}",
            acc_id, other
        )),
    }
}

// This is almost a copy of https://github.com/near/core-contracts/blob/master/lockup/src/getters.rs#L64
impl LockupContract {
    /// Returns the amount of tokens that are locked in the account due to lockup or vesting.
    pub fn get_locked_amount(&self, timestamp: u64, has_bug: bool) -> WrappedBalance {
        let lockup_amount = self.lockup_information.lockup_amount;
        if let TransfersInformation::TransfersEnabled {
            transfers_timestamp,
        } = &self.lockup_information.transfers_information
        {
            let lockup_timestamp = std::cmp::max(
                transfers_timestamp
                    .0
                    .saturating_add(self.lockup_information.lockup_duration),
                self.lockup_information.lockup_timestamp.unwrap_or(0),
            );
            let block_timestamp = timestamp;
            if lockup_timestamp <= block_timestamp {
                let unreleased_amount =
                    if let &Some(release_duration) = &self.lockup_information.release_duration {
                        let start_lockup = if has_bug {
                            transfers_timestamp.0
                        } else {
                            lockup_timestamp
                        };
                        let end_timestamp = start_lockup.saturating_add(release_duration);
                        if block_timestamp >= end_timestamp {
                            // Everything is released
                            0
                        } else {
                            let time_left = U256::from(end_timestamp - block_timestamp);
                            let unreleased_amount = U256::from(lockup_amount) * time_left
                                / U256::from(release_duration);
                            // The unreleased amount can't be larger than lockup_amount because the
                            // time_left is smaller than total_time.
                            unreleased_amount.as_u128()
                        }
                    } else {
                        0
                    };

                let unvested_amount = match &self.vesting_information {
                    VestingInformation::VestingSchedule(vs) => {
                        self.get_unvested_amount(vs.clone(), block_timestamp)
                    }
                    VestingInformation::Terminating(terminating) => terminating.unvested_amount,
                    // Vesting is private, so we can assume the vesting started before lockup date.
                    _ => U128(0),
                };
                return std::cmp::max(
                    unreleased_amount
                        .saturating_sub(self.lockup_information.termination_withdrawn_tokens),
                    unvested_amount.0,
                )
                .into();
            }
        }
        // The entire balance is still locked before the lockup timestamp.
        (lockup_amount - self.lockup_information.termination_withdrawn_tokens).into()
    }

    /// Returns the amount of tokens that are locked in this account due to vesting schedule.
    /// Takes raw vesting schedule, in case the internal vesting schedule is private.
    pub fn get_unvested_amount(
        &self,
        vesting_schedule: VestingSchedule,
        block_timestamp: u64,
    ) -> WrappedBalance {
        let lockup_amount = self.lockup_information.lockup_amount;
        match &self.vesting_information {
            VestingInformation::Terminating(termination_information) => {
                termination_information.unvested_amount
            }
            VestingInformation::None => U128::from(0),
            _ => {
                if block_timestamp < vesting_schedule.cliff_timestamp.0 {
                    // Before the cliff, nothing is vested
                    lockup_amount.into()
                } else if block_timestamp >= vesting_schedule.end_timestamp.0 {
                    // After the end, everything is vested
                    0.into()
                } else {
                    // cannot overflow since block_timestamp < vesting_schedule.end_timestamp
                    let time_left = U256::from(vesting_schedule.end_timestamp.0 - block_timestamp);
                    // The total time is positive. Checked at the contract initialization.
                    let total_time = U256::from(
                        vesting_schedule.end_timestamp.0 - vesting_schedule.start_timestamp.0,
                    );
                    let unvested_amount = U256::from(lockup_amount) * time_left / total_time;
                    // The unvested amount can't be larger than lockup_amount because the
                    // time_left is smaller than total_time.
                    unvested_amount.as_u128().into()
                }
            }
        }
    }
}
