use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok};
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn remove_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
				(PAUL(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		assert_ok!(Supersig::remove_members(
			Origin::signed(supersig_account.clone()),
			vec!(BOB(), CHARLIE(), CHARLIE()).try_into().unwrap()
		));
		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::NotMember);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::NotMember);
		assert_eq!(Supersig::members(0, PAUL()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 2);

		let reserve = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((Supersig::total_members(0) as u32).into())
			.saturating_mul(<Test as SuperConfig>::DepositPerByte::get());
		assert_eq!(Balances::reserved_balance(get_supersig_account(0)), reserve);
		assert_eq!(
			last_event(),
			Event::Supersig(crate::Event::MembersRemoved(
				supersig_account,
				vec!(BOB(), CHARLIE())
			))
		);
	})
}

#[test]
fn remove_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
				(PAUL(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(ALICE()),
				vec!(BOB(), CHARLIE()).try_into().unwrap()
			),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn remove_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
				(PAUL(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let bad_supersig_account = get_supersig_account(1);
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(bad_supersig_account),
				vec!(BOB(), CHARLIE()).try_into().unwrap()
			),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn remove_users_leaving_0_users() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
			}
			.try_into()
			.unwrap(),
		));
		let supersig_account = get_supersig_account(0);
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(supersig_account),
				vec!(ALICE(), BOB()).try_into().unwrap()
			),
			Error::<Test>::InvalidNumberOfMembers
		);
	})
}
