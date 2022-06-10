//! Autogenerated weights for pallet_cf_threshold_signature
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-02-24, STEPS: `20`, REPEAT: 10, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./target/release/chainflip-node
// benchmark
// --extrinsic
// *
// --pallet
// pallet_cf_threshold-signature
// --output
// state-chain/pallets/cf-threshold-signature/src/weights.rs
// --execution=wasm
// --steps=20
// --repeat=10
// --template=state-chain/chainflip-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_cf_threshold_signature.
pub trait WeightInfo {
	fn signature_success() -> Weight;
	fn report_signature_failed(a: u32, ) -> Weight;
	fn on_initialize() -> Weight;
	fn determine_offenders(a: u32, ) -> Weight;
	fn set_threshold_signature_timeout() -> Weight;
}

/// Weights for pallet_cf_threshold_signature using the Substrate node and recommended hardware.
pub struct PalletWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for PalletWeight<T> {
	// Storage: EthereumThresholdSigner PendingRequests (r:1 w:1)
	// Storage: Environment StakeManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastAttemptIdCounter (r:1 w:1)
	// Storage: Validator Validators (r:1 w:0)
	// Storage: Online Nodes (r:150 w:0)
	// Storage: EthereumBroadcaster Expiries (r:1 w:1)
	// Storage: EthereumBroadcaster AwaitingTransactionSignature (r:0 w:1)
	fn signature_success() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(868_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(157 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: Validator Validators (r:1 w:0)
	// Storage: EthereumThresholdSigner PendingRequests (r:1 w:1)
	fn report_signature_failed(a: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(43_369_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((301_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn on_initialize() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(0 as Weight)
	}
	fn determine_offenders(a: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(157_000 as Weight)
			// Standard Error: 0
			.saturating_add((48_000 as Weight).saturating_mul(a as Weight))
	}
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:1)
	fn set_threshold_signature_timeout() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(7_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: EthereumThresholdSigner PendingRequests (r:1 w:1)
	// Storage: Environment StakeManagerAddress (r:1 w:0)
	// Storage: Environment EthereumChainId (r:1 w:0)
	// Storage: EthereumBroadcaster BroadcastIdCounter (r:1 w:1)
	// Storage: EthereumBroadcaster BroadcastAttemptIdCounter (r:1 w:1)
	// Storage: Validator Validators (r:1 w:0)
	// Storage: Online Nodes (r:150 w:0)
	// Storage: EthereumBroadcaster Expiries (r:1 w:1)
	// Storage: EthereumBroadcaster AwaitingTransactionSignature (r:0 w:1)
	fn signature_success() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(868_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(157 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	// Storage: Validator Validators (r:1 w:0)
	// Storage: EthereumThresholdSigner PendingRequests (r:1 w:1)
	fn report_signature_failed(a: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(43_369_000 as Weight)
			// Standard Error: 4_000
			.saturating_add((301_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(RocksDbWeight::get().reads(2 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn on_initialize() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(0 as Weight)
	}
	fn determine_offenders(a: u32, ) -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(157_000 as Weight)
			// Standard Error: 0
			.saturating_add((48_000 as Weight).saturating_mul(a as Weight))
	}
	// Storage: EthereumThresholdSigner ThresholdSignatureResponseTimeout (r:1 w:1)
	fn set_threshold_signature_timeout() -> Weight {
		#[allow(clippy::unnecessary_cast)]
		(7_000_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}
