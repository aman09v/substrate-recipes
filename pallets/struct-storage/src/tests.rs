use crate::*;
use frame_support::{assert_ok, impl_outer_event, impl_outer_origin, parameter_types};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{AtLeast32Bit, BlakeTwo256, IdentityLookup},
	Perbill,
};

// hacky Eq implementation for testing InnerThing
impl<Hash: Clone, Balance: Copy + AtLeast32Bit> PartialEq for InnerThing<Hash, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.number == other.number
	}
}
impl<Hash: Clone, Balance: Copy + AtLeast32Bit> Eq for InnerThing<Hash, Balance> {}
// "" for SuperThing
impl<Hash: Clone, Balance: Copy + AtLeast32Bit> PartialEq for SuperThing<Hash, Balance> {
	fn eq(&self, other: &Self) -> bool {
		self.super_number == other.super_number
	}
}
impl<Hash: Clone, Balance: Copy + AtLeast32Bit> Eq for SuperThing<Hash, Balance> {}

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
	type AccountData = balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}
// note: very unrealistic for most test envs
parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
}
impl balances::Trait for TestRuntime {
	type Balance = u64;
	type MaxLocks = ();
	type Event = TestEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = system::Module<TestRuntime>;
	type WeightInfo = ();
}

mod struct_storage {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		struct_storage<T>,
		system<T>,
		balances<T>,
	}
}

impl Trait for TestRuntime {
	type Event = TestEvent;
}

pub type System = system::Module<TestRuntime>;
pub type StructStorage = Module<TestRuntime>;

struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> TestExternalities {
		let storage = system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[test]
fn insert_inner_works() {
	ExternalityBuilder::build().execute_with(|| {
		// prepare hash
		let data = H256::from_low_u64_be(16);
		// insert inner thing
		assert_ok!(StructStorage::insert_inner_thing(
			Origin::signed(1),
			3u32,
			data,
			7u64.into()
		));

		// check storage matches expectations
		let expected_storage_item = InnerThing {
			number: 3u32,
			hash: data,
			balance: 7u64,
		};
		assert_eq!(
			StructStorage::inner_things_by_numbers(3u32),
			expected_storage_item
		);

		// check events emitted match expectations
		let expected_event = TestEvent::struct_storage(RawEvent::NewInnerThing(3u32, data, 7u64));

		assert_eq!(
			System::events()[0].event,
			expected_event,
		);
	})
}

#[test]
fn insert_super_thing_with_existing_works() {
	ExternalityBuilder::build().execute_with(|| {
		// prepare hash
		let data = H256::from_low_u64_be(16);
		// insert inner first (tested in direct test above)
		assert_ok!(StructStorage::insert_inner_thing(
			Origin::signed(1),
			3u32,
			data,
			7u64.into()
		));
		// insert super with existing inner
		assert_ok!(StructStorage::insert_super_thing_with_existing_inner(
			Origin::signed(1),
			3u32,
			5u32
		));

		// check storage matches expectations
		let expected_inner = InnerThing {
			number: 3u32,
			hash: data,
			balance: 7u64,
		};
		assert_eq!(StructStorage::inner_things_by_numbers(3u32), expected_inner);
		let expected_outer = SuperThing {
			super_number: 5u32,
			inner_thing: expected_inner.clone(),
		};
		assert_eq!(
			StructStorage::super_things_by_super_numbers(5u32),
			expected_outer
		);

		let expected_event = TestEvent::struct_storage(RawEvent::NewSuperThingByExistingInner(
			5u32,
			3u32,
			data,
			7u64.into(),
		));

		assert_eq!(
			System::events()[1].event,
			expected_event,
		);
	})
}

#[test]
fn insert_super_with_new_inner_works() {
	ExternalityBuilder::build().execute_with(|| {
		// prepare hash
		let data = H256::from_low_u64_be(16);
		// insert super with new inner
		assert_ok!(StructStorage::insert_super_thing_with_new_inner(
			Origin::signed(1),
			3u32,
			data,
			7u64.into(),
			5u32,
		));

		// check storage matches expectations
		let expected_inner = InnerThing {
			number: 3u32,
			hash: data,
			balance: 7u64,
		};
		assert_eq!(StructStorage::inner_things_by_numbers(3u32), expected_inner);
		let expected_outer = SuperThing {
			super_number: 5u32,
			inner_thing: expected_inner.clone(),
		};
		assert_eq!(
			StructStorage::super_things_by_super_numbers(5u32),
			expected_outer
		);

		//Test that the expected events were emitted
		let our_events = System::events()
		.into_iter().map(|r| r.event)
		.filter_map(|e| {
			if let TestEvent::struct_storage(inner) = e { Some(inner) } else { None }
		})
		.collect::<Vec<_>>();

		let expected_events = vec![
			RawEvent::NewInnerThing(3u32, data, 7u64),
			RawEvent::NewSuperThingByNewInner(
				5u32,
				3u32,
				data,
				7u64.into(),
			),
		];

		assert_eq!(our_events, expected_events);

	})
}
