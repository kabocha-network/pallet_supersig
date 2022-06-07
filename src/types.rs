use crate::*;
use codec::{Decode, Encode};
use scale_info::TypeInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq, Eq)]
pub struct Supersig<AccountId> {
	pub members: Vec<AccountId>,
	master: Option<AccountId>,
}

impl<AccountId: std::cmp::PartialEq + Clone> Supersig<AccountId> {
	pub fn new(members: Vec<AccountId>, master: Option<AccountId>) -> Option<Self> {
		if members.is_empty() {
			return None
		}
		if let Some(master) = master.as_ref() {
			if !members.contains(master) {
				return None
			}
		}
		Some(Self { members, master })
	}

	pub fn master(&self) -> Option<AccountId> {
		self.master.clone()
	}

	pub fn can_remove_member(&self, member: &AccountId) -> bool {
		if let Some(master) = &self.master {
			master == member
		} else {
			true
		}
	}
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug)]
pub struct PreimageCall<AccountId, Balance> {
	pub data: Vec<u8>,
	pub provider: AccountId,
	pub deposit: Balance,
	// pub master_signed: bool
}

pub type SigIndex = u128;
pub type CallIndex = u128;
