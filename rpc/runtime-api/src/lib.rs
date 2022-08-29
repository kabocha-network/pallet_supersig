#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

use pallet_supersig::{rpc::ProposalState, CallId, Role, SupersigId};

sp_api::decl_runtime_apis! {
	pub trait SuperSigApi<AccountId>
	where
		AccountId: Codec,
	{
		fn get_user_supersigs(user_account: AccountId) -> Vec<SupersigId>;
		fn list_members(supersig_account: AccountId) -> Vec<(AccountId, Role)>;
		fn list_proposals(supersig_account: AccountId) -> (Vec<ProposalState<AccountId>>, u32);
		fn get_proposal_state(supersig_account: AccountId, call_id: CallId) -> (ProposalState<AccountId>, u32);
	}
}
