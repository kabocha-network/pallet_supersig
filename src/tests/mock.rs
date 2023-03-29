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

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Supersig: pallet_supersig::{Pallet, Call, Storage, Event<T>},

		Balances: pallet_balances,
	}
);

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
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
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
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub const SupersigPalletId: PalletId = PalletId(*b"id/susig");
	pub const SupersigPreimageByteDeposit: Balance = 1000;
	pub const MaxAccountsPerTransaction: u32 = 4;
	pub const MaxCallDataSize: u32 = 1024;

}

impl pallet_supersig::Config for Test {
	type Call = RuntimeCall;
	type Currency = Balances;
	type DepositPerByte = SupersigPreimageByteDeposit;
	type MaxAccountsPerTransaction = MaxAccountsPerTransaction;
	type MaxCallDataSize = MaxCallDataSize;
	type PalletId = SupersigPalletId;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_supersig::weights::SubstrateWeight<Test>;
}

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
#[allow(non_snake_case)]
pub fn PAUL() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Paul")
}
#[allow(non_snake_case)]
pub fn DONALD() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Donald")
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
				(CHARLIE(), 101_000),
				(PAUL(), 100_000),
				(DONALD(), 100_000),
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
