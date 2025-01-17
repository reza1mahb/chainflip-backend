#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../../cf-doc-head.md")]

use cf_chains::{address::AddressConverter, AnyChain, ForeignChainAddress};
use cf_primitives::{Asset, AssetAmount, ForeignChain};
use cf_traits::{
	impl_pallet_safe_mode, liquidity::LpBalanceApi, AccountRoleRegistry, Chainflip, DepositApi,
	EgressApi, PoolApi,
};

use frame_support::{pallet_prelude::*, sp_runtime::DispatchResult};
use frame_system::pallet_prelude::*;
pub use pallet::*;

mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod migrations;
pub mod weights;
pub use weights::WeightInfo;

pub const PALLET_VERSION: StorageVersion = StorageVersion::new(1);

impl_pallet_safe_mode!(PalletSafeMode; deposit_enabled, withdrawal_enabled);

#[frame_support::pallet]
pub mod pallet {
	use cf_chains::{address::EncodedAddress, Chain};
	use cf_primitives::{ChannelId, EgressId};

	use super::*;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: Chainflip {
		/// Because we want to emit events when there is a config change during
		/// an runtime upgrade
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// API for handling asset deposits.
		type DepositHandler: DepositApi<
			AnyChain,
			AccountId = <Self as frame_system::Config>::AccountId,
		>;

		/// API for handling asset egress.
		type EgressHandler: EgressApi<AnyChain>;

		/// A converter to convert address to and from human readable to internal address
		/// representation.
		type AddressConverter: AddressConverter;

		/// Safe Mode access.
		type SafeMode: Get<PalletSafeMode>;

		/// The interface for sweeping funds from pools into free balance
		type PoolApi: PoolApi;

		/// Benchmark weights
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The user does not have enough fund.
		InsufficientBalance,
		/// The user has reached the maximum balance.
		BalanceOverflow,
		/// The caller is not authorized to modify the trading position.
		UnauthorisedToModify,
		/// The Asset cannot be egressed because the destination address is not invalid.
		InvalidEgressAddress,
		/// Then given encoded address cannot be decoded into a valid ForeignChainAddress.
		InvalidEncodedAddress,
		/// An liquidity refund address must be set by the user for the chain before
		/// deposit address can be requested.
		NoLiquidityRefundAddressRegistered,
		/// Liquidity deposit is disabled due to Safe Mode.
		LiquidityDepositDisabled,
		/// Withdrawals are disabled due to Safe Mode.
		WithdrawalsDisabled,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AccountDebited {
			account_id: T::AccountId,
			asset: Asset,
			amount_debited: AssetAmount,
		},
		AccountCredited {
			account_id: T::AccountId,
			asset: Asset,
			amount_credited: AssetAmount,
		},
		LiquidityDepositAddressReady {
			channel_id: ChannelId,
			asset: Asset,
			deposit_address: EncodedAddress,
			// account the funds will be credited to upon deposit
			account_id: T::AccountId,
			deposit_chain_expiry_block: <AnyChain as Chain>::ChainBlockNumber,
		},
		WithdrawalEgressScheduled {
			egress_id: EgressId,
			asset: Asset,
			amount: AssetAmount,
			destination_address: EncodedAddress,
		},
		LiquidityRefundAddressRegistered {
			account_id: T::AccountId,
			chain: ForeignChain,
			address: ForeignChainAddress,
		},
	}

	#[pallet::pallet]
	#[pallet::storage_version(PALLET_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	/// Storage for user's free balances/ DoubleMap: (AccountId, Asset) => Balance
	pub type FreeBalances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Identity, Asset, AssetAmount>;

	/// Stores the registered energency withdrawal address for an Account
	#[pallet::storage]
	pub type LiquidityRefundAddress<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::AccountId,
		Twox64Concat,
		ForeignChain,
		ForeignChainAddress,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// For when the user wants to deposit assets into the Chain.
		/// Generates a new deposit address for the user to posit their assets.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::request_liquidity_deposit_address())]
		pub fn request_liquidity_deposit_address(
			origin: OriginFor<T>,
			asset: Asset,
		) -> DispatchResult {
			ensure!(T::SafeMode::get().deposit_enabled, Error::<T>::LiquidityDepositDisabled);

			let account_id = T::AccountRoleRegistry::ensure_liquidity_provider(origin)?;

			ensure!(
				LiquidityRefundAddress::<T>::contains_key(&account_id, ForeignChain::from(asset)),
				Error::<T>::NoLiquidityRefundAddressRegistered
			);

			let (channel_id, deposit_address, expiry_block) =
				T::DepositHandler::request_liquidity_deposit_address(account_id.clone(), asset)?;

			Self::deposit_event(Event::LiquidityDepositAddressReady {
				channel_id,
				asset,
				deposit_address: T::AddressConverter::to_encoded_address(deposit_address),
				account_id,
				deposit_chain_expiry_block: expiry_block,
			});

			Ok(())
		}

		/// For when the user wants to withdraw their free balances out of the chain.
		/// Requires a valid foreign chain address.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::withdraw_asset())]
		pub fn withdraw_asset(
			origin: OriginFor<T>,
			amount: AssetAmount,
			asset: Asset,
			destination_address: EncodedAddress,
		) -> DispatchResult {
			ensure!(T::SafeMode::get().withdrawal_enabled, Error::<T>::WithdrawalsDisabled);
			if amount > 0 {
				let account_id = T::AccountRoleRegistry::ensure_liquidity_provider(origin)?;

				let destination_address_internal =
					T::AddressConverter::try_from_encoded_address(destination_address.clone())
						.map_err(|_| Error::<T>::InvalidEgressAddress)?;

				// Check validity of Chain and Asset
				ensure!(
					destination_address_internal.chain() == ForeignChain::from(asset),
					Error::<T>::InvalidEgressAddress
				);

				// Debit the asset from the account.
				Self::try_debit_account(&account_id, asset, amount)?;

				let egress_id = T::EgressHandler::schedule_egress(
					asset,
					amount,
					destination_address_internal,
					None,
				);

				Self::deposit_event(Event::<T>::WithdrawalEgressScheduled {
					egress_id,
					asset,
					amount,
					destination_address,
				});
			}
			Ok(())
		}

		/// Register the account as a Liquidity Provider.
		/// Account roles are immutable once registered.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::register_lp_account())]
		pub fn register_lp_account(who: OriginFor<T>) -> DispatchResult {
			let account_id = ensure_signed(who)?;

			T::AccountRoleRegistry::register_as_liquidity_provider(&account_id)?;

			Ok(())
		}

		/// Registers an Liquidity Refund Address(LRA) for an account.
		/// To request deposit address for a chain, an LRA must be registered for that chain.
		///
		/// ## Events
		///
		/// - [On Success](Event::LiquidityRefundAddressRegistered)
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::register_liquidity_refund_address())]
		pub fn register_liquidity_refund_address(
			origin: OriginFor<T>,
			address: EncodedAddress,
		) -> DispatchResult {
			let account_id = T::AccountRoleRegistry::ensure_liquidity_provider(origin)?;

			let decoded_address = T::AddressConverter::try_from_encoded_address(address)
				.map_err(|()| Error::<T>::InvalidEncodedAddress)?;

			LiquidityRefundAddress::<T>::insert(
				&account_id,
				decoded_address.chain(),
				decoded_address.clone(),
			);

			Self::deposit_event(Event::<T>::LiquidityRefundAddressRegistered {
				account_id,
				chain: decoded_address.chain(),
				address: decoded_address,
			});
			Ok(())
		}
	}
}

impl<T: Config> LpBalanceApi for Pallet<T> {
	type AccountId = <T as frame_system::Config>::AccountId;

	#[cfg(feature = "runtime-benchmarks")]
	fn register_liquidity_refund_address(
		account_id: &Self::AccountId,
		address: ForeignChainAddress,
	) {
		LiquidityRefundAddress::<T>::insert(account_id, address.chain(), address);
	}

	fn ensure_has_refund_address_for_pair(
		account_id: &Self::AccountId,
		base_asset: Asset,
		pair_asset: Asset,
	) -> DispatchResult {
		ensure!(
			LiquidityRefundAddress::<T>::contains_key(account_id, ForeignChain::from(base_asset)) &&
				LiquidityRefundAddress::<T>::contains_key(
					account_id,
					ForeignChain::from(pair_asset)
				),
			Error::<T>::NoLiquidityRefundAddressRegistered
		);
		Ok(())
	}

	fn try_credit_account(
		account_id: &Self::AccountId,
		asset: Asset,
		amount: AssetAmount,
	) -> DispatchResult {
		if amount == 0 {
			return Ok(())
		}

		let mut balance = FreeBalances::<T>::get(account_id, asset).unwrap_or_default();
		balance = balance.checked_add(amount).ok_or(Error::<T>::BalanceOverflow)?;
		FreeBalances::<T>::insert(account_id, asset, balance);

		Self::deposit_event(Event::AccountCredited {
			account_id: account_id.clone(),
			asset,
			amount_credited: amount,
		});
		Ok(())
	}

	fn try_debit_account(
		account_id: &Self::AccountId,
		asset: Asset,
		amount: AssetAmount,
	) -> DispatchResult {
		if amount == 0 {
			return Ok(())
		}

		let mut balance = FreeBalances::<T>::get(account_id, asset).unwrap_or_default();
		ensure!(balance >= amount, Error::<T>::InsufficientBalance);
		balance = balance.saturating_sub(amount);
		FreeBalances::<T>::insert(account_id, asset, balance);

		Self::deposit_event(Event::AccountDebited {
			account_id: account_id.clone(),
			asset,
			amount_debited: amount,
		});
		Ok(())
	}
}
