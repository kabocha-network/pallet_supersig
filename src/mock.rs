use crate as pallet_supersig;
use frame_support::{parameter_types, traits::Everything, PalletId};
use frame_system as system;
use sp_core::{sr25519, Pair, Public, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

#[frame_support::pallet]
pub mod nothing {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::event]
	pub enum Event<T: Config> {
		Nothing {},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		pub fn do_nothing(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			Ok(().into())
		}
	}
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Supersig: pallet_supersig::{Pallet, Call, Storage, Event<T>},
		Nothing: nothing::{Pallet, Call, Storage, Event<T>},

		Balances: pallet_balances,
	}
);

impl nothing::Config for Test {
	type Event = Event;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = ();
	type Version = ();
}

pub type Balance = u64;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1_000;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const SupersigPalletId: PalletId = PalletId(*b"id/susig");
	pub const SupersigPreimageByteDeposit: Balance = 1000;
}

impl pallet_supersig::Config for Test {
	type Call = Call;
	type Currency = Balances;
	type Event = Event;
	type PalletId = SupersigPalletId;
	type PreimageByteDeposit = SupersigPreimageByteDeposit;
	type WeightInfo = pallet_supersig::weights::SubstrateWeight<Test>;
}

pub type NoCall = nothing::Call<Test>;

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Helper function to generate a crypto pair from seeds
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Mock users AccountId
#[allow(non_snake_case)]
pub fn ALICE() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Alice")
}
#[allow(non_snake_case)]
pub fn BOB() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Bob")
}
#[allow(non_snake_case)]
pub fn CHARLIE() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Charlie")
}
pub struct ExtBuilder {
	caps_endowed_accounts: Vec<(AccountId, u64)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder {
			caps_endowed_accounts: vec![
				(ALICE(), 1_000_000),
				(BOB(), 100_000),
				(CHARLIE(), 100_000),
			],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, accounts: Vec<(AccountId, u64)>) -> Self {
		for account in accounts {
			self.caps_endowed_accounts.push(account);
		}
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: self.caps_endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
