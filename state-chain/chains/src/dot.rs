use crate::*;

pub mod batch_fetch;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use sp_core::{sr25519, H256};
use sp_runtime::{
	generic::{Era, SignedPayload, UncheckedExtrinsic},
	traits::{BlakeTwo256, DispatchInfoOf, Hash, SignedExtension, StaticLookup},
	MultiAddress, MultiSignature,
};

use sp_core::crypto::AccountId32;
use sp_runtime::{
	traits::{AccountIdLookup, Verify},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
};

use codec::{Decode, Encode};
use core::str::FromStr;
use scale_info::TypeInfo;

pub type PolkadotSignature = sr25519::Signature;

pub type PolkadotGovKey = (); // Todo

pub type PolkadotBalance = u128;
pub type PolkadotBlockNumber = u32;
pub type PolkadotIndex = u32;
pub type PolkadotHash = sp_core::H256;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type PolkadotAccountId = AccountId32;

pub type PolkadotAddress = MultiAddress<PolkadotAccountId, ()>;

pub type PolkadotAccountIdLookup = <AccountIdLookup<PolkadotAccountId, ()> as StaticLookup>::Source;

pub type PolkadotCallHasher = BlakeTwo256;

pub type PolkadotCallHash = <PolkadotCallHasher as Hash>::Output;

pub type PolkadotSpecVersion = u32;
pub type PolkadotTransactionVersion = u32;

pub type PolkadotUncheckedExtrinsic =
	UncheckedExtrinsic<PolkadotAddress, PolkadotRuntimeCall, MultiSignature, PolkadotSignedExtra>;
/// The payload being signed in transactions.
pub type PolkadotPayload = SignedPayload<PolkadotRuntimeCall, PolkadotSignedExtra>;
pub type EncodedPolkadotPayload = Vec<u8>;

// Polkadot mainnet
// pub const POLKADOT_BLOCK_HASH_COUNT: PolkadotBlockNumber = 2400; //import from runtime common
// types crate in polkadot repo pub const POLKADOT_SPEC_VERSION: PolkadotSpecVersion = 9290;
// pub const POLKADOT_TRANSACTION_VERSION: PolkadotTransactionVersion = 14;
// pub const POLKADOT_GENESIS_HASH: &str =
// 	"0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3";

// Westend testnet
pub const POLKADOT_BLOCK_HASH_COUNT: PolkadotBlockNumber = 4096; //import from runtime common types crate in polkadot repo
pub const POLKADOT_SPEC_VERSION: PolkadotSpecVersion = 9300;
pub const POLKADOT_TRANSACTION_VERSION: PolkadotTransactionVersion = 13;
pub const POLKADOT_GENESIS_HASH: &str =
	"0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e";

#[allow(clippy::unnecessary_cast)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum PolkadotProxyType {
	Any = 0,
	NonTransfer = 1,
	Governance = 2,
	Staking = 3,
	// Skip 4 as it is now removed (was SudoBalances)
	IdentityJudgement = 5,
	CancelProxy = 6,
	Auction = 7,
}

#[derive(Copy, Clone, RuntimeDebug, Default, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Polkadot;

impl Chain for Polkadot {
	type ChainBlockNumber = u64;
	type ChainAmount = u128;
	type TrackedData = eth::TrackedData<Self>;
	type ChainAccount = PolkadotAccountId;
	type ChainAsset = ();
}

impl ChainCrypto for Polkadot {
	type AggKey = PolkadotPublicKey;
	type Payload = EncodedPolkadotPayload;
	type ThresholdSignature = PolkadotSignature;
	type TransactionHash = PolkadotHash;
	type GovKey = PolkadotGovKey;

	fn verify_threshold_signature(
		agg_key: &Self::AggKey,
		payload: &Self::Payload,
		signature: &Self::ThresholdSignature,
	) -> bool {
		signature.verify(&payload[..], &agg_key.0)
	}

	fn agg_key_to_payload(agg_key: Self::AggKey) -> Self::Payload {
		Blake2_256::hash(&agg_key.0).to_vec()
	}
}

impl ChainAbi for Polkadot {
	type UnsignedTransaction = ();
	type SignedTransaction = Vec<u8>;
	// Not needed in Polkadot since we can sign natively with the AggKey.
	type SignerCredential = ();
	type ReplayProtection = (); //Todo
	type ValidationError = ();

	// This function is not needed in Polkadot.
	fn verify_signed_transaction(
		_unsigned_tx: &Self::UnsignedTransaction,
		_signed_tx: &Self::SignedTransaction,
		_signer_credential: &Self::SignerCredential,
	) -> Result<Self::TransactionHash, Self::ValidationError> {
		Err(())
	}
}

/// The handler for creating and signing polkadot extrinsics, and creating signature payload
#[derive(Debug, Encode, Decode, TypeInfo, Eq, PartialEq, Clone)]
pub struct PolkadotExtrinsicHandler {
	vault_account: <Polkadot as Chain>::ChainAccount,
	extrinsic_call: Option<PolkadotRuntimeCall>,
	signed_extrinsic: Option<PolkadotUncheckedExtrinsic>,
	nonce: PolkadotIndex,
	extra: Option<PolkadotSignedExtra>,
	signature_payload: Option<<Polkadot as ChainCrypto>::Payload>,
}

impl PolkadotExtrinsicHandler {
	pub fn new_empty(
		nonce: PolkadotIndex,
		vault_account: <Polkadot as Chain>::ChainAccount,
	) -> Self {
		Self {
			nonce,
			vault_account,
			extrinsic_call: None,
			signed_extrinsic: None,
			extra: None,
			signature_payload: None,
		}
	}

	pub fn insert_extrinsic_call(&mut self, extrinsic_call: PolkadotRuntimeCall) {
		self.extrinsic_call = Some(extrinsic_call);
	}

	pub fn insert_threshold_signature_payload(
		&mut self,
	) -> Option<<Polkadot as ChainCrypto>::Payload> {
		let tip = 0;
		let extra: PolkadotSignedExtra = PolkadotSignedExtra((
			(),
			(),
			(),
			(),
			PolkadotCheckMortality(Era::Immortal),
			PolkadotCheckNonce(self.nonce),
			(),
			PolkadotChargeTransactionPayment(tip),
			(),
		));
		let raw_payload = PolkadotPayload::new(self.extrinsic_call.clone()?, extra)
			.map_err(|e| {
				// This should not happen since the SignedExtension of the Extra type is implemented
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		self.signature_payload =
			raw_payload.using_encoded(|encoded_payload| Some(encoded_payload.to_vec()));
		self.extra = Some(extra);

		self.signature_payload.clone()
	}

	pub fn insert_signature_and_get_signed_unchecked_extrinsic(
		&mut self,
		signature: <Polkadot as ChainCrypto>::ThresholdSignature,
	) -> Option<PolkadotUncheckedExtrinsic> {
		self.signed_extrinsic = Some(PolkadotUncheckedExtrinsic::new_signed(
			self.extrinsic_call.clone()?,
			PolkadotAddress::Id(self.vault_account.clone()),
			sp_runtime::MultiSignature::Sr25519(signature),
			self.extra?,
		));
		self.signed_extrinsic.clone()
	}

	pub fn is_signed(&self) -> Option<bool> {
		match self.signed_extrinsic.clone()?.signature {
			Some((_signed, signature, extra)) => {
				let raw_payload =
					SignedPayload::new(self.signed_extrinsic.clone()?.function, extra).ok()?;
				if !raw_payload
					.using_encoded(|payload| signature.verify(payload, &self.vault_account))
				{
					Some(false)
				} else {
					Some(true)
				}
			},
			None => Some(false),
		}
	}
}

// The Polkadot Runtime type that is expected by the polkadot runtime
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub enum PolkadotRuntimeCall {
	#[codec(index = 0u8)]
	System(SystemCall),
	#[codec(index = 4u8)] // INDEX FOR WESTEND: 4, FOR POLKADOT: 5
	Balances(BalancesCall),
	#[codec(index = 26u8)]
	Utility(UtilityCall),
	#[codec(index = 29u8)]
	Proxy(ProxyCall),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub enum SystemCall {}

#[allow(non_camel_case_types)]
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub enum BalancesCall {
	/// Transfer some liquid free balance to another account.
	///
	/// `transfer` will set the `FreeBalance` of the sender and receiver.
	/// If the sender's account is below the existential deposit as a result
	/// of the transfer, the account will be reaped.
	///
	/// The dispatch origin for this call must be `Signed` by the transactor.
	///
	/// # <weight>
	/// - Dependent on arguments but not critical, given proper implementations for input config
	///   types. See related functions below.
	/// - It contains a limited number of reads and writes internally and no complex computation.
	///
	/// Related functions:
	///
	///   - `ensure_can_withdraw` is always called internally but has a bounded complexity.
	///   - Transferring balances to accounts that did not exist before will cause
	///     `T::OnNewAccount::on_new_account` to be called.
	///   - Removing enough funds from an account will trigger `T::DustRemoval::on_unbalanced`.
	///   - `transfer_keep_alive` works the same way as `transfer`, but has an additional check
	///     that the transfer will not kill the origin account.
	/// ---------------------------------
	/// - Origin account is already in memory, so no DB operations for them.
	/// # </weight>
	#[codec(index = 0u8)]
	transfer {
		#[allow(missing_docs)]
		dest: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		#[codec(compact)]
		value: PolkadotBalance,
	},
	/// Transfer the entire transferable balance from the caller account.
	///
	/// NOTE: This function only attempts to transfer _transferable_ balances. This means that
	/// any locked, reserved, or existential deposits (when `keep_alive` is `true`), will not be
	/// transferred by this function. To ensure that this function results in a killed account,
	/// you might need to prepare the account by removing any reference counters, storage
	/// deposits, etc...
	///
	/// The dispatch origin of this call must be Signed.
	///
	/// - `dest`: The recipient of the transfer.
	/// - `keep_alive`: A boolean to determine if the `transfer_all` operation should send all of
	///   the funds the account has, causing the sender account to be killed (false), or transfer
	///   everything except at least the existential deposit, which will guarantee to keep the
	///   sender account alive (true). # <weight>
	/// - O(1). Just like transfer, but reading the user's transferable balance first. #</weight>
	#[codec(index = 4u8)]
	transfer_all {
		#[allow(missing_docs)]
		dest: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		keep_alive: bool,
	},
}

#[allow(non_camel_case_types)]
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub enum UtilityCall {
	/// Send a batch of dispatch calls.
	///
	/// May be called from any origin.
	///
	/// - `calls`: The calls to be dispatched from the same origin. The number of call must not
	///   exceed the constant: `batched_calls_limit` (available in constant metadata).
	///
	/// If origin is root then call are dispatch without checking origin filter. (This includes
	/// bypassing `frame_system::Config::BaseCallFilter`).
	///
	/// # <weight>
	/// - Complexity: O(C) where C is the number of calls to be batched.
	/// # </weight>
	///
	/// This will return `Ok` in all circumstances. To determine the success of the batch, an
	/// event is deposited. If a call failed and the batch was interrupted, then the
	/// `BatchInterrupted` event is deposited, along with the number of successful calls made
	/// and the error of the failed call. If all were successful, then the `BatchCompleted`
	/// event is deposited.
	#[codec(index = 0u8)]
	batch {
		#[allow(missing_docs)]
		calls: Vec<PolkadotRuntimeCall>,
	},
	/// Send a call through an indexed pseudonym of the sender.
	///
	/// Filter from origin are passed along. The call will be dispatched with an origin which
	/// use the same filter as the origin of this call.
	///
	/// NOTE: If you need to ensure that any account-based filtering is not honored (i.e.
	/// because you expect `proxy` to have been used prior in the call stack and you do not want
	/// the call restrictions to apply to any sub-accounts), then use `as_multi_threshold_1`
	/// in the Multisig pallet instead.
	///
	/// NOTE: Prior to version *12, this was called `as_limited_sub`.
	///
	/// The dispatch origin for this call must be _Signed_.
	#[codec(index = 1u8)]
	as_derivative {
		#[allow(missing_docs)]
		index: u16,
		#[allow(missing_docs)]
		call: Box<PolkadotRuntimeCall>,
	},
	/// Send a batch of dispatch calls and atomically execute them.
	/// The whole transaction will rollback and fail if any of the calls failed.
	///
	/// May be called from any origin.
	///
	/// - `calls`: The calls to be dispatched from the same origin. The number of call must not
	///   exceed the constant: `batched_calls_limit` (available in constant metadata).
	///
	/// If origin is root then call are dispatch without checking origin filter. (This includes
	/// bypassing `frame_system::Config::BaseCallFilter`).
	///
	/// # <weight>
	/// - Complexity: O(C) where C is the number of calls to be batched.
	/// # </weight>
	#[codec(index = 2u8)]
	batch_all {
		#[allow(missing_docs)]
		calls: Vec<PolkadotRuntimeCall>,
	},
}

#[allow(non_camel_case_types)]
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub enum ProxyCall {
	/// Dispatch the given `call` from an account that the sender is authorised for through
	/// `add_proxy`.
	///
	/// Removes any corresponding announcement(s).
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
	/// - `call`: The call to be made by the `real` account.
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	#[codec(index = 0u8)]
	proxy {
		#[allow(missing_docs)]
		real: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		force_proxy_type: Option<PolkadotProxyType>,
		#[allow(missing_docs)]
		call: Box<PolkadotRuntimeCall>,
	},
	/// Register a proxy account for the sender that is able to make calls on its behalf.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `proxy`: The account that the `caller` would like to make a proxy.
	/// - `proxy_type`: The permissions allowed for this proxy account.
	/// - `delay`: The announcement period required of the initial proxy. Will generally be
	/// zero.
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	#[codec(index = 1u8)]
	add_proxy {
		#[allow(missing_docs)]
		delegate: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		proxy_type: PolkadotProxyType,
		#[allow(missing_docs)]
		delay: PolkadotBlockNumber,
	},
	/// Unregister a proxy account for the sender.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `proxy`: The account that the `caller` would like to remove as a proxy.
	/// - `proxy_type`: The permissions currently enabled for the removed proxy account.
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	#[codec(index = 2u8)]
	remove_proxy {
		#[allow(missing_docs)]
		delegate: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		proxy_type: PolkadotProxyType,
		#[allow(missing_docs)]
		delay: PolkadotBlockNumber,
	},
	/// Unregister all proxy accounts for the sender.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// WARNING: This may be called on accounts created by `anonymous`, however if done, then
	/// the unreserved fees will be inaccessible. **All access to this account will be lost.**
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	#[codec(index = 3u8)]
	remove_proxies {},
	/// Spawn a fresh new account that is guaranteed to be otherwise inaccessible, and
	/// initialize it with a proxy of `proxy_type` for `origin` sender.
	///
	/// Requires a `Signed` origin.
	///
	/// - `proxy_type`: The type of the proxy that the sender will be registered as over the
	/// new account. This will almost always be the most permissive `ProxyType` possible to
	/// allow for maximum flexibility.
	/// - `index`: A disambiguation index, in case this is called multiple times in the same
	/// transaction (e.g. with `utility::batch`). Unless you're using `batch` you probably just
	/// want to use `0`.
	/// - `delay`: The announcement period required of the initial proxy. Will generally be
	/// zero.
	///
	/// Fails with `Duplicate` if this has already been called in this transaction, from the
	/// same sender, with the same parameters.
	///
	/// Fails if there are insufficient funds to pay for deposit.
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	/// TODO: Might be over counting 1 read
	#[codec(index = 4u8)]
	anonymous {
		#[allow(missing_docs)]
		proxy_type: PolkadotProxyType,
		#[allow(missing_docs)]
		delay: PolkadotBlockNumber,
		#[allow(missing_docs)]
		index: u16,
	},
	/// Removes a previously spawned anonymous proxy.
	///
	/// WARNING: **All access to this account will be lost.** Any funds held in it will be
	/// inaccessible.
	///
	/// Requires a `Signed` origin, and the sender account must have been created by a call to
	/// `anonymous` with corresponding parameters.
	///
	/// - `spawner`: The account that originally called `anonymous` to create this account.
	/// - `index`: The disambiguation index originally passed to `anonymous`. Probably `0`.
	/// - `proxy_type`: The proxy type originally passed to `anonymous`.
	/// - `height`: The height of the chain when the call to `anonymous` was processed.
	/// - `ext_index`: The extrinsic index in which the call to `anonymous` was processed.
	///
	/// Fails with `NoPermission` in case the caller is not a previously created anonymous
	/// account whose `anonymous` call has corresponding parameters.
	///
	/// # <weight>
	/// Weight is a function of the number of proxies the user has (P).
	/// # </weight>
	#[codec(index = 5u8)]
	kill_anonymous {
		#[allow(missing_docs)]
		spawner: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		proxy_type: PolkadotProxyType,
		#[allow(missing_docs)]
		index: u16,
		#[allow(missing_docs)]
		#[codec(compact)]
		height: PolkadotBlockNumber,
		#[allow(missing_docs)]
		#[codec(compact)]
		ext_index: u32,
	},
	/// Publish the hash of a proxy-call that will be made in the future.
	///
	/// This must be called some number of blocks before the corresponding `proxy` is attempted
	/// if the delay associated with the proxy relationship is greater than zero.
	///
	/// No more than `MaxPending` announcements may be made at any one time.
	///
	/// This will take a deposit of `AnnouncementDepositFactor` as well as
	/// `AnnouncementDepositBase` if there are no other pending announcements.
	///
	/// The dispatch origin for this call must be _Signed_ and a proxy of `real`.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `call_hash`: The hash of the call to be made by the `real` account.
	///
	/// # <weight>
	/// Weight is a function of:
	/// - A: the number of announcements made.
	/// - P: the number of proxies the user has.
	/// # </weight>
	#[codec(index = 6u8)]
	announce {
		#[allow(missing_docs)]
		real: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		call_hash: PolkadotCallHash,
	},
	/// Remove a given announcement.
	///
	/// May be called by a proxy account to remove a call they previously announced and return
	/// the deposit.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `call_hash`: The hash of the call to be made by the `real` account.
	///
	/// # <weight>
	/// Weight is a function of:
	/// - A: the number of announcements made.
	/// - P: the number of proxies the user has.
	/// # </weight>
	#[codec(index = 7u8)]
	remove_announcement {
		#[allow(missing_docs)]
		real: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		call_hash: PolkadotCallHash,
	},
	/// Remove the given announcement of a delegate.
	///
	/// May be called by a target (proxied) account to remove a call that one of their delegates
	/// (`delegate`) has announced they want to execute. The deposit is returned.
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `delegate`: The account that previously announced the call.
	/// - `call_hash`: The hash of the call to be made.
	///
	/// # <weight>
	/// Weight is a function of:
	/// - A: the number of announcements made.
	/// - P: the number of proxies the user has.
	/// # </weight>
	#[codec(index = 8u8)]
	reject_announcement {
		#[allow(missing_docs)]
		delegate: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		call_hash: PolkadotCallHash,
	},
	/// Dispatch the given `call` from an account that the sender is authorized for through
	/// `add_proxy`.
	///
	/// Removes any corresponding announcement(s).
	///
	/// The dispatch origin for this call must be _Signed_.
	///
	/// Parameters:
	/// - `real`: The account that the proxy will make a call on behalf of.
	/// - `force_proxy_type`: Specify the exact proxy type to be used and checked for this call.
	/// - `call`: The call to be made by the `real` account.
	///
	/// # <weight>
	/// Weight is a function of:
	/// - A: the number of announcements made.
	/// - P: the number of proxies the user has.
	/// # </weight>
	#[codec(index = 9u8)]
	proxy_announced {
		#[allow(missing_docs)]
		delegate: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		real: PolkadotAccountIdLookup,
		#[allow(missing_docs)]
		force_proxy_type: Option<PolkadotProxyType>,
		#[allow(missing_docs)]
		call: Box<PolkadotRuntimeCall>,
	},
}
#[derive(Debug, Encode, Decode, Copy, Clone, Eq, PartialEq, TypeInfo)]
pub struct PolkadotChargeTransactionPayment(#[codec(compact)] PolkadotBalance);

#[derive(Debug, Encode, Decode, Copy, Clone, Eq, PartialEq, TypeInfo)]
pub struct PolkadotCheckNonce(#[codec(compact)] pub PolkadotIndex);

#[derive(Debug, Encode, Decode, Copy, Clone, Eq, PartialEq, TypeInfo)]
pub struct PolkadotCheckMortality(pub Era);

#[derive(Debug, Encode, Decode, Copy, Clone, Eq, PartialEq, TypeInfo)]
pub struct PolkadotSignedExtra(
	pub  (
		(),
		(),
		(),
		(),
		PolkadotCheckMortality,
		PolkadotCheckNonce,
		(),
		PolkadotChargeTransactionPayment,
		(),
	),
);

impl SignedExtension for PolkadotSignedExtra {
	type AccountId = PolkadotAccountId;
	type Call = ();
	type AdditionalSigned = (
		(),
		PolkadotSpecVersion,
		PolkadotTransactionVersion,
		PolkadotHash,
		PolkadotHash,
		(),
		(),
		(),
		(),
	);
	type Pre = ();
	const IDENTIFIER: &'static str = "PolkadotSignedExtra";

	fn additional_signed(
		&self,
	) -> sp_std::result::Result<Self::AdditionalSigned, TransactionValidityError> {
		Ok((
			(),
			POLKADOT_SPEC_VERSION,
			POLKADOT_TRANSACTION_VERSION,
			H256::from_str(POLKADOT_GENESIS_HASH).unwrap(),
			H256::from_str(POLKADOT_GENESIS_HASH).unwrap(),
			(),
			(),
			(),
			(),
		))
	}

	fn pre_dispatch(
		self,
		_who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		_who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		Ok(<ValidTransaction as Default>::default())
	}
}

#[derive(Ord, PartialOrd, Debug, Encode, Decode, Copy, Clone, Eq, PartialEq, TypeInfo)]
pub struct PolkadotPublicKey(pub sr25519::Public);

impl TryFrom<Vec<u8>> for PolkadotPublicKey {
	type Error = ();

	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		data.as_slice().try_into().map(Self)
	}
}

impl From<PolkadotPublicKey> for Vec<u8> {
	fn from(k: PolkadotPublicKey) -> Self {
		k.0.to_vec()
	}
}

#[cfg(test)]
mod test_polkadot_extrinsics {

	use super::*;
	use crate::dot::sr25519::Pair;
	use sp_core::crypto::Pair as TraitPair;
	use sp_runtime::{traits::IdentifyAccount, MultiSigner};

	// test westend account 1 (CHAINFLIP-TEST)
	// address: "5E2WfQFeafdktJ5AAF6ZGZ71Yj4fiJnHWRomVmeoStMNhoZe"
	pub const RAW_SEED_1: [u8; 32] =
		hex_literal::hex!("858c1ee915090a119d4cb0774b908fa585ef7882f4648c577606490cc94f6e15");
	pub const NONCE_1: u32 = 4; //correct nonce has to be provided for this account (see/track onchain)

	// test westend account 2 (CHAINFLIP-TEST-2)
	// address: "5GNn92C9ngX4sNp3UjqGzPbdRfbbV8hyyVVNZaH2z9e5kzxA"
	pub const RAW_SEED_2: [u8; 32] =
		hex_literal::hex!("4b734882accd7a0e27b8b0d3cb7db79ab4da559d1d5f84f35fd218a1ee12ece4");
	pub const _NONCE_2: u32 = 1; //correct nonce has to be provided for this account (see/track onchain)

	#[ignore]
	#[test]
	fn create_test_extrinsic() {
		let keypair_1: Pair = <Pair as TraitPair>::from_seed(&RAW_SEED_1);
		let keypair_2: Pair = <Pair as TraitPair>::from_seed(&RAW_SEED_2);

		let account_id_1: AccountId32 = MultiSigner::Sr25519(keypair_1.public()).into_account();
		let account_id_2: AccountId32 = MultiSigner::Sr25519(keypair_2.public()).into_account();

		let test_runtime_call: PolkadotRuntimeCall =
			PolkadotRuntimeCall::Balances(BalancesCall::transfer {
				dest: PolkadotAccountIdLookup::from(account_id_2),
				value: 35_000_000_000u128, //0.035 WND
			});

		println!(
			"CallHash: 0x{}",
			test_runtime_call.using_encoded(|encoded| hex::encode(Blake2_256::hash(encoded)))
		);
		println!("Encoded Call: 0x{}", hex::encode(test_runtime_call.encode()));

		let mut extrinsic_handler = PolkadotExtrinsicHandler::new_empty(NONCE_1, account_id_1);
		extrinsic_handler.insert_extrinsic_call(test_runtime_call);
		extrinsic_handler
			.insert_threshold_signature_payload()
			.expect("This shouldn't fail");

		let signed_extrinsic: Option<PolkadotUncheckedExtrinsic> = extrinsic_handler
			.insert_signature_and_get_signed_unchecked_extrinsic(
				keypair_1.sign(
					&extrinsic_handler.signature_payload.clone().expect("This can't fail")[..],
				),
			);

		assert!(extrinsic_handler.is_signed().unwrap_or(false));

		println!("encoded extrinsic: 0x{}", hex::encode(signed_extrinsic.unwrap().encode()));
	}
}