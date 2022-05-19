use crate::{mock::*, Error, Supersig as SupersigStruct};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_runtime::traits::AccountIdConversion;
pub use sp_std::boxed::Box;

////////////
//
// create_supersig() tests
//
////////////

#[test]
fn create_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig = SupersigStruct {
			members: vec![ALICE(), BOB(), CHARLIE()],
			threshold: 2,
		};

		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));

		assert_eq!(
			Balances::free_balance(get_account_id(0)),
			Balances::minimum_balance()
		);
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
	});
}

#[test]
fn create_multiple_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		let supersig_1 = SupersigStruct {
			members: vec![ALICE(), BOB(), CHARLIE()],
			threshold: 2,
		};
		let supersig_2 = SupersigStruct {
			members: vec![ALICE(), BOB()],
			threshold: 2,
		};

		assert_eq!(Supersig::nonce_supersig(), 0);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));
		assert_eq!(Supersig::nonce_supersig(), 1);
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
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
			Supersig::create_supersig(Origin::signed(ALICE()), vec!(), 2),
			Error::<Test>::InvalidSupersig
		);
	});
}

#[test]
fn create_with_bad_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(
			Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 0),
			Error::<Test>::InvalidSupersig
		);
		assert_noop!(
			Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 5),
			Error::<Test>::InvalidSupersig
		);
	});
}

////////////
//
// submit_call() tests
//
////////////

#[test]
fn submit_calls() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});

		// let data = call.encode();
		// let provider = ALICE();
		// let deposit = Balances::from(data.len() as
		// u32).saturating_mul(Supersig::PreimageByteDeposit::get()); let preimage = PreimageCall {
		//     data,
		//     provider,
		//     deposit,
		// };

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call.clone())
		));
		assert_eq!(Supersig::nonce_call(0), 1);
		assert_ok!(Supersig::submit_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			Box::new(call.clone())
		));
		assert_eq!(Supersig::nonce_call(0), 2);
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_id,
			Box::new(call)
		));
		assert_eq!(Supersig::nonce_call(0), 3);
		// assert_eq!(Supersig::calls(0).unwrap(), preimage);
	})
}
#[test]
fn submit_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let bad_supersig_id = get_account_id(1);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_noop!(
			Supersig::submit_call(Origin::signed(CHARLIE()), bad_supersig_id, Box::new(call)),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn submit_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_noop!(
			Supersig::submit_call(Origin::signed(CHARLIE()), supersig_id, Box::new(call)),
			Error::<Test>::NotMember
		);
	})
}

////////////
//
// approve_call() tests
//
////////////

#[test]
fn approve_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			3
		));
		let supersig_id = get_account_id(0);
		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			0
		));
		assert_eq!(Supersig::votes(0, 0), 1);
		assert!(Supersig::users_votes((0, 0), ALICE()));

		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_id,
			0
		));
		assert_eq!(Supersig::votes(0, 0), 2);
		assert!(Supersig::users_votes((0, 0), CHARLIE()));
		assert!(!Supersig::users_votes((0, 0), BOB()));
	})
}

#[test]
fn approve_call_until_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));
		let supersig_id = get_account_id(0);
		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			0
		));
		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_id,
			0
		));

		assert_eq!(Supersig::votes(0, 0), 2);
		assert!(Supersig::users_votes((0, 0), ALICE()));
		assert!(Supersig::users_votes((0, 0), BOB()));
		assert!(!Supersig::users_votes((0, 0), CHARLIE()));

		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Balances::reserved_balance(ALICE()), 0);
	})
}

#[test]
fn approve_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_id,
			Box::new(call)
		));
		assert_noop!(
			Supersig::approve_call(Origin::signed(CHARLIE()), get_account_id(3), 0),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn user_already_voted() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_id.clone(),
			0
		));
		assert_noop!(
			Supersig::approve_call(Origin::signed(CHARLIE()), supersig_id, 0),
			Error::<Test>::AlreadyVoted
		);
	})
}

#[test]
fn approve_not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::approve_call(Origin::signed(CHARLIE()), supersig_id, 0),
			Error::<Test>::NotMember
		);
	})
}

////////////
//
// remove_call() tests
//
////////////

#[test]
fn remove_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert!(Supersig::calls(0, 0).is_some());
		assert_ok!(Supersig::remove_call(
			Origin::signed(supersig_id.clone()),
			supersig_id,
			0
		));
		assert!(Supersig::calls(0, 0).is_none());
	})
}

#[test]
fn non_allowed_remove_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(Origin::signed(BOB()), supersig_id, 0),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_unknown_call() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
			2
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});
		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_noop!(
			Supersig::remove_call(Origin::signed(supersig_id.clone()), supersig_id, 1),
			Error::<Test>::CallNotFound
		);
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
