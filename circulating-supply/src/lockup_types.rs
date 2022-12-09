#![allow(
    clippy::assign_op_pattern,
    clippy::manual_range_contains,
    clippy::ptr_offset_with_cast
)]
// Copied from lockup contract code
// https://github.com/near/core-contracts/blob/master/lockup/src/types.rs
// https://github.com/near/core-contracts/blob/master/lockup/src/lib.rs

use uint::construct_uint;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Raw type for duration in nanoseconds
pub type Duration = u64;
/// Raw type for timestamp in nanoseconds
pub type Timestamp = u64;

/// Timestamp in nanosecond wrapped into a struct for JSON serialization as a string.
pub type WrappedTimestamp = U64;
/// Balance wrapped into a struct for JSON serialization as a string.
pub type WrappedBalance = U128;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockupContract {
    /// The account ID of the owner.
    pub owner_account_id: AccountId,

    /// Information about lockup schedule and the amount.
    pub lockup_information: LockupInformation,

    /// Information about vesting including schedule or termination status.
    pub vesting_information: VestingInformation,

    /// Account ID of the staking pool whitelist contract.
    pub staking_pool_whitelist_account_id: AccountId,

    /// Information about staking and delegation.
    /// `Some` means the staking information is available and the staking pool contract is selected.
    /// `None` means there is no staking pool selected.
    pub staking_information: Option<StakingInformation>,

    /// The account ID that the NEAR Foundation, that has the ability to terminate vesting.
    pub foundation_account_id: Option<AccountId>,
}

/// Contains information about token lockups.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct LockupInformation {
    /// The amount in yocto-NEAR tokens locked for this account.
    pub lockup_amount: Balance,
    /// The amount of tokens that were withdrawn by NEAR foundation due to early termination
    /// of vesting.
    /// This amount has to be accounted separately from the lockup_amount to make sure
    /// linear release is not being affected.
    pub termination_withdrawn_tokens: Balance,
    /// [deprecated] - the duration in nanoseconds of the lockup period from
    /// the moment the transfers are enabled. During this period tokens are locked and
    /// the release doesn't start. Instead of this, use `lockup_timestamp` and `release_duration`
    pub lockup_duration: Duration,
    /// If present, it is the duration when the full lockup amount will be available. The tokens
    /// are linearly released from the moment tokens are unlocked, defined by:
    /// `max(transfers_timestamp + lockup_duration, lockup_timestamp)`.
    /// If not present, the tokens are not locked (though, vesting logic could be used).
    pub release_duration: Option<Duration>,
    /// The optional absolute lockup timestamp in nanoseconds which locks the tokens until this
    /// timestamp passes. Until this moment the tokens are locked and the release doesn't start.
    /// If not present, `transfers_timestamp` will be used.
    pub lockup_timestamp: Option<Timestamp>,
    /// The information about the transfers. Either transfers are already enabled, then it contains
    /// the timestamp when they were enabled. Or the transfers are currently disabled and
    /// it contains the account ID of the transfer poll contract.
    pub transfers_information: TransfersInformation,
}

/// Contains information about the transfers. Whether transfers are enabled or disabled.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TransfersInformation {
    /// The timestamp when the transfers were enabled.
    TransfersEnabled {
        transfers_timestamp: WrappedTimestamp,
    },
    /// The account ID of the transfers poll contract, to check if the transfers are enabled.
    /// The lockup period can start only after the transfer voted to be enabled.
    /// At the launch of the network transfers are disabled for all lockup contracts, once transfers
    /// are enabled, they can't be disabled and don't need to be checked again.
    TransfersDisabled { transfer_poll_account_id: AccountId },
}

/// Describes the status of transactions with the staking pool contract or terminated unvesting
/// amount withdrawal.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum TransactionStatus {
    /// There are no transactions in progress.
    Idle,
    /// There is a transaction in progress.
    Busy,
}

/// Contains information about current stake and delegation.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakingInformation {
    /// The Account ID of the staking pool contract.
    pub staking_pool_account_id: AccountId,

    /// Contains status whether there is a transaction in progress.
    pub status: TransactionStatus,

    /// The amount of tokens that were deposited from this account to the staking pool.
    /// NOTE: The unstaked amount on the staking pool might be higher due to staking rewards.
    pub deposit_amount: WrappedBalance,
}

/// Contains information about vesting schedule.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VestingSchedule {
    /// The timestamp in nanosecond when the vesting starts. E.g. the start date of employment.
    pub start_timestamp: WrappedTimestamp,
    /// The timestamp in nanosecond when the first part of lockup tokens becomes vested.
    /// The remaining tokens will vest continuously until they are fully vested.
    /// Example: a 1 year of employment at which moment the 1/4 of tokens become vested.
    pub cliff_timestamp: WrappedTimestamp,
    /// The timestamp in nanosecond when the vesting ends.
    pub end_timestamp: WrappedTimestamp,
}

/// Initialization argument type to define the vesting schedule
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum VestingScheduleOrHash {
    /// [deprecated] After transfers are enabled, only public schedule is used.
    /// The vesting schedule is private and this is a hash of (vesting_schedule, salt).
    /// In JSON, the hash has to be encoded with base64 to a string.
    VestingHash(Base64VecU8),
    /// The vesting schedule (public)
    VestingSchedule(VestingSchedule),
}

/// Contains information about vesting that contains vesting schedule and termination information.
#[derive(Serialize, BorshDeserialize, BorshSerialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum VestingInformation {
    None,
    /// [deprecated] After transfers are enabled, only public schedule is used.
    /// Vesting schedule is hashed for privacy and only will be revealed if the NEAR foundation
    /// has to terminate vesting.
    /// The contract assume the vesting schedule doesn't affect lockup release and duration, because
    /// the vesting started before transfers were enabled and the duration is shorter or the same.
    VestingHash(Base64VecU8),
    /// Explicit vesting schedule.
    VestingSchedule(VestingSchedule),
    /// The information about the early termination of the vesting schedule.
    /// It means the termination of the vesting is currently in progress.
    /// Once the unvested amount is transferred out, `VestingInformation` is removed.
    Terminating(TerminationInformation),
}

/// Describes the status of transactions with the staking pool contract or terminated unvesting
/// amount withdrawal.
#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Copy, Clone, Debug,
)]
#[serde(crate = "near_sdk::serde")]
pub enum TerminationStatus {
    /// Initial stage of the termination in case there are deficit on the account.
    VestingTerminatedWithDeficit,
    /// A transaction to unstake everything is in progress.
    UnstakingInProgress,
    /// The transaction to unstake everything from the staking pool has completed.
    EverythingUnstaked,
    /// A transaction to withdraw everything from the staking pool is in progress.
    WithdrawingFromStakingPoolInProgress,
    /// Everything is withdrawn from the staking pool. Ready to withdraw out of the account.
    ReadyToWithdraw,
    /// A transaction to withdraw tokens from the account is in progress.
    WithdrawingFromAccountInProgress,
}

/// Contains information about early termination of the vesting schedule.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TerminationInformation {
    /// The amount of tokens that are unvested and has to be transferred back to NEAR Foundation.
    /// These tokens are effectively locked and can't be transferred out and can't be restaked.
    pub unvested_amount: WrappedBalance,

    /// The status of the withdrawal. When the unvested amount is in progress of withdrawal the
    /// status will be marked as busy, to avoid withdrawing the funds twice.
    pub status: TerminationStatus,
}

/// Contains a vesting schedule with a salt.
#[derive(BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VestingScheduleWithSalt {
    /// The vesting schedule
    pub vesting_schedule: VestingSchedule,
    /// Salt to make the hash unique
    pub salt: Base64VecU8,
}
