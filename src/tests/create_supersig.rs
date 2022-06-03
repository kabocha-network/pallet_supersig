use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Supersig as SupersigStruct};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, cmp::min, mem::size_of};

#[test]
fn create_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig = SupersigStruct::new(vec![ALICE(), BOB(), CHARLIE()], None).unwrap();

		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			None
		));

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());

		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&get_account_id(0)),
			1
		);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigCreated(get_account_id(0)))
		);
	});
}

#[test]
fn create_supersig_with_master() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig = SupersigStruct::new(vec![ALICE(), BOB(), CHARLIE()], Some(ALICE())).unwrap();

		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			Some(ALICE())
		));

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((3u32).into())
			.saturating_mul(<Test as SuperConfig>::PricePerBytes::get());

		assert_eq!(Balances::reserved_balance(get_account_id(0)), deposit);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
		assert_eq!(
			frame_system::Pallet::<Test>::providers(&get_account_id(0)),
			1
		);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::SupersigCreated(get_account_id(0)))
		);
	});
}

#[test]
fn create_supersig_with_master_not_included() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(
				Origin::signed(ALICE()),
				vec!(ALICE(), BOB(), CHARLIE()),
				Some(PAUL())
			),
			Error::<Test>::InvalidSupersig
		);
	});
}

#[test]
fn create_multiple_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_1 = SupersigStruct::new(vec![ALICE(), BOB(), CHARLIE()], None).unwrap();
		let supersig_2 = SupersigStruct::new(vec![ALICE(), BOB()], None).unwrap();

		assert_eq!(Supersig::nonce_supersig(), 0);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			None
		));
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			None
		));
		assert_eq!(Supersig::nonce_supersig(), 2);

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		assert_eq!(Balances::free_balance(get_account_id(1)), 0u64);
		Balances::transfer(Origin::signed(ALICE()), get_account_id(1), 10_000).unwrap();

		assert_eq!(Balances::free_balance(get_account_id(0)), 0u64);
		assert_eq!(Balances::free_balance(get_account_id(1)), 10_000);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig_1);
		assert_eq!(Supersig::supersigs(1).unwrap(), supersig_2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(Origin::signed(ALICE()), vec!(), None),
			Error::<Test>::InvalidSupersig
		);
	});
}
