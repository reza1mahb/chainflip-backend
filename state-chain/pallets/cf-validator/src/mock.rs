use super::*;
use crate as pallet_cf_validator;
//use sp_core::{sr25519};
use sp_core::H256;
use codec::{Encode, Decode};
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
use frame_support::{parameter_types, construct_runtime, traits::{OnInitialize, OnFinalize}};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		ValidatorManager: pallet_cf_validator::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
}

pub struct TestValidatorProvider;

fn account<AccountId: Decode + Default>(name: &'static str, index: u32, seed: u32) -> AccountId {
    let entropy = (name, index, seed).using_encoded(blake2_256);
    AccountId::decode(&mut &entropy[..]).unwrap_or_default()
}

impl<T: Config> ValidatorProvider<T> for TestValidatorProvider {
    fn get_validators() -> Option<Vec<T::AccountId>> {
        Some(vec![account("ALICE", 0, 0),
                  account("BOB", 1, 0),
                  account("CHARLIE", 2, 0)])
    }

    fn session_ending() {
        // Get ready for next set to be called in get_validators()
    }

    fn session_starting() {
        // New session starting
    }
}
parameter_types! {
	pub const MinEpoch: u64 = 1;
	pub const MinValidatorSetSize: u64 = 2;
}

impl Config for Test {
    type Event = Event;
    type MinEpoch = MinEpoch;
    type MinValidatorSetSize = MinValidatorSetSize;
    type ValidatorProvider = TestValidatorProvider;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    frame_system::GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        // ValidatorManager::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        // ValidatorManager::on_initialize(System::block_number());
    }
}
