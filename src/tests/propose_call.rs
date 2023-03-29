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

// Test that the max live proposal limit is working
#[test]
fn test_live_proposals_limit() {
	ExtBuilder::default()
		.balances(vec![])
		.build()
		.execute_with(|| {
			// create supersig account with Alice
			assert_ok!(Supersig::create_supersig(
				RawOrigin::Signed(ALICE()).into(),
				vec! {
					(ALICE(), Role::Standard),
					(CHARLIE(), Role::Standard),
				}
				.try_into()
				.unwrap()
			));
			let supersig_account = get_supersig_account(0);
			

			// create 3 proposals if the max is 3
			for i in 1..3 {
				// let call = pallet_balances::Call::transfer {
				// 	dest: ALICE(),
				// 	value: 1000,
				// };

				let call: RuntimeCall = frame_system::Call::remark {
					remark: "test".into(),
				}
				.into();
				Supersig::propose_call(
					RawOrigin::Signed(ALICE()).into(),
					supersig_account.clone(),
					Box::new(call.clone())
				);
				assert_ok!(
					Supersig::propose_call(
						RawOrigin::Signed(ALICE()).into(),
						supersig_account.clone(),
						Box::new(call.clone()),
				));
				// 
				
				// vote for the proposal from Alice
				assert_ok!(Supersig::approve_call(
					RawOrigin::Signed(ALICE()).into(),
					supersig_account.clone(),
					(i as u64).into(),
				));
			};
			// should not be able to create a sixth proposal
			let call: RuntimeCall = frame_system::Call::remark {
				remark: "test".into(),
			}	
			.into();
			assert_noop!(
				Supersig::propose_call(
					RawOrigin::Signed(ALICE()).into(),
					supersig_account.clone(),
					Box::new(call.clone())
			),
				Error::<Test>::TooManyActiveProposals
			);
		})
}

// Test that active proposals are decremented when proposal is approved
#[test]
fn test_active_proposals_decrement_on_approve() {
	ExtBuilder::default()
		.balances(vec![])
		.build()
		.execute_with(|| {
			// create supersig account with BOB
			assert_ok!(Supersig::create_supersig(
				RawOrigin::Signed(BOB()).into(),
				vec! {
					
					(BOB(), Role::Standard),
					
				}
				.try_into()
				.unwrap()
			));
			let supersig_account = get_supersig_account(0);
			let supersig_id = Supersig::get_supersig_id_from_account(&supersig_account).unwrap();
			// create 1 proposal
			let call: RuntimeCall = frame_system::Call::remark {
				remark: "test".into(),
			}	
			.into();
			assert_ok!(Supersig::propose_call(
				RawOrigin::Signed(BOB()).into(),
				supersig_account.clone(),
				Box::new(call)
			));

			// vote for the proposal from Alice
			assert_ok!(Supersig::approve_call(
				RawOrigin::Signed(BOB()).into(),
				supersig_account.clone(),
				0,
			));

			// should be zero active proposals
			assert_eq!(Supersig::active_proposals(supersig_id), 0);
		});
}

#[test]
	fn test_propose_call_max_active_proposals() {
		ExtBuilder::default()
		.balances(vec![])
		.build()
		.execute_with(|| {
			// create supersig account with BOB
			assert_ok!(Supersig::create_supersig(
				RawOrigin::Signed(ALICE()).into(),
				vec! {
					(ALICE(), Role::Standard),
				}
				.try_into()
				.unwrap()
			));
			let supersig_account = get_supersig_account(0);
			let call: RuntimeCall = frame_system::Call::remark {
						remark: "test".into(),
			}
			.into();
			// propose 3 calls
			for i in 1..3 {
					Supersig::propose_call(
						RawOrigin::Signed(ALICE()).into(), 
						supersig_account.clone(), 
						Box::new(call.clone()),
					);
			};

				// propose a 4th call once one of the active ones is approved
				assert_ok!(Supersig::approve_call(
					RawOrigin::Signed(ALICE()).into(), 
					supersig_account.clone(), 
					0
				));
				assert_ok!(Supersig::propose_call(
					RawOrigin::Signed(ALICE()).into(), 
					supersig_account.clone(), 
					Box::new(call.clone()),
				));
			});
	}

	#[test]
	fn test_propose_call_active_proposals_multiple_accounts() {
		ExtBuilder::default()
			.balances(vec![])
			.build()
			.execute_with(|| {
				// create supersig account with ALICE
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
				let supersig_account = get_supersig_account(0);
				let supersig_account_1 = get_supersig_account(1);
				let supersig_account_2 = get_supersig_account(2);
				let supersig_account_3 = get_supersig_account(3);
				let supersig_account_4 = get_supersig_account(4);

				let call: RuntimeCall = frame_system::Call::remark {
							remark: "test".into(),
				}
				.into();

				// propose a call with each account
				assert_ok!(Supersig::propose_call(
					RawOrigin::Signed(ALICE()).into(), 
					supersig_account.clone(), 
					Box::new(call.clone()),
					));
				assert_ok!(Supersig::propose_call(
					RawOrigin::Signed(BOB()).into(), 
					supersig_account_1.clone(), 
					Box::new(call.clone()),
				));
				assert_ok!(Supersig::propose_call(
					RawOrigin::Signed(BOB()).into(), 
					supersig_account_2.clone(), 
					Box::new(call.clone()),
				));
				assert_ok!(Supersig::propose_call(
					RawOrigin::Signed(BOB()).into(), 
					supersig_account_3.clone(), 
					Box::new(call.clone()),
				));

				// try to propose a fifth call just to show that MaxCallPerAccount is per account not the total chain.
				assert_ok!(
					Supersig::propose_call(
						RawOrigin::Signed(ALICE()).into(), 
						supersig_account_4.clone(), 
						Box::new(call.clone()),
					)
				);
		}
	);
}
#[test]
	fn test_remove_call_active_proposals() {
		ExtBuilder::default()
			.balances(vec![])
			.build()
			.execute_with(|| {
				// create supersig account with ALICE
				assert_ok!(Supersig::create_supersig(
					RawOrigin::Signed(ALICE()).into(),
					vec! {
						(ALICE(), Role::Standard),
						(BOB(), Role::Standard),
					}
					.try_into()
					.unwrap()
				));
			

				let supersig_account = get_supersig_account(0);
				let supersig_id = Supersig::get_supersig_id_from_account(&supersig_account).unwrap();
				let call: RuntimeCall = frame_system::Call::remark {
					remark: "test".into(),
		}
		.into();

		// Test that we cannot propose more than `MaxCallsPerAccount`
		for i in 1..=3 {
			Supersig::propose_call(
				RawOrigin::Signed(ALICE()).into(), 
				supersig_account.clone(), 
				Box::new(call.clone()),
			);
			// assert_noop!(
			// 	Supersig::propose_call(
			// 		RawOrigin::Signed(ALICE()).into(),
			// 		supersig_account.clone(), 
			// 		Box::new(call.clone()),
			// 	),
			// 	Error::<Test>::TooManyActiveProposals
			// );
		}
		
		

			// Test that we can propose more after we remove one
			assert_ok!(Supersig::remove_call(
				RawOrigin::Signed(ALICE()).into(), 
				supersig_account.clone(), 
				0
			));
			assert_eq!(Supersig::active_proposals(
				supersig_id.clone(),
			),
				2
			);

			assert_ok!(Supersig::propose_call(
				RawOrigin::Signed(ALICE()).into(), 
					supersig_account.clone(), 
					Box::new(call.clone()),
			));
			assert_eq!(Supersig::active_proposals(
				supersig_id.clone(),
				),
				3
			);

			// Test that we cannot remove more than `MaxCallsPerAccount` calls
			for i in 1..=3 {
				assert_ok!(Supersig::remove_call(
					RawOrigin::Signed(ALICE()).into(), 
					supersig_account.clone(), 
					i,
				));
				assert_noop!(
					Supersig::remove_call(
						RawOrigin::Signed(ALICE()).into(), 
						supersig_account.clone(), 
						(i as u32 - 1).into(),
					),
					Error::<Test>::CallNotFound,
				);
			}
			assert_eq!(
				Supersig::active_proposals(
					supersig_id.clone()
				), 
			0
			);
		}
	);
	}