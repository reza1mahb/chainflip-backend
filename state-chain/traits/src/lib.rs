#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::pallet_prelude::Member;
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, UnfilteredDispatchable, Weight},
	traits::{Imbalance, SignedImbalance},
	Parameter,
};
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

pub mod mocks;

/// and Chainflip was born...some base types
pub trait Chainflip {
	/// An amount for a bid
	type Amount: Member + Parameter + Default + Eq + Ord + Copy + AtLeast32BitUnsigned;
	/// An identity for a validator
	type ValidatorId: Member + Parameter;
}

/// A trait abstracting the functionality of the witnesser
pub trait Witnesser {
	/// The type of accounts that can witness.
	type AccountId;
	/// The call type of the runtime.
	type Call: UnfilteredDispatchable;

	/// Witness an event. The event is represented by a call, which is dispatched when a threshold number of witnesses
	/// have been made.
	///
	/// **IMPORTANT**
	/// The encoded `call` and its arguments are expected to be *unique*. If necessary this should be enforced by adding
	/// a salt or nonce to the function arguments.
	/// **IMPORTANT**
	fn witness(who: Self::AccountId, call: Self::Call) -> DispatchResultWithPostInfo;
}

pub trait EpochInfo {
	/// The id type used for the validators.
	type ValidatorId;
	/// An amount
	type Amount;
	/// The index of an epoch
	type EpochIndex;

	/// The current set of validators
	fn current_validators() -> Vec<Self::ValidatorId>;

	/// Checks if the account is currently a validator.
	fn is_validator(account: &Self::ValidatorId) -> bool;

	/// If we are in auction phase then the proposed set to validate once the auction is
	/// confirmed else an empty vector
	fn next_validators() -> Vec<Self::ValidatorId>;

	/// The amount to be used as bond, this is the minimum stake needed to get into the
	/// candidate validator set
	fn bond() -> Self::Amount;

	/// The current epoch we are in
	fn epoch_index() -> Self::EpochIndex;

	/// Whether or not we are currently in the auction resolution phase of the current Epoch.
	fn is_auction_phase() -> bool;
}

/// The phase of an Auction. At the start we are waiting on bidders, we then run an auction and
/// finally it is completed
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum AuctionPhase<ValidatorId, Amount> {
	/// Waiting for bids, we store the last set of winners and min bid required
	WaitingForBids,
	/// Bids are now taken and validated
	BidsTaken(Vec<Bid<ValidatorId, Amount>>),
	/// We have ran the auction and have a set of validators with minimum active bid.  This waits on confirmation
	/// via the trait `VaultRotation`
	ValidatorsSelected(Vec<ValidatorId>, Amount),
	/// The confirmed set of validators
	ConfirmedValidators(Vec<ValidatorId>, Amount),
}

impl<ValidatorId, Amount: Default> Default for AuctionPhase<ValidatorId, Amount> {
	fn default() -> Self {
		AuctionPhase::WaitingForBids
	}
}

/// A bid represented by a validator and the amount they wish to bid
pub type Bid<ValidatorId, Amount> = (ValidatorId, Amount);
/// A bid that has been classified as out of the validating set
pub type RemainingBid<ValidatorId, Amount> = Bid<ValidatorId, Amount>;

/// A successful auction result
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct AuctionResult<ValidatorId, Amount> {
	pub winners: Vec<ValidatorId>,
	pub minimum_active_bid: Amount,
}

/// A range of min, max for active validator set
pub type ActiveValidatorRange = (u32, u32);

/// An Auction
///
/// An auction is broken down into three phases described by `AuctionPhase`
/// At the start we look for bidders provided by `BidderProvider` from which an auction is ran
/// This results in a set of winners and a minimum bid after the auction.  After each successful
/// call of `process()` the phase will transition else resulting in an error and preventing to move
/// on.  A confirmation is looked to before completing the auction with the `AuctionConfirmation`
/// trait.
pub trait Auction {
	type ValidatorId;
	type Amount;
	type BidderProvider;

	/// Range describing auction set size
	fn active_range() -> ActiveValidatorRange;
	/// Set new auction range, returning on success the old value
	fn set_active_range(range: ActiveValidatorRange) -> Result<ActiveValidatorRange, AuctionError>;
	/// Our last successful auction result
	fn auction_result() -> Option<AuctionResult<Self::ValidatorId, Self::Amount>>;
	/// The current phase we find ourselves in
	fn phase() -> AuctionPhase<Self::ValidatorId, Self::Amount>;
	/// Are we in an auction?
	fn waiting_on_bids() -> bool;
	/// Move our auction process to the next phase returning success with phase completed
	///
	/// At each phase we assess the bidders based on a fixed set of criteria which results
	/// in us arriving at a winning list and a bond set for this auction
	fn process() -> Result<AuctionPhase<Self::ValidatorId, Self::Amount>, AuctionError>;
	/// Abort the process and back the preliminary phase
	fn abort();
}

/// Feedback on a vault rotation
pub trait VaultRotationHandler {
	type ValidatorId;
	/// The vault rotation has been aborted
	fn vault_rotation_aborted();
	/// Penalise bad validators during a vault rotation
	fn penalise(bad_validators: &[Self::ValidatorId]);
}

/// Errors occurring during a rotation
#[derive(RuntimeDebug, Encode, Decode, PartialEq, Clone)]
pub enum RotationError<ValidatorId> {
	/// An invalid request index
	InvalidCeremonyId,
	/// Empty validator set provided
	EmptyValidatorSet,
	/// A set of badly acting validators
	BadValidators(Vec<ValidatorId>),
	/// The keygen response says the newly generated key is the same as the old key
	KeyUnchanged,
	/// Failed to construct a valid chain specific payload for rotation
	FailedToConstructPayload,
	/// The vault rotation is not confirmed
	NotConfirmed,
	/// Failed to make keygen request
	FailedToMakeKeygenRequest,
	/// New public key has not been set by a keygen_response
	NewPublicKeyNotSet,
}

/// Rotating vaults
pub trait VaultRotator {
	type ValidatorId;
	/// Start a vault rotation with the following `candidates`
	fn start_vault_rotation(
		candidates: Vec<Self::ValidatorId>,
	) -> Result<(), RotationError<Self::ValidatorId>>;

	/// In order for the validators to be rotated we are waiting on a confirmation that the vaults
	/// have been rotated.
	fn finalize_rotation() -> Result<(), RotationError<Self::ValidatorId>>;
}

/// An error has occurred during an auction
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub enum AuctionError {
	Empty,
	MinValidatorSize,
	InvalidRange,
	Abort,
	NotConfirmed,
}

/// Handler for Epoch life cycle events.
pub trait EpochTransitionHandler {
	/// The id type used for the validators.
	type ValidatorId;
	type Amount: Copy;
	/// A new epoch has started
	///
	/// The new set of validator `new_validators` are now validating
	fn on_new_epoch(_new_validators: &Vec<Self::ValidatorId>, _new_bond: Self::Amount) {}
}

/// Providing bidders for an auction
pub trait BidderProvider {
	type ValidatorId;
	type Amount;
	/// Provide a list of bidders
	fn get_bidders() -> Vec<Bid<Self::ValidatorId, Self::Amount>>;
}

/// Trait for rotate bond after epoch.
pub trait BondRotation {
	type AccountId;
	type Balance;

	/// Sets the validator bond for all new_validator to the new_bond and
	/// the bond for all old validators to zero.
	fn update_validator_bonds(new_validators: &[Self::AccountId], new_bond: Self::Balance);
}

/// Provide feedback on staking
pub trait StakeHandler {
	type ValidatorId;
	type Amount;
	/// A validator has updated their stake and now has a new total amount
	fn stake_updated(validator_id: &Self::ValidatorId, new_total: Self::Amount);
}

pub trait StakeTransfer {
	type AccountId;
	type Balance;
	type Handler: StakeHandler<ValidatorId = Self::AccountId, Amount = Self::Balance>;

	/// An account's tokens that are free to be staked.
	fn stakeable_balance(account_id: &Self::AccountId) -> Self::Balance;

	/// An account's tokens that are free to be claimed.
	fn claimable_balance(account_id: &Self::AccountId) -> Self::Balance;

	/// Credit an account with stake from off-chain. Returns the total stake in the account.
	fn credit_stake(account_id: &Self::AccountId, amount: Self::Balance) -> Self::Balance;

	/// Reserves funds for a claim, if enough claimable funds are available.
	///
	/// Note this function makes no assumptions about how many claims may be pending simultaneously: if enough funds
	/// are available, it succeeds. Otherwise, it fails.
	fn try_claim(account_id: &Self::AccountId, amount: Self::Balance) -> Result<(), DispatchError>;

	/// Performs any necessary settlement once a claim has been confirmed off-chain.
	fn settle_claim(amount: Self::Balance);

	/// Reverts a pending claim in the case of an expiry or cancellation.
	fn revert_claim(account_id: &Self::AccountId, amount: Self::Balance);
}

/// Trait for managing token issuance.
pub trait Issuance {
	type AccountId;
	type Balance;
	/// An imbalance representing freshly minted, unallocated funds.
	type Surplus: Imbalance<Self::Balance>;

	/// Mint new funds.
	fn mint(amount: Self::Balance) -> Self::Surplus;

	/// Burn funds from somewhere.
	fn burn(amount: Self::Balance) -> <Self::Surplus as Imbalance<Self::Balance>>::Opposite;

	/// Returns the total issuance.
	fn total_issuance() -> Self::Balance;
}

/// Distribute rewards somehow.
pub trait RewardsDistribution {
	type Balance;
	/// An imbalance representing an unallocated surplus of funds.
	type Surplus: Imbalance<Self::Balance> + Into<SignedImbalance<Self::Balance, Self::Surplus>>;

	/// Distribute some rewards.
	fn distribute(rewards: Self::Surplus);

	/// The execution weight of calling the distribution function.
	fn execution_weight() -> Weight;
}

pub trait RewardRollover {
	type AccountId;
	/// Rolls over to another rewards period with a new set of beneficiaries, provided enough funds are available.
	///
	/// 1. Checks that all entitlements can be honoured, ie. there are enough reserves.
	/// 2. Credits all current beneficiaries with any remaining reward entitlements.
	/// 3. If any dust is left over in the reserve, keeps it for the next reward period.
	/// 4. Resets the apportioned rewards counter to zero.
	/// 5. Updates the list of beneficiaries.
	fn rollover(new_beneficiaries: &Vec<Self::AccountId>) -> Result<(), DispatchError>;
}

/// Allow triggering of emissions.
pub trait EmissionsTrigger {
	/// Trigger emissions.
	fn trigger_emissions();
}

/// A nonce
pub type Nonce = u64;

/// A identifier for the chain a nonce is required
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum NonceIdentifier {
	Ethereum = 1,
	Bitcoin = 2,
	Dot = 3,
}

/// Provide a nonce
pub trait NonceProvider {
	/// Provide the next nonce for the chain identified
	fn next_nonce(identifier: NonceIdentifier) -> Nonce;
}

pub trait Online {
	/// The validator id used
	type ValidatorId;
	/// The online status of the validator
	fn is_online(validator_id: &Self::ValidatorId) -> bool;
}

/// A representation of the current network state
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq)]
pub struct NetworkState {
	pub online: u32,
	pub offline: u32,
}

impl NetworkState {
	/// Return the percentage of validators online rounded down
	pub fn percentage_online(&self) -> u32 {
		self.online
			.saturating_mul(100)
			.checked_div(self.online + self.offline)
			.unwrap_or(0)
	}
}

/// To handle those emergency rotations
pub trait EmergencyRotation {
	/// Request an emergency rotation
	fn request_emergency_rotation();
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, Copy)]
pub enum ChainflipAccountState {
	Passive,
	Backup,
	Validator,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
pub struct ChainflipAccountData {
	pub state: ChainflipAccountState,
}

impl Default for ChainflipAccountData {
	fn default() -> Self {
		ChainflipAccountData {
			state: ChainflipAccountState::Passive,
		}
	}
}

pub trait ChainflipAccount {
	type AccountId;

	fn get(account_id: &Self::AccountId) -> ChainflipAccountData;
	fn update_state(account_id: &Self::AccountId, state: ChainflipAccountState);
}

pub struct ChainflipAccountStore<T>(PhantomData<T>);

impl<T: frame_system::Config<AccountData = ChainflipAccountData>> ChainflipAccount
	for ChainflipAccountStore<T>
{
	type AccountId = T::AccountId;

	fn get(account_id: &Self::AccountId) -> ChainflipAccountData {
		frame_system::Pallet::<T>::get(account_id)
	}

	fn update_state(account_id: &Self::AccountId, state: ChainflipAccountState) {
		frame_system::Pallet::<T>::mutate(account_id, |account_data| {
			(*account_data).state = state;
		})
		.expect("mutating account state")
	}
}

/// Slashing a validator
pub trait Slashing {
	/// An identifier for our validator
	type AccountId;
	/// Block number
	type BlockNumber;
	/// Function which implements the slashing logic
	fn slash(validator_id: &Self::AccountId, blocks_offline: Self::BlockNumber) -> Weight;
}
