use crate::{Error, Supersig as SupersigStruct};
use super::mock::*;
use super::helper::*;
use frame_support::{
	assert_noop, assert_ok,
	traits::Currency
};
pub use sp_std::boxed::Box;

#[test]
fn create_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig = SupersigStruct {
			members: vec![ALICE(), BOB(), CHARLIE()],
		};

		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE())
		));

		assert_eq!(
			Balances::free_balance(get_account_id(0)),
			Balances::minimum_balance()
		);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&get_account_id(0)),
			1
		);
	});
}

#[test]
fn create_multiple_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_1 = SupersigStruct {
			members: vec![ALICE(), BOB(), CHARLIE()],
		};
		let supersig_2 = SupersigStruct {
			members: vec![ALICE(), BOB()],
		};

		assert_eq!(Supersig::nonce_supersig(), 0);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		assert_eq!(Supersig::nonce_supersig(), 2);

		assert_eq!(
			Balances::free_balance(get_account_id(0)),
			Balances::minimum_balance()
		);
		assert_eq!(
			Balances::free_balance(get_account_id(1)),
			Balances::minimum_balance()
		);
		Balances::transfer(Origin::signed(ALICE()), get_account_id(1), 10_000).unwrap();

		assert_eq!(
			Balances::free_balance(get_account_id(0)),
			Balances::minimum_balance()
		);
		assert_eq!(
			Balances::free_balance(get_account_id(1)),
			Balances::minimum_balance() + 10_000
		);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig_1);
		assert_eq!(Supersig::supersigs(1).unwrap(), supersig_2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(Origin::signed(ALICE()), vec!()),
			Error::<Test>::InvalidSupersig
		);
	});
}

