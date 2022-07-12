#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
#[cfg(not(feature = "std"))]
use sp_std::prelude::Vec;

sp_api::decl_runtime_apis! {
	pub trait SuperSigApi<AccountId>
	where
		AccountId: Codec,
	{
		fn get_account_supersigs(origin: AccountId) -> Vec<u128>;
	}
}
