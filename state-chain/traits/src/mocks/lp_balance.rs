use crate::LpBalanceApi;
use cf_chains::assets::any::Asset;
use cf_primitives::AssetAmount;
use sp_runtime::DispatchResult;

#[cfg(feature = "runtime-benchmarks")]
use cf_chains::ForeignChainAddress;

pub struct MockBalance;
impl LpBalanceApi for MockBalance {
	type AccountId = u64;

	#[cfg(feature = "runtime-benchmarks")]
	fn register_liquidity_refund_address(_who: &Self::AccountId, _address: ForeignChainAddress) {}

	fn ensure_has_refund_address_for_pair(
		_who: &Self::AccountId,
		_base_asset: Asset,
		_pair_asset: Asset,
	) -> DispatchResult {
		Ok(())
	}

	fn try_credit_account(
		_who: &Self::AccountId,
		_asset: Asset,
		_amount: AssetAmount,
	) -> DispatchResult {
		Ok(())
	}

	fn try_debit_account(
		_who: &Self::AccountId,
		_asset: Asset,
		_amount: AssetAmount,
	) -> DispatchResult {
		Ok(())
	}
}
