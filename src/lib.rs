#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use frame_support::{
	dispatch::DispatchResult,
	traits::{tokens::ExistenceRequirement, Currency},
	PalletId,
};

pub use sp_core::Hasher;

pub use codec::{Decode, Encode};
pub use sp_runtime::traits::AccountIdConversion;
pub use sp_std::prelude::Vec;

use scale_info::TypeInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq, Eq)]
pub struct Dorg<AccountId> {
	members: Vec<AccountId>,
	threshold: u64,
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug)]
pub struct CallHash {
	call_hash: (),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// the obiquitous event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The trait to manage funds
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce_dorg)]
	pub type NonceDorg<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn dorgs)]
	pub type Dorgs<T: Config> = StorageMap<_, Blake2_256, u128, Dorg<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageDoubleMap<_, Blake2_256, Dorg<T::AccountId>, Blake2_256, CallHash, u32>;

	#[pallet::storage]
	#[pallet::getter(fn users_votes)]
	pub type UsersVotes<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		(Dorg<T::AccountId>, CallHash),
		Blake2_256,
		T::AccountId,
		bool,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Dorg has been created [dorg]
		DorgCreated(T::AccountId),
		/// a Call has been submited [dorg, call_hash, submiter]
		CallSubmitted(T::AccountId, (), T::AccountId),
		/// a Call has been voted [dorg, call_hash, voter]
		CallVoted(T::AccountId, (), T::AccountId),
		/// a Call has been executed [dorg, call_hash]
		CallExecuted(T::AccountId, ()),
		/// a Call has been removed [dorg, call_hash]
		CallRemoved(T::AccountId, ()),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// dorg either have no member or have an invalid treshold (0)
		InvalidDorg,
		/// the dorg doesn't exist
		DorgNotFound,
		/// the call already exists
		CallAlreadyExists,
		/// the call doesn't exist
		CallNotFound,
		/// the user is not a member of the dorg
		NotMember,
		/// the user already voted for the call
		AlreadyVoted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_dorg(
			origin: OriginFor<T>,
			members: Vec<T::AccountId>,
			threshold: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if members.is_empty() || threshold == 0 {
				return Err(Error::<T>::InvalidDorg.into())
			};
			let index = Self::nonce_dorg();
			let dorg_id: T::AccountId = T::PalletId::get().into_sub_account(index);

			let minimum_balance = T::Currency::minimum_balance();
			T::Currency::transfer(
				&who,
				&dorg_id,
				minimum_balance,
				ExistenceRequirement::AllowDeath,
			)?;

			let dorg = Dorg { members, threshold };

			Dorgs::<T>::insert(index, dorg);
			NonceDorg::<T>::put(index + 1);

			Self::deposit_event(Event::<T>::DorgCreated(dorg_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_index_from_id(id: &T::AccountId) -> Option<u64> {
			PalletId::try_from_sub_account(id).map(|(_, index)| index)
		}
	}
}
