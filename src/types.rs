use crate::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::MaxEncodedLen;
use scale_info::TypeInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, MaxEncodedLen, Debug)]
#[codec(mel_bound())]
pub enum Role {
	Standard,
	Master,
	NotMember,
}

impl Default for Role {
	fn default() -> Self {
		Role::NotMember
	}
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug)]
pub struct PreimageCall<AccountId, Balance> {
	pub data: Vec<u8>,
	pub provider: AccountId,
	pub deposit: Balance,
}

pub type SupersigId = u128;
pub type CallId = u128;
