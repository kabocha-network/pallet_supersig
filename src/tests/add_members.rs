use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;
pub use sp_std::{boxed::Box, mem::size_of};

#[test]
fn add_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members = vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			members,
		));

		let supersig_account = get_supersig_account(0);
		assert_ok!(Balances::transfer(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			100_000
		));
		assert_ok!(Supersig::add_members(
			RawOrigin::Signed(supersig_account.clone()).into(),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));

		assert_eq!(Supersig::members(0, ALICE()), Role::Standard);
		assert_eq!(Supersig::members(0, BOB()), Role::Master);
		assert_eq!(Supersig::members(0, CHARLIE()), Role::Standard);
		assert_eq!(Supersig::total_members(0), 3);

		let deposit = Balance::from(size_of::<<Test as frame_system::Config>::AccountId>() as u32)
			.saturating_mul((Supersig::total_members(0) as u32).into())
			.saturating_mul(<Test as SuperConfig>::DepositPerByte::get());
		assert_eq!(Balances::reserved_balance(get_supersig_account(0)), deposit);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::MembersAdded(
				supersig_account,
				vec!((CHARLIE(), Role::Standard))
			))
		);
	})
}

#[test]
fn add_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members: BoundedVec<_, _> =
			vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			members.clone()
		));
		assert_noop!(
			Supersig::add_members(RawOrigin::Signed(ALICE()).into(), members),
			Error::<Test>::NotSupersig
		);
	})
}

#[test]
fn add_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let members: BoundedVec<_, _> =
			vec![(ALICE(), Role::Standard), (BOB(), Role::Standard)].try_into().unwrap();
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			members.clone()
		));
		let bad_supersig_account = get_supersig_account(1);
		assert_noop!(
			Supersig::add_members(RawOrigin::Signed(bad_supersig_account).into(), members),
			Error::<Test>::NotSupersig
		);
	})
}
