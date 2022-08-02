use crate::Vec;
use codec::{Decode, Encode};
pub use sp_std::{boxed::Box, cmp::max, mem::size_of};

use crate::{CallId, Calls, Config, Error, Members, MembersVotes, Pallet, Role, SupersigId};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ProposalState<AccountId> {
	id: CallId,
	provider: AccountId,
	voters: Vec<AccountId>,
}

impl<AccoutId: Clone> ProposalState<AccoutId> {
	pub fn new(id: CallId, provider: AccoutId, voters: Vec<AccoutId>) -> Self {
		Self {
			id,
			provider,
			voters,
		}
	}

	pub fn id(&self) -> &CallId {
		&self.id
	}

	pub fn provider(&self) -> &AccoutId {
		&self.provider
	}

	pub fn voters(&self) -> &Vec<AccoutId> {
		&self.voters
	}
}

impl<T: Config> Pallet<T> {
	pub fn get_supersig_id(supersig_account: &T::AccountId) -> Result<SupersigId, Error<T>> {
		Self::get_supersig_id_from_account(supersig_account)
	}

	pub fn get_user_supersigs(who: &T::AccountId) -> Vec<SupersigId> {
		Members::<T>::iter()
			.filter_map(|(supersig_id, member_id, _)| {
				if member_id == *who {
					Some(supersig_id)
				} else {
					None
				}
			})
			.collect()
	}

	pub fn list_members(supersig_id: &SupersigId) -> Vec<(T::AccountId, Role)> {
		Members::<T>::iter_prefix(supersig_id).collect()
	}

	pub fn list_proposals(supersig_id: &SupersigId) -> (Vec<ProposalState<T::AccountId>>, u32) {
		let member_count = Self::total_members(supersig_id);
		let proposal_state = Calls::<T>::iter_prefix(supersig_id)
			.map(|(call_id, call)| {
				let voters = MembersVotes::<T>::iter_prefix((supersig_id, call_id))
					.filter_map(
						|(account_id, vote)| {
							if vote { Some(account_id) } else { None }
						},
					)
					.collect();

				ProposalState::new(call_id, call.provider, voters)
			})
			.collect();
		(proposal_state, member_count)
	}

	pub fn get_proposal_state(
		supersig_id: &SupersigId,
		call_id: &CallId,
	) -> Option<(ProposalState<T::AccountId>, u32)> {
		let call = Self::calls(supersig_id, call_id)?;
		let member_count = Self::total_members(supersig_id);
		let voters = MembersVotes::<T>::iter_prefix((supersig_id, call_id))
			.filter_map(
				|(account_id, vote)| {
					if vote { Some(account_id) } else { None }
				},
			)
			.collect();

		Some((
			ProposalState::new(*call_id, call.provider, voters),
			member_count,
		))
	}
}
