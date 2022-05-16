use crate::{
    mock::*,
    Error,
    Supersig as SupersigStruct,
};
use frame_support::{
    assert_noop, assert_ok,
	traits::{tokens::ExistenceRequirement, Currency},
};
// use sp_runtime::traits::BadOrigin;
use sp_runtime::traits::AccountIdConversion;

#[test]
fn create_supersig_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let supersig = SupersigStruct {
            members: vec!(ALICE, BOB, CHARLIE),
            threshold: 2
        };

		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
        assert_eq!(Supersig::nonce_supersig(), 1);
        assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
	});
}

#[test]
fn create_multiple_supersig_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let supersig_1 = SupersigStruct {
            members: vec!(ALICE, BOB, CHARLIE),
            threshold: 2
        };
        let supersig_2 = SupersigStruct {
            members: vec!(ALICE, BOB),
            threshold: 5
        };

        assert_eq!(Supersig::nonce_supersig(), 0);
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 2));
        // assert_eq!(Supersig::nonce_supersig(), 1);
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB), 5));
        // assert_eq!(Supersig::nonce_supersig(), 2);

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
        assert_eq!(Supersig::nonce_supersig(), 2);
        assert_eq!(Supersig::supersigs(0).unwrap(), supersig_1);
        assert_eq!(Supersig::supersigs(1).unwrap(), supersig_2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Supersig::create_supersig(Origin::signed(ALICE), vec!(), 2), Error::<Test>::InvalidSupersig);
	});
}

#[test]
fn create_with_0_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Supersig::create_supersig(Origin::signed(ALICE), vec!(ALICE, BOB, CHARLIE), 0), Error::<Test>::InvalidSupersig);
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
    SupersigPalletId::get().into_sub_account(index)
}
