use crate::{Module, RawEvent, Trait};
use frame_support::{assert_err, assert_ok, impl_outer_event, impl_outer_origin, parameter_types};
use frame_system as system;
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

impl_outer_origin! {
	pub enum Origin for TestRuntime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for TestRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

mod storage_cache {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		storage_cache<T>,
		system<T>,
	}
}

impl Trait for TestRuntime {
	type Event = TestEvent;
}

pub type System = system::Module<TestRuntime>;
pub type StorageCache = Module<TestRuntime>;

struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> sp_io::TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		let mut ext = sp_io::TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[test]
fn init_storage() {
	ExternalityBuilder::build().execute_with(|| {
		assert_ok!(StorageCache::set_copy(Origin::signed(1), 10));
		assert_eq!(StorageCache::some_copy_value(), 10);

		assert_ok!(StorageCache::set_king(Origin::signed(2)));
		assert_eq!(StorageCache::king_member(), 2);

		assert_ok!(StorageCache::mock_add_member(Origin::signed(1)));
		assert_err!(
			StorageCache::mock_add_member(Origin::signed(1)),
			"member already in group"
		);
		assert!(StorageCache::group_members().contains(&1));
	})
}

#[test]
fn increase_value_errs_on_overflow() {
	ExternalityBuilder::build().execute_with(|| {
		let num1: u32 = u32::max_value() - 9;
		assert_ok!(StorageCache::set_copy(Origin::signed(1), num1));
		// test first overflow panic for both methods
		assert_err!(
			StorageCache::increase_value_no_cache(Origin::signed(1), 10),
			"addition overflowed1"
		);
		assert_err!(
			StorageCache::increase_value_w_copy(Origin::signed(1), 10),
			"addition overflowed1"
		);

		let num2: u32 = 2147483643;
		assert_ok!(StorageCache::set_copy(Origin::signed(1), num2));
		// test second overflow panic for both methods
		assert_err!(
			StorageCache::increase_value_no_cache(Origin::signed(1), 10),
			"addition overflowed2"
		);
		assert_err!(
			StorageCache::increase_value_w_copy(Origin::signed(1), 10),
			"addition overflowed2"
		);
	})
}

#[test]
fn increase_value_works() {
	ExternalityBuilder::build().execute_with(|| {
		System::set_block_number(5);
		assert_ok!(StorageCache::set_copy(Origin::signed(1), 25));
		assert_ok!(StorageCache::increase_value_no_cache(Origin::signed(1), 10));
		// proof: x = 25, 2x + 10 = 60 qed
		let expected_event1 = TestEvent::storage_cache(RawEvent::InefficientValueChange(60, 5));
		assert!(System::events().iter().any(|a| a.event == expected_event1));

		// Ensure the storage value has actually changed from the first call
		assert_eq!(StorageCache::some_copy_value(), 60);

		assert_ok!(StorageCache::increase_value_w_copy(Origin::signed(1), 10));
		// proof: x = 60, 2x + 10 = 130
		let expected_event2 = TestEvent::storage_cache(RawEvent::BetterValueChange(130, 5));
		assert!(System::events().iter().any(|a| a.event == expected_event2));

		// check storage
		assert_eq!(StorageCache::some_copy_value(), 130);
	})
}

#[test]
fn swap_king_errs_as_intended() {
	ExternalityBuilder::build().execute_with(|| {
		assert_ok!(StorageCache::mock_add_member(Origin::signed(1)));
		assert_ok!(StorageCache::set_king(Origin::signed(1)));
		assert_err!(
			StorageCache::swap_king_no_cache(Origin::signed(3)),
			"current king is a member so maintains priority"
		);
		assert_err!(
			StorageCache::swap_king_with_cache(Origin::signed(3)),
			"current king is a member so maintains priority"
		);

		assert_ok!(StorageCache::set_king(Origin::signed(2)));
		assert_err!(
			StorageCache::swap_king_no_cache(Origin::signed(3)),
			"new king is not a member so doesn't get priority"
		);
		assert_err!(
			StorageCache::swap_king_with_cache(Origin::signed(3)),
			"new king is not a member so doesn't get priority"
		);
	})
}

#[test]
fn swap_king_works() {
	ExternalityBuilder::build().execute_with(|| {
		assert_ok!(StorageCache::mock_add_member(Origin::signed(2)));
		assert_ok!(StorageCache::mock_add_member(Origin::signed(3)));

		assert_ok!(StorageCache::set_king(Origin::signed(1)));
		assert_ok!(StorageCache::swap_king_no_cache(Origin::signed(2)));

		let expected_event = TestEvent::storage_cache(RawEvent::InefficientKingSwap(1, 2));
		assert!(System::events().iter().any(|a| a.event == expected_event));
		assert_eq!(StorageCache::king_member(), 2);

		assert_ok!(StorageCache::set_king(Origin::signed(1)));
		assert_eq!(StorageCache::king_member(), 1);
		assert_ok!(StorageCache::swap_king_with_cache(Origin::signed(3)));

		let expected_event = TestEvent::storage_cache(RawEvent::BetterKingSwap(1, 3));

		assert_eq!(
			System::events()[1].event,
			expected_event,
		);

		assert_eq!(StorageCache::king_member(), 3);
	})
}
