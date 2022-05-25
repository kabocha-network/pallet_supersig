use crate::{mock::*, Error, Supersig as SupersigStruct};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, ReservableCurrency},
};
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
			frame_system::Pallet::<Test>::consumers(&get_account_id(0)),
			supersig.members.len() as u32
		);
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
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {});

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
	})
}
#[test]
fn submit_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
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
		assert!(Supersig::users_votes((0, 0, ALICE())));

		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_id,
			0
		));
		assert_eq!(Supersig::votes(0, 0), 2);
		assert!(Supersig::users_votes((0, 0, CHARLIE())));
		assert!(!Supersig::users_votes((0, 0, BOB())));
	})
}

#[test]
fn approve_call_until_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			100_000
		));

		let bob_balance = Balances::free_balance(BOB());

		let call = Call::Balances(pallet_balances::Call::transfer {
			dest: BOB(),
			value: 100_000,
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			0
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			0
		));

		assert_eq!(Supersig::votes(0, 0), 2);
		assert!(Supersig::users_votes((0, 0, ALICE())));
		assert!(Supersig::users_votes((0, 0, BOB())));
		assert!(!Supersig::users_votes((0, 0, CHARLIE())));

		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Balances::reserved_balance(ALICE()), 0);

		assert_eq!(bob_balance + 100_000, Balances::free_balance(BOB()));
	})
}

#[test]
fn approve_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
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
// add_members test
//
////////////

#[test]
fn add_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_ok!(Supersig::add_members(
			Origin::signed(supersig_id.clone()),
			supersig_id,
			vec!(BOB(), CHARLIE())
		));
		let supersig = Supersig::supersigs(0).unwrap();
		assert_eq!(supersig.members, vec!(ALICE(), BOB(), CHARLIE()));
		assert_eq!(
			frame_system::Pallet::<Test>::consumers(&get_account_id(0)),
			supersig.members.len() as u32
		);
	})
}

#[test]
fn add_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::add_members(Origin::signed(ALICE()), supersig_id, vec!(BOB(), CHARLIE())),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn add_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::add_members(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				vec!(BOB(), CHARLIE())
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

////////////
//
// remove_members test
//
////////////

#[test]
fn remove_members() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);
		assert_ok!(Supersig::remove_members(
			Origin::signed(supersig_id.clone()),
			supersig_id,
			vec!(BOB(), CHARLIE())
		));
		let supersig = Supersig::supersigs(0).unwrap();
		assert_eq!(supersig.members, vec!(ALICE()));
		assert_eq!(
			frame_system::Pallet::<Test>::consumers(&get_account_id(0)),
			supersig.members.len() as u32
		);
	})
}

#[test]
fn remove_users_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::remove_members(Origin::signed(ALICE()), supersig_id, vec!(BOB(), CHARLIE())),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_users_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::remove_members(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				vec!(BOB(), CHARLIE())
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

////////////
//
// remove_supersig test
//
////////////

#[test]
fn remove_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);
		let bob_balance = Balances::free_balance(BOB());
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			amount
		));
		assert_eq!(frame_system::Pallet::<Test>::consumers(&supersig_id), 3);
		assert_ok!(Supersig::remove_supersig(
			Origin::signed(supersig_id.clone()),
			supersig_id.clone(),
			BOB()
		));

		assert_eq!(Supersig::supersigs(0), None);
		assert_eq!(Supersig::nonce_call(0), 0);
		assert!(Supersig::calls(0, 0).is_none());
		assert_eq!(Supersig::votes(0, 0), 0);
		assert_eq!(frame_system::Pallet::<Test>::consumers(&supersig_id), 0);
		assert_eq!(frame_system::Pallet::<Test>::providers(&supersig_id), 0);
		assert_eq!(
			Balances::free_balance(BOB()),
			bob_balance + amount + Balances::minimum_balance()
		);
	})
}

#[test]
fn remove_supersig_not_allowed() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let supersig_id = get_account_id(0);
		assert_noop!(
			Supersig::remove_supersig(Origin::signed(ALICE()), supersig_id, BOB()),
			Error::<Test>::NotAllowed
		);
	})
}

#[test]
fn remove_supersig_unknown_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB()),
		));
		let bad_supersig_id = get_account_id(1);
		assert_noop!(
			Supersig::remove_supersig(
				Origin::signed(bad_supersig_id.clone()),
				bad_supersig_id,
				BOB()
			),
			Error::<Test>::SupersigNotFound
		);
	})
}

#[test]
fn cannot_remove_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);
		let amount = 10_000u64;
		assert_ok!(Balances::transfer(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			amount
		));
		assert_ok!(Balances::reserve(&supersig_id, amount));
		assert_noop!(
			Supersig::remove_supersig(Origin::signed(supersig_id.clone()), supersig_id, BOB()),
			Error::<Test>::CannotDeleteSupersig
		);
	})
}

#[test]
fn cannot_liquidate_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);

		let call = Call::Balances(pallet_balances::Call::transfer_all {
			dest: ALICE(),
			keep_alive: false,
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call.clone())
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			0
		));

		assert_ok!(Supersig::approve_call(
			Origin::signed(CHARLIE()),
			supersig_id.clone(),
			0
		));

		assert!(Supersig::calls(0, 0).is_none());

		assert!(System::account_exists(&supersig_id));
	});
}

////////////
//
// helper functions
//
////////////

fn get_account_id(index: u64) -> <Test as frame_system::Config>::AccountId {
	SupersigPalletId::get().into_sub_account(index)
}
