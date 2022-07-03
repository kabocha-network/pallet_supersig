#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait SuperSigApi<AccountId>
	where
		AccountId: Codec,
	{
		fn get_account_supersigs(origin: AccountId) -> Vec<u128>;
	}
}
