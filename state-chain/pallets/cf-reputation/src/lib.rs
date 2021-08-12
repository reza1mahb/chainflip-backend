#![cfg_attr(not(feature = "std"), no_std)]

//! # ChainFlip Reputation Module
//!
//! A module to manage the reputation of our validators for the ChainFlip State Chain
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Module`]
//!
//! ## Overview
//! The module contains functionality to measure the liveness of our validators.  This is measured
//! with a *heartbeat* which should be submitted via the extrinsic `heartbeat()` within the time
//! period set by the *heartbeat interval*.  By continuing to submit heartbeats the validator will
//! earn *online credits*.  These *online credits* are exchanged for *reputation points*
//! when they have been *online* for a specified period.  *Reputation points* buffer the validator
//! from being slashed when they go offline for a period of time.
//!
//! Penalties in terms of reputation points are incurred when any one of the *offline conditions* are
//! met.  Falling into negative reputation leads to the eventual slashing of FLIP.  As soon as reputation
//! is positive slashing stops.
//!
//! ## Terminology
//! - **Validator:** A node in our network that is producing blocks.
//! - **Heartbeat:** A term used to measure the liveness of a validator.
//! - **Heartbeat interval:** Number of blocks we would expect to receive a heartbeat from a validator.
//! - **Online:** A node that is online has successfully submitted a heartbeat during the current
//!   heartbeat interval.
//! - **Offline:** A node that is considered offline when they have *not* submitted a heartbeat during
//!   the last heartbeat interval or has met one of the other *offline conditions*.
//!  -**Online credits:** A credit accrued by being continously online which inturn is used to earn
//!   *reputation points*.  Failing to stay *online* results in losing all of their *online credits*.
//! - **Reputation points:** A point system which allows validators to earn reputation by being *online*.
//!   They lose reputation points by being *offline*.
//! - **Offline conditions:** One of the following conditions: missed heartbeat, failed to broadcast
//!   an output or failed to participate in a signing ceremony.  Each condition has its associated
//!   penalty in reputation points
//! - **Slashing:** The process of debiting FLIP tokens from a validator.  Slashing only occurs in this
//!   pallet when a validator's reputation points fall below zero *and* they have met one of the
//!   *offline conditions*
//!

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_support::sp_std::convert::TryInto;
pub use pallet::*;
use pallet_cf_validator::EpochTransitionHandler;
use sp_runtime::traits::Saturating;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;

/// Slashing a validator
pub trait Slashing {
	/// An identifier for our validator
	type ValidatorId;
	/// Block number
	type BlockNumber;
	/// Slash this validator by said amount of blocks
	fn slash(validator_id: &Self::ValidatorId, blocks: &Self::BlockNumber) -> Weight;
}

/// Conditions as judged as offline
pub enum OfflineCondition {
	/// A broadcast of an output has failed
	BroadcastOutputFailed(ReputationPoints),
	/// There was a failure in participation during a signing
	ParticipateSigningFailed(ReputationPoints),
}

/// Error on reporting an offline condition
#[derive(Debug, PartialEq)]
pub enum ReportError {
	// Validator doesn't exist
	UnknownValidator,
}

/// Offline conditions are reported on
pub trait OfflineConditions {
	type ValidatorId;
	/// Report the condition for validator
	/// Returns `Ok(Weight)` else an error if the validator isn't valid
	fn report(
		condition: OfflineCondition,
		validator_id: &Self::ValidatorId,
	) -> Result<Weight, ReportError>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use cf_traits::EpochInfo;
	use frame_system::pallet_prelude::*;
	use sp_std::ops::Neg;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Reputation points type as signed integer
	pub type ReputationPoints = i32;
	/// The credits one earns being online, equivalent to a blocktime online
	pub type OnlineCreditsFor<T> = <T as frame_system::Config>::BlockNumber;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// A stable ID for a validator.
		type ValidatorId: Member + Parameter + From<<Self as frame_system::Config>::AccountId>;

		// An amount of a bid
		type Amount: Copy;

		/// The number of blocks for the time frame we would test liveliness within
		#[pallet::constant]
		type HeartbeatBlockInterval: Get<<Self as frame_system::Config>::BlockNumber>;

		/// The number of reputation points we lose for every x blocks offline
		#[pallet::constant]
		type ReputationPointPenalty: Get<(u32, u32)>;

		/// The floor and ceiling values for a reputation score
		#[pallet::constant]
		type ReputationPointFloorAndCeiling: Get<(ReputationPoints, ReputationPoints)>;

		/// When we have to, we slash
		type Slasher: Slashing<
			ValidatorId = Self::ValidatorId,
			BlockNumber = <Self as frame_system::Config>::BlockNumber,
		>;

		// Information about the current epoch.
		type EpochInfo: EpochInfo<ValidatorId = Self::ValidatorId, Amount = Self::Amount>;
	}

	/// Pallet implements [`Hooks`] trait
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// On initializing each block we check liveness every heartbeat interval
		///
		fn on_initialize(current_block: BlockNumberFor<T>) -> Weight {
			if current_block % T::HeartbeatBlockInterval::get() == Zero::zero() {
				return Self::check_liveness(current_block);
			}

			Zero::zero()
		}
	}

	/// The ratio at which one accrues Reputation points in exchange for online credits
	///
	#[pallet::storage]
	#[pallet::getter(fn accrual_ratio)]
	pub(super) type AccrualRatio<T: Config> =
		StorageValue<_, (ReputationPoints, OnlineCreditsFor<T>), ValueQuery>;

	/// Those that we are awaiting heartbeats
	///
	#[pallet::storage]
	pub(super) type AwaitingHeartbeats<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ValidatorId, bool, OptionQuery>;

	/// A map tracking our validators.  We record the number of blocks they have been alive
	/// according to the heartbeats submitted.  We are assuming that during a `HeartbeatInterval`
	/// if a `heartbeat()` transaction is submitted that they are alive during the entire
	/// `HeartbeatInterval` of blocks.
	///
	#[pallet::storage]
	#[pallet::getter(fn reputation)]
	pub type Reputation<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ValidatorId,
		(OnlineCreditsFor<T>, ReputationPoints),
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Broadcast of an output has failed for validator
		BroadcastOutputFailed(T::ValidatorId, ReputationPoints),
		/// Validator has failed to participate in a signing ceremony
		ParticipateSigningFailed(T::ValidatorId, ReputationPoints),
		/// The accrual rate for our reputation poins has been updated \[points, online credits\]
		AccrualRateUpdated(ReputationPoints, OnlineCreditsFor<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A heartbeat has already been submitted for this validator
		AlreadySubmittedHeartbeat,
		/// An invalid amount of reputation points set for the accrual ratio
		InvalidAccrualReputationPoints,
		/// An invalid amount of online credits for the accrual ratio
		InvalidAccrualOnlineCredits,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// A heartbeat that is used to measure the liveness of a validator
		/// Every interval we have a set of validators we expect a heartbeat from with which we
		/// mark off when we have received a heartbeat.  In doing so the validator is credited
		/// the blocks for this heartbeat interval.  Once the block credits have surpassed the accrual
		/// block number they will earn reputation points based on the accrual ratio.
		///
		#[pallet::weight(10_000)]
		pub(super) fn heartbeat(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// for the validator
			let validator_id: T::ValidatorId = ensure_signed(origin)?.into();
			// Ensure we haven't had a heartbeat for this interval yet for this validator
			ensure!(
				AwaitingHeartbeats::<T>::get(&validator_id).unwrap_or(false),
				Error::<T>::AlreadySubmittedHeartbeat
			);
			// Update this validator from the hot list
			AwaitingHeartbeats::<T>::mutate(&validator_id, |awaiting| *awaiting = Some(false));
			// Check if this validator has reputation
			if !Reputation::<T>::contains_key(&validator_id) {
				// Credit this validator with the blocks for this interval and set 0 reputation points
				Reputation::<T>::insert(validator_id, (Self::online_credit_reward(), 0));
			} else {
				// Update reputation points for this validator
				Reputation::<T>::mutate(validator_id, |(online_credits, points)| {
					// Accrue some online credits of `HeartbeatInterval` size
					*online_credits = *online_credits + Self::online_credit_reward();
					let (rewarded_points, credits) = AccrualRatio::<T>::get();
					// If we have hit a number of credits to earn reputation points
					if *online_credits >= credits {
						// Swap these credits for reputation
						*online_credits = *online_credits - credits;
						// Update reputation
						*points = *points + rewarded_points;
					}
				});
			}

			Ok(().into())
		}

		/// The accrual ratio can be updated and would come into play in the current heartbeat interval
		/// This is only available to sudo
		///
		#[pallet::weight(10_000)]
		pub(super) fn update_accrual_ratio(
			origin: OriginFor<T>,
			points: ReputationPoints,
			online_credits: OnlineCreditsFor<T>,
		) -> DispatchResultWithPostInfo {
			// Ensure we are root when setting this
			let _ = ensure_root(origin)?;
			// Some very basic validation here.  Should be improved in subsequent PR based on
			// further definition of limits
			ensure!(
				points > Zero::zero(),
				Error::<T>::InvalidAccrualReputationPoints
			);
			ensure!(
				online_credits > Zero::zero(),
				Error::<T>::InvalidAccrualOnlineCredits
			);
			// Online credits are equivalent to block time and hence should be less than our
			// heartbeat interval
			ensure!(
				online_credits > T::HeartbeatBlockInterval::get(),
				Error::<T>::InvalidAccrualOnlineCredits
			);

			AccrualRatio::<T>::set((points, online_credits));
			Self::deposit_event(Event::AccrualRateUpdated(points, online_credits));

			Ok(().into())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub accrual_ratio: (ReputationPoints, OnlineCreditsFor<T>),
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				accrual_ratio: (Zero::zero(), Zero::zero()),
			}
		}
	}

	/// On genesis we are initializing the accrual ratio confirming that it is greater than the
	/// heartbeat interval.  We also expect a set of validators to expect heartbeats from.
	///
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			assert!(
				self.accrual_ratio.1 > T::HeartbeatBlockInterval::get(),
				"Heartbeat interval needs to be less than block duration reward"
			);
			AccrualRatio::<T>::set(self.accrual_ratio);
			// A list of those we expect to be online, which are our set of validators
			for validator_id in T::EpochInfo::current_validators().iter() {
				AwaitingHeartbeats::<T>::insert(validator_id, true);
			}
		}
	}

	/// Implementation of the `EpochTransitionHandler` trait with which we populate are
	/// expected list of validators.
	///
	impl<T: Config> EpochTransitionHandler for Pallet<T> {
		type ValidatorId = T::ValidatorId;
		type Amount = T::Amount;

		fn on_new_epoch(new_validators: &Vec<Self::ValidatorId>, _new_bond: Self::Amount) {
			// Clear our expectations
			AwaitingHeartbeats::<T>::remove_all();
			// Set the new list of validators we expect a heartbeat from
			for validator_id in new_validators.iter() {
				AwaitingHeartbeats::<T>::insert(validator_id, true);
			}
		}
	}

	/// Implementation of `OfflineConditions` reporting on `OfflineCondition` with specified number
	/// of reputation points
	impl<T: Config> OfflineConditions for Pallet<T> {
		type ValidatorId = T::ValidatorId;

		fn report(
			condition: OfflineCondition,
			validator_id: &Self::ValidatorId,
		) -> Result<Weight, ReportError> {
			// Confirm validator is present
			ensure!(
				Reputation::<T>::contains_key(validator_id),
				ReportError::UnknownValidator
			);

			// Handle offline conditions
			match condition {
				OfflineCondition::BroadcastOutputFailed(penalty) => {
					// Broadcast this penalty
					Self::deposit_event(Event::BroadcastOutputFailed(
						(*validator_id).clone(),
						penalty,
					));
					// Update reputation points
					Ok(Self::update_reputation(validator_id, penalty.neg()))
				}
				OfflineCondition::ParticipateSigningFailed(penalty) => {
					// Broadcast this penalty
					Self::deposit_event(Event::ParticipateSigningFailed(
						(*validator_id).clone(),
						penalty,
					));
					// Update reputation points
					Ok(Self::update_reputation(validator_id, penalty.neg()))
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Return number of online credits for reward
		///
		fn online_credit_reward() -> OnlineCreditsFor<T> {
			// Equivalent to the number of blocks used for the heartbeat
			T::HeartbeatBlockInterval::get()
		}

		/// Update reputation for validator.  Points are clamped to `ReputationPointFloorAndCeiling`
		///
		fn update_reputation(validator_id: &T::ValidatorId, points: ReputationPoints) -> Weight {
			Reputation::<T>::mutate(validator_id, |(_, current_points)| {
				*current_points = *current_points + points;
				let (mut floor, mut ceiling) = T::ReputationPointFloorAndCeiling::get();
				*current_points = *current_points.clamp(&mut floor, &mut ceiling);

				T::DbWeight::get().reads_writes(1, 1)
			})
		}

		/// Calculate the penalty for being offline for an amount of blocks based on`ReputationPointPenalty`
		///
		fn calculate_offline_penalty(
			current_block: T::BlockNumber,
			last_block: T::BlockNumber,
		) -> Result<ReputationPoints, &'static str> {
			ensure!(current_block > last_block, "invalid block");
			// What points for what blocks we penalise by
			let (penalty_points, penalty_blocks) = T::ReputationPointPenalty::get();
			// The blocks we have missed
			let dead_blocks: u32 = current_block
				.saturating_sub(last_block)
				.try_into()
				.map_err(|_| "dead blocks")?;
			// The points calculated
			let penalty: ReputationPoints = (penalty_points.saturating_mul(dead_blocks)
				/ penalty_blocks)
				.try_into()
				.map_err(|_| "penalty")?;
			// The points to be penalised
			Ok(penalty.neg())
		}

		/// Check liveness of our expected list of validators at the current block.
		/// For those that we are still *awaiting* on will be penalised reputation points and any online
		/// credits earned will be set to zero.  In other words we expect continued liveness before we
		/// earn points.
		/// Once the reputation points fall below zero slashing comes into play and is delegated to the
		/// `Slashing` trait.
		fn check_liveness(current_block: T::BlockNumber) -> Weight {
			let mut weight = 0;
			// Let's run through those that haven't come back to us and those that have
			AwaitingHeartbeats::<T>::translate(|validator_id, awaiting| {
				// Still waiting on these, penalise and those that are in reputation debt will be
				// slashed
				if awaiting
					&& Reputation::<T>::mutate(
						&validator_id,
						|(online_credits, reputation_points)| {
							if T::ReputationPointFloorAndCeiling::get().0 < *reputation_points {
								// Update reputation points
								*reputation_points = *reputation_points
									+ Self::calculate_offline_penalty(
										current_block,
										current_block - T::HeartbeatBlockInterval::get(),
									)
									.expect("calculating offline penalty shouldn't fail");
								// Reset the credits earned as being online consecutively
								*online_credits = Zero::zero();
							}
							weight += T::DbWeight::get().reads_writes(1, 1);

							*reputation_points
						},
					) < Zero::zero() || Reputation::<T>::get(&validator_id).1 < Zero::zero()
				{
					// At this point we slash the validator by the amount of blocks offline
					weight += T::Slasher::slash(&validator_id, &T::HeartbeatBlockInterval::get());
				}

				weight += T::DbWeight::get().reads(1);
				Some(true)
			});

			weight
		}
	}
}

/// An implementation of `Slashing` which kindly doesn't slash
impl<T: Config> Slashing for Pallet<T> {
	type ValidatorId = T::ValidatorId;
	type BlockNumber = u64;

	fn slash(_validator_id: &Self::ValidatorId, _blocks: &Self::BlockNumber) -> Weight {
		0
	}
}
