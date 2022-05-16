use crate::{
    mock::*,
    Error,
    Supersig as SupersigStruct,
    PreimageCall,
};
use frame_support::{
    assert_noop, assert_ok,
	traits::{tokens::ExistenceRequirement, Currency},
};
// use sp_runtime::traits::BadOrigin;
use sp_runtime::traits::{AccountIdConversion, Saturating};
use codec::{Decode, Encode};
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
            members: vec!(ALICE(), BOB(), CHARLIE()),
            threshold: 2
        };

		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 2));

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
        assert_eq!(Supersig::nonce_supersig(), 1);
        assert_eq!(Supersig::supersigs(0).unwrap(), supersig);
	});
}

#[test]
fn create_multiple_supersig() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
        let supersig_1 = SupersigStruct {
            members: vec!(ALICE(), BOB(), CHARLIE()),
            threshold: 2
        };
        let supersig_2 = SupersigStruct {
            members: vec!(ALICE(), BOB()),
            threshold: 5
        };

        assert_eq!(Supersig::nonce_supersig(), 0);
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 2));
        assert_eq!(Supersig::nonce_supersig(), 1);
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB()), 5));
        assert_eq!(Supersig::nonce_supersig(), 2);

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
		assert_eq!(Balances::free_balance(get_account_id(1)), Balances::minimum_balance());
        Balances::transfer(
            Origin::signed(ALICE()),
            get_account_id(1),
            10_000,
        ).unwrap();

		assert_eq!(Balances::free_balance(get_account_id(0)), Balances::minimum_balance());
		assert_eq!(Balances::free_balance(get_account_id(1)), Balances::minimum_balance() + 10_000);
        assert_eq!(Supersig::supersigs(0).unwrap(), supersig_1);
        assert_eq!(Supersig::supersigs(1).unwrap(), supersig_2);
	});
}

#[test]
fn create_with_empty_list() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(), 2), Error::<Test>::InvalidSupersig);
	});
}

#[test]
fn create_with_0_threshold() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_noop!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 0), Error::<Test>::InvalidSupersig);
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
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB(), CHARLIE()), 2));
        let supersig_id = get_account_id(0);

        let call = Call::Nothing(NoCall::do_nothing {});

        // let data = call.encode();
        // let provider = ALICE();
		// let deposit = Balances::from(data.len() as u32).saturating_mul(Supersig::PreimageByteDeposit::get());
        // let preimage = PreimageCall {
        //     data,
        //     provider,
        //     deposit,
        // };


        assert_ok!(Supersig::submit_call(Origin::signed(ALICE()), supersig_id.clone(), Box::new(call.clone())));
        assert_eq!(Supersig::nonce_call(), 1);
        assert_ok!(Supersig::submit_call(Origin::signed(BOB()), supersig_id.clone(), Box::new(call.clone())));
        assert_eq!(Supersig::nonce_call(), 2);
        assert_ok!(Supersig::submit_call(Origin::signed(CHARLIE()), supersig_id, Box::new(call)));
        assert_eq!(Supersig::nonce_call(), 3);
        // assert_eq!(Supersig::calls(0).unwrap(), preimage);
    })
}

#[test]
fn not_a_member() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(Origin::signed(ALICE()), vec!(ALICE(), BOB()), 2));
        let supersig_id = get_account_id(0);

        let call = Call::Nothing(NoCall::do_nothing {});
        assert_noop!(Supersig::submit_call(Origin::signed(CHARLIE()), supersig_id, Box::new(call)), Error::<Test>::NotMember);
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
