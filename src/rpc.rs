use crate::Vec;
use codec::{Decode, Encode};
use sp_runtime::DispatchError;
pub use sp_std::{boxed::Box, cmp::max, mem::size_of};

use crate::{CallId, Calls, Config, Error, Members, MembersVotes, Pallet, Role, SupersigId};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct ProposalState<AccountId> {
	id: CallId,
	encoded_call: Vec<u8>,
	provider: AccountId,
	voters: Vec<AccountId>,
}

impl<AccoutId: Clone> ProposalState<AccoutId> {
	pub fn new(
		id: CallId,
		encoded_call: Vec<u8>,
		provider: AccoutId,
		voters: Vec<AccoutId>,
	) -> Self {
		Self {
			id,
			encoded_call,
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
	pub fn get_user_supersigs(user_account: &T::AccountId) -> Vec<SupersigId> {
		Members::<T>::iter()
			.filter_map(|(supersig_id, member_id, _)| {
				if member_id == *user_account {
					Some(supersig_id)
				} else {
					None
				}
			})
			.collect()
	}

	pub fn list_members(
		supersig_account: &T::AccountId,
	) -> Result<Vec<(T::AccountId, Role)>, DispatchError> {
		let supersig_id = Self::get_supersig_id_from_account(supersig_account)?;
		Ok(Members::<T>::iter_prefix(supersig_id).collect())
	}

	pub fn list_proposals(
		supersig_account: &T::AccountId,
	) -> Result<(Vec<ProposalState<T::AccountId>>, u32), DispatchError> {
		let supersig_id = Self::get_supersig_id_from_account(supersig_account)?;
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

				ProposalState::new(call_id, call.data, call.provider, voters)
			})
			.collect();
		Ok((proposal_state, member_count))
	}

	pub fn get_proposal_state(
		supersig_account: &T::AccountId,
		call_id: &CallId,
	) -> Result<(ProposalState<T::AccountId>, u32), DispatchError> {
		let supersig_id = Self::get_supersig_id_from_account(supersig_account)?;
		let call = Self::calls(supersig_id, call_id).ok_or(Error::<T>::CallNotFound)?;
		let member_count = Self::total_members(supersig_id);
		let voters = MembersVotes::<T>::iter_prefix((supersig_id, call_id))
			.filter_map(
				|(account_id, vote)| {
					if vote { Some(account_id) } else { None }
				},
			)
			.collect();

		Ok((
			ProposalState::new(*call_id, call.data, call.provider, voters),
			member_count,
		))
	}
}
