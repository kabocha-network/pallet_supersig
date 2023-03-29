use super::{helper::*, mock::*};
use crate::{Config as SuperConfig, Error, Role};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
pub use sp_std::boxed::Box;

#[test]
fn propose_calls() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);

		let call: RuntimeCall = frame_system::Call::remark {
			remark: "test".into(),
		}
		.into();
		let call1: RuntimeCall = frame_system::Call::remark {
			remark: "test1".into(),
		}
		.into();
		let call2: RuntimeCall = frame_system::Call::remark {
			remark: "test2".into(),
		}
		.into();

		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(ALICE()).into(),
			supersig_account.clone(),
			Box::new(call.clone())
		));
		let deposit = Balance::from(call.encode().len() as u32)
			.saturating_mul(<Test as SuperConfig>::DepositPerByte::get());
		assert_eq!(Balances::reserved_balance(ALICE()), deposit);
		assert_eq!(Supersig::nonce_call(0), 1);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::CallSubmitted(
				supersig_account.clone(),
				0,
				ALICE()
			))
		);
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(BOB()).into(),
			supersig_account.clone(),
			Box::new(call1)
		));
		assert_eq!(Supersig::nonce_call(0), 2);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::CallSubmitted(
				supersig_account.clone(),
				1,
				BOB()
			))
		);
		assert_ok!(Supersig::propose_call(
			RawOrigin::Signed(CHARLIE()).into(),
			supersig_account.clone(),
			Box::new(call2)
		));
		assert_eq!(Supersig::nonce_call(0), 3);
		assert_eq!(
			last_event(),
			RuntimeEvent::Supersig(crate::Event::CallSubmitted(supersig_account, 2, CHARLIE()))
		);
	})
}
#[test]
fn submit_supersig_doesnt_exist() {
	ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let bad_supersig_account = get_supersig_account(1);

		let call: RuntimeCall = frame_system::Call::remark {
			remark: "test".into(),
		}
		.into();
		assert_noop!(
			Supersig::propose_call(
				RawOrigin::Signed(CHARLIE()).into(),
				bad_supersig_account,
				Box::new(call)
			),
			Error::<Test>::NotSupersig
		);
	})
}
#[test]
fn propose_call_data_too_large() {
    ExtBuilder::default().balances(vec![]).build().execute_with(|| {
		assert_ok!(Supersig::create_supersig(
			RawOrigin::Signed(ALICE()).into(),
			vec! {
				(ALICE(), Role::Standard),
				(BOB(), Role::Standard),
				(CHARLIE(), Role::Standard),
			}
			.try_into()
			.unwrap()
		));
		let supersig_account = get_supersig_account(0);
        // Generate a call with data that exceeds the MaxCallDataSize
        let large_data: Vec<u8> = vec![0; 2000];
        let call: RuntimeCall = frame_system::Call::remark { remark: large_data.into() }.into();

        assert_noop!(
            Supersig::propose_call(
                RawOrigin::Signed(ALICE()).into(),
                supersig_account.clone(),
                Box::new(call)
            ),
            Error::<Test>::CallDataTooLarge
        );
    });
}
