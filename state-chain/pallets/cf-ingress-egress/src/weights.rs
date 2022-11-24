//! Autogenerated weights for pallet_cf_ingress_egress
//!
//! THIS FILE WAS AUTO-GENERATED USING CHAINFLIP NODE BENCHMARK CMD VERSION 4.0.0-dev
//! DATE: 2022-11-14, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("three-node-test"), DB CACHE: 1024

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// pallet
// --chain
// three-node-test
// --extrinsic
// *
// --pallet
// pallet_cf_ingress_egress
// --output
// state-chain/pallets/cf-ingress-egress/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_ingress_egress.
pub trait WeightInfo {
	fn egress_assets(n: u32, ) -> Weight;
	fn disable_asset_egress() -> Weight;
	fn on_idle_with_nothing_to_send() -> Weight;
	fn do_single_ingress() -> Weight;
}

/// Weights for pallet_cf_ingress_egress using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	// Storage: EthereumIngressEgress DisabledEgressAssets (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentKeyholdersEpoch (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner RetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	fn egress_assets(n: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(77_770_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((1_182_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(T::DbWeight::get().reads(16 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: EthereumIngressEgress DisabledEgressAssets (r:0 w:1)
	fn disable_asset_egress() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(10_000_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	fn on_idle_with_nothing_to_send() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(3_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: EthereumIngressEgress IntentIngressDetails (r:1 w:0)
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	// Storage: EthereumIngressEgress IntentActions (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn do_single_ingress() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(27_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	// Storage: EthereumIngressEgress DisabledEgressAssets (r:1 w:0)
	// Storage: Environment EthereumKeyManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: Environment EthereumSignatureNonce (r:1 w:1)
	// Storage: EthereumThresholdSigner ThresholdSignatureRequestIdCounter (r:1 w:1)
	// Storage: Validator CeremonyIdCounter (r:1 w:1)
	// Storage: EthereumVault CurrentKeyholdersEpoch (r:1 w:0)
	// Storage: EthereumVault Vaults (r:1 w:0)
	// Storage: Validator EpochAuthorityCount (r:1 w:0)
	// Storage: Reputation Suspensions (r:3 w:0)
	// Storage: Validator HistoricalAuthorities (r:1 w:0)
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:0)
	// Storage: EthereumThresholdSigner RetryQueues (r:1 w:1)
	// Storage: EthereumThresholdSigner Signature (r:0 w:1)
	// Storage: EthereumThresholdSigner PendingCeremonies (r:0 w:1)
	// Storage: EthereumThresholdSigner RequestCallback (r:0 w:1)
	fn egress_assets(n: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(77_770_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((1_182_000 as Weight).saturating_mul(n as Weight))
			.saturating_add(RocksDbWeight::get().reads(16 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	// Storage: EthereumIngressEgress DisabledEgressAssets (r:0 w:1)
	fn disable_asset_egress() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(10_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	fn on_idle_with_nothing_to_send() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(3_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: EthereumIngressEgress IntentIngressDetails (r:1 w:0)
	// Storage: EthereumIngressEgress ScheduledEgressRequests (r:1 w:1)
	// Storage: EthereumIngressEgress IntentActions (r:1 w:0)
	// Storage: LiquidityProvider FreeBalances (r:1 w:1)
	fn do_single_ingress() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(27_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(4 as Weight))
			.saturating_add(RocksDbWeight::get().writes(2 as Weight))
	}
}
