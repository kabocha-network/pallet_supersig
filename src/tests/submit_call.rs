use super::{helper::*, mock::*};
use crate::Error;
use frame_support::{assert_noop, assert_ok};
pub use sp_std::boxed::Box;

#[test]
fn submit_calls() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			Origin::signed(ALICE()),
			vec!(ALICE(), BOB(), CHARLIE()),
		));
		let supersig_id = get_account_id(0);

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		let call1 = Call::Nothing(NoCall::do_nothing {
			nothing: "test1".into(),
		});
		let call2 = Call::Nothing(NoCall::do_nothing {
			nothing: "test2".into(),
		});

		assert_ok!(Supersig::submit_call(
			Origin::signed(ALICE()),
			supersig_id.clone(),
			Box::new(call)
		));
		assert_eq!(Supersig::nonce_call(0), 1);
		assert_ok!(Supersig::submit_call(
			Origin::signed(BOB()),
			supersig_id.clone(),
			Box::new(call1)
		));
		assert_eq!(Supersig::nonce_call(0), 2);
		assert_ok!(Supersig::submit_call(
			Origin::signed(CHARLIE()),
			supersig_id,
			Box::new(call2)
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

		let call = Call::Nothing(NoCall::do_nothing {
			nothing: "test".into(),
		});
		assert_noop!(
			Supersig::submit_call(Origin::signed(CHARLIE()), bad_supersig_id, Box::new(call)),
			Error::<Test>::SupersigNotFound
		);
	})
}
