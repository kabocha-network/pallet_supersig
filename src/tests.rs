use crate::{
    mock::*,
    Error,
    Dorg as DorgStruct,
};
use frame_support::{
    assert_noop, assert_ok,
	traits::{tokens::ExistenceRequirement, Currency},
};
// use sp_runtime::traits::BadOrigin;
use sp_runtime::traits::AccountIdConversion;

#[test]
fn create_dorg_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let dorg = DorgStruct {
            members: vec!(ALICE, BOB, CHARLIE),
            threshold: 2
        };

		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
        assert_eq!(Dorg::nonce_dorg(), 1);
        assert_eq!(Dorg::dorgs(0).unwrap(), dorg);
	});
}

#[test]
fn create_multiple_dorg_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let dorg_1 = DorgStruct {
            members: vec!(ALICE, BOB, CHARLIE),
            threshold: 2
        };
        let dorg_2 = DorgStruct {
            members: vec!(ALICE, BOB),
            threshold: 5
        };

        assert_eq!(Dorg::nonce_dorg(), 0);
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
        // assert_eq!(Dorg::nonce_dorg(), 1);
		assert_ok!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB), 5));
        // assert_eq!(Dorg::nonce_dorg(), 2);

        assert_eq!(get_account_id(0), get_account_id(10000));
		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
		assert_eq!(Balances::free_balance(get_account_id(1)), Balances::minimum_balance());
        Balances::transfer(
            Origin::signed(ALICE),
            get_account_id(1),
            10_000,
        ).unwrap();

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
		assert_eq!(Balances::free_balance(get_account_id(1)), Balances::minimum_balance() + 10_000);
        assert_eq!(Dorg::nonce_dorg(), 2);
        assert_eq!(Dorg::dorgs(0).unwrap(), dorg_1);
        assert_eq!(Dorg::dorgs(1).unwrap(), dorg_2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Dorg::create_dorg(Origin::signed(ALICE), vec!(), 2), Error::<Test>::InvalidDorg);
	});
}

#[test]
fn create_with_0_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Dorg::create_dorg(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 0), Error::<Test>::InvalidDorg);
	});
}

#[test]
fn submit_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let call = Call::Nothing(NoCall::do_nothing {});
    })
}

////////////
//
// helper functions
//
////////////

fn get_account_id(index: u64) -> <Test as frame_system::Config>::AccountId {
    DorgPalletId::get().into_sub_account(index)
}
