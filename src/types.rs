use crate::*;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::pallet_prelude::MaxEncodedLen;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Debug))]
#[codec(mel_bound())]
pub enum Roles {
    Member,
    Master,
    NotMember
}

impl Default for Roles {
    fn default() -> Self {
        Roles::NotMember
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
