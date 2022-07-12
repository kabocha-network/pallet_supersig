use super::{helper::*, mock::*};
use crate::{Role};
use frame_support::{ assert_ok };
pub use sp_std::{boxed::Box, mem::size_of};

fn create_supersig(supersig_id : u128) -> sp_runtime::AccountId32 {
	let creator = vec![(ALICE(), Role::Master)].try_into().unwrap();
	assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), creator,));
	let supersig_account = get_supersig_account(u64::try_from(supersig_id).unwrap());
	assert_ok!(Balances::transfer(
		Origin::signed(ALICE()),
		supersig_account.clone(),
		100_000
	));
	assert_ok!(Supersig::add_members(
		Origin::signed(supersig_account.clone()),
		vec!((BOB(), Role::Standard), (CHARLIE(), Role::Standard)).try_into().unwrap()
	));
	assert_eq!(Supersig::members(supersig_id, ALICE()), Role::Master);
	assert_eq!(Supersig::members(supersig_id, BOB()), Role::Standard);
	assert_eq!(Supersig::members(supersig_id, CHARLIE()), Role::Standard);
	assert_eq!(Supersig::total_members(supersig_id), 3);
	supersig_account
}

#[test]
fn get_account_supersigs() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let mut supersig_count : u128 = 0;

        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![]);

		create_supersig(supersig_count);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![0]);
		supersig_count += 1;

		create_supersig(supersig_count);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0]);
		supersig_count += 1;

		create_supersig(supersig_count);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0, 2]);
		supersig_count += 1;

		create_supersig(supersig_count);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0, 3, 2]);
		supersig_count += 1;

		create_supersig(supersig_count);
        assert_eq!(Supersig::get_account_supersigs(ALICE()), vec![1, 0, 3, 4, 2]);
	})
}

#[test]
fn get_members_connected() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let mut supersig_count : u128 = 0;
		let mut supersig_account : sp_runtime::AccountId32;

		create_supersig(supersig_count);
        assert_eq!(
			Supersig::get_members_connected(supersig_count),
			vec![(ALICE(), Role::Master), (BOB(), Role::Standard), (CHARLIE(), Role::Standard)]
		);
		supersig_count += 1;

		supersig_account = create_supersig(supersig_count);
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account),
			vec!((BOB(), Role::Master), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));
		assert_eq!(Supersig::total_members(supersig_count), 3);
        assert_eq!(
			Supersig::get_members_connected(supersig_count),
			vec![(ALICE(), Role::Master), (BOB(), Role::Master), (CHARLIE(), Role::Standard)]
		);
		supersig_count += 1;

		supersig_account = create_supersig(supersig_count);
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_account),
			vec!((ALICE(), Role::Standard), (CHARLIE(), Role::Standard)).try_into().unwrap()
		));
        assert_eq!(
			Supersig::get_members_connected(2),
			vec![(ALICE(), Role::Standard), (BOB(), Role::Standard), (CHARLIE(), Role::Standard)]
		);
	})
}

#[test]
fn get_proposals() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let mut call_id = 0;
		let supersig_id = 0;
		let call = Call::Nothing(
			NoCall::do_nothing {
				nothing: "test".into()
			});
		let data = vec![2, 0, 16, 116, 101, 115, 116];
		let supersig_account = create_supersig(supersig_id);

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call.clone())
		));
		assert_eq!(
			Supersig::get_proposals(supersig_id),
			(vec![((data.clone(), ALICE(), 7000), vec![])], 3)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_account.clone(),
			call_id
		));
		let first_proposal = ((data.clone(), ALICE(), 7000), vec![BOB()]);
		assert_eq!(
			Supersig::get_proposals(supersig_id),
			(vec![first_proposal.clone()], 3)
		);
		call_id += 1;

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call.clone())
		));
		assert_eq!(
			Supersig::get_proposals(supersig_id),
			(vec![((data.clone(), ALICE(), 7000), vec![]), first_proposal.clone()], 3)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_account.clone(),
			call_id
		));
		let second_proposal = ((data.clone(), ALICE(), 7000), vec![BOB()]);

		assert_eq!(
			Supersig::get_proposals(supersig_id),
			(vec![second_proposal.clone(), first_proposal.clone()], 3)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			call_id
		));
		assert_eq!(
			Supersig::get_proposals(0),
			(vec![first_proposal.clone()], 3)
		);
	})
}

#[test]
fn get_proposal_state() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let mut call_id = 0;
		let supersig_id = 0;
		let call = Call::Nothing(
			NoCall::do_nothing {
				nothing: "test".into()
			});
		let supersig_account = create_supersig(supersig_id);

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call.clone())
		));
		assert_eq!(
			Supersig::get_proposal_state(supersig_id, call_id),
			(true, vec![], 3, 0)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_account.clone(),
			call_id
		));
		assert_eq!(
			Supersig::get_proposal_state(supersig_id, call_id),
			(true, vec![BOB()], 3, 1)
		);
		call_id += 1;

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			Box::new(call.clone())
		));
		assert_eq!(
			Supersig::get_proposal_state(supersig_id, call_id),
			(true, vec![], 3, 0)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_account.clone(),
			call_id
		));
		assert_eq!(
			Supersig::get_proposal_state(supersig_id, call_id),
			(true, vec![BOB()], 3, 1)
		);
		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_account.clone(),
			call_id
		));
		assert_eq!(
			Supersig::get_proposal_state(supersig_id, call_id),
			(false, vec![], 3, 0)
		);
	})
}