use std::time::Duration;

use actix::Addr;
use anyhow::Context;

use near_client::{Query, ViewClientActor};
use near_indexer::near_primitives;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::json_types::{U128, U64};

use super::lockup_types::{
    LockupContract, TransfersInformation, VestingInformation, VestingSchedule, WrappedBalance, U256,
};

// The timestamp (nanos) when transfers were enabled in the Mainnet after community voting
// Tuesday, 13 October 2020 18:38:58.293
pub(super) const TRANSFERS_ENABLED: Duration = Duration::from_nanos(1602614338293769340);

pub(super) async fn get_lockup_contract_state(
    view_client: &Addr<ViewClientActor>,
    account_id: &near_primitives::types::AccountId,
    block_height: &near_primitives::types::BlockHeight,
) -> anyhow::Result<LockupContract> {
    let block_reference = near_primitives::types::BlockReference::BlockId(
        near_primitives::types::BlockId::Height(*block_height),
    );
    let request = near_primitives::views::QueryRequest::ViewState {
        account_id: account_id.clone(),
        prefix: vec![].into(),
    };
    let query = Query::new(block_reference, request);

    let state_response = view_client
        .send(query)
        .await
        .with_context(|| {
            format!(
                "Failed to deliver ViewState for lockup contract {}, block_height {}",
                account_id, block_height
            )
        })?
        .with_context(|| {
            format!(
                "Invalid ViewState query for lockup contract {}, block_height {}",
                account_id, block_height
            )
        })?;

    let view_state_result = match state_response.kind {
        near_primitives::views::QueryResponseKind::ViewState(x) => x,
        _ => {
            anyhow::bail!(
                "Failed to extract ViewState response for lockup contract {}, block_height {}",
                account_id,
                block_height
            )
        }
    };
    let view_state = view_state_result.values.get(0).with_context(|| {
        format!(
            "Failed to find encoded lockup contract for {}, block_height {}",
            account_id, block_height
        )
    })?;

    let mut state = LockupContract::try_from_slice(&view_state.value)
        .with_context(|| format!("Failed to construct LockupContract for {}", account_id))?;

    // If owner of the lockup account didn't call the
    // `check_transfers_vote` contract method we won't be able to
    // get proper information based on timestamp, that's why we inject
    // the `transfer_timestamp` which is phase2 timestamp
    state.lockup_information.transfers_information = TransfersInformation::TransfersEnabled {
        transfers_timestamp: U64(TRANSFERS_ENABLED.as_nanos() as u64),
    };
    Ok(state)
}

// The lockup contract implementation had a bug that affected lockup start date.
// https://github.com/near/core-contracts/pull/136
// For each contract, we should choose the logic based on the binary version of the contract
pub(super) fn is_bug_inside_contract(
    code_hash: &near_primitives::hash::CryptoHash,
    account_id: &near_primitives::types::AccountId,
) -> anyhow::Result<bool> {
    match &*code_hash.to_string() {
        // The first implementation, with the bug
        "3kVY9qcVRoW3B5498SMX6R3rtSLiCdmBzKs7zcnzDJ7Q" => Ok(true),
        // We have 6 lockups created at 6th of April 2021, assume it's buggy
        "DiC9bKCqUHqoYqUXovAnqugiuntHWnM3cAc7KrgaHTu" => Ok(true),
        // Another 5 lockups created in May/June 2021, assume they are OK
        "Cw7bnyp4B6ypwvgZuMmJtY6rHsxP2D4PC8deqeJ3HP7D" => Ok(false),
        // The most fresh one
        "4Pfw2RU6e35dUsHQQoFYfwX8KFFvSRNwMSNLXuSFHXrC" => Ok(false),
        other => anyhow::bail!(
            "Unable to recognise the version of contract {}, code hash {}",
            account_id,
            other
        ),
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
                    if let Some(release_duration) = self.lockup_information.release_duration {
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
