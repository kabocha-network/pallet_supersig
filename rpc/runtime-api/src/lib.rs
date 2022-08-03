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
		fn get_supersig_id(supersig_account: AccountId) -> Option<SupersigId>;
		fn get_user_supersigs(who: AccountId) -> Vec<SupersigId>;
		fn list_members(supersig_id: SupersigId) -> Vec<(AccountId, Role)>;
		fn list_proposals(supersig_id: SupersigId) -> (Vec<ProposalState<AccountId>>, u32);
		fn get_proposal_state(supersig_id: SupersigId, call_id: CallId) -> Option<(ProposalState<AccountId>, u32)>;
	}
}
