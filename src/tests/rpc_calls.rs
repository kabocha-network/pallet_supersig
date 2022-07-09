use super::{helper::*, mock::*};
use crate::{Role};
use frame_support::{ assert_ok };
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn get_account_supersigs() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));

		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Master);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![0]);

		let members2 = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members2,));
		let supersig_account2 = get_supersig_account(1);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account2.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account2.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(1, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(1, BOB()), Role::Master);
		assert_eq!(Supersig::members(1, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(1), 3);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0]);

		let members3 = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(BOB()), members3,));
		let supersig_account3 = get_supersig_account(2);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account3.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account3.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(2, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(2, BOB()), Role::Master);
		assert_eq!(Supersig::members(2, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(2), 3);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0, 2]);
	})
}

#[test]
fn get_members_connected_to_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members,));

		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Master);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);
        assert_eq!(Supersig::get_members_connected_to_supersig(0), vec![(ALICE(), Role::Standard), (BOB(), Role::Master), (CHARLIE(), Role::Standard)]);

		let members2 = vec![(ALICE(), Role::Master), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), members2,));
		let supersig_account2 = get_supersig_account(1);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account2.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account2.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(1, ALICE()), Role::Master);
		assert_eq!(Supersig::members(1, BOB()), Role::Master);
		assert_eq!(Supersig::members(1, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(1), 3);
        assert_eq!(Supersig::get_members_connected_to_supersig(1), vec![(ALICE(), Role::Master), (BOB(), Role::Master), (CHARLIE(), Role::Standard)]);

		let members3 = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(Origin::signed(BOB()), members3,));
		let supersig_account3 = get_supersig_account(2);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_account3.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account3.clone()),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(2, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(2, BOB()), Role::Master);
		assert_eq!(Supersig::members(2, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(2), 3);
        assert_eq!(Supersig::get_members_connected_to_supersig(2), vec![(ALICE(), Role::Standard), (BOB(), Role::Master), (CHARLIE(), Role::Standard)]);
	})
}