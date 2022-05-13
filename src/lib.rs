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
	traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
	PalletId,
};

pub use sp_core::Hasher;

pub use codec::{Decode, Encode};
pub use sp_runtime::traits::{AccountIdConversion, Dispatchable, Hash, Saturating};
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
pub struct PreimageCall<AccountId, Balance> {
	data: Vec<u8>,
	provider: AccountId,
	deposit: Balance,
}

pub type DorgIndex = u128;
pub type CallIndex = u128;

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
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The base id used for accountId calculation
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The call type
		type Call: Parameter + Dispatchable<Origin = Self::Origin> + From<Call<Self>>;
		/// The amount of balance that must be deposited per byte of preimage stored.
		#[pallet::constant]
		type PreimageByteDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce_dorg)]
	pub type NonceDorg<T: Config> = StorageValue<_, DorgIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn dorgs)]
	pub type Dorgs<T: Config> =
		StorageMap<_, Blake2_256, DorgIndex, Dorg<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_call)]
	pub type NonceCall<T: Config> = StorageValue<_, CallIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn calls)]
	pub type Calls<T: Config> =
		StorageMap<_, Blake2_256, CallIndex, PreimageCall<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageDoubleMap<_, Blake2_256, DorgIndex, Blake2_256, CallIndex, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users_votes)]
	pub type UsersVotes<T: Config> =
		StorageDoubleMap<_, Blake2_256, (DorgIndex, CallIndex), Blake2_256, T::AccountId, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Dorg has been created [dorg]
		DorgCreated(T::AccountId),
		/// a Call has been submited [dorg, call_nonce, submiter]
		CallSubmitted(T::AccountId, u128, T::AccountId),
		/// a Call has been voted [dorg, call_nonce, voter]
		CallVoted(T::AccountId, u128, T::AccountId),
		/// a Call has been executed [dorg, call_nonce]
		CallExecuted(T::AccountId, u128),
		/// a Call has been removed [dorg, call_nonce]
		CallRemoved(T::AccountId, u128),
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

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_call(
			origin: OriginFor<T>,
			dorg_id: T::AccountId,
			call: <T as pallet::Config>::Call,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let id = Self::get_index_from_id(&dorg_id).ok_or(Error::<T>::DorgNotFound)?;

			if !Self::is_user_in_dorg(id, &who) {
				return Err(Error::<T>::NotMember.into())
			}
			let nonce = Self::nonce_call();
			let data = call.encode();
			let deposit = <BalanceOf<T>>::from(data.len() as u32)
				.saturating_mul(T::PreimageByteDeposit::get());

            T::Currency::reserve(&who, deposit)?;

			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data,
				provider: who.clone(),
				deposit,
			};

            Calls::<T>::insert(nonce, preimage);
            Self::deposit_event(Event::<T>::CallSubmitted(dorg_id, nonce, who));

            NonceCall::<T>::put(nonce + 1);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_index_from_id(id: &T::AccountId) -> Option<u128> {
			PalletId::try_from_sub_account(id).map(|(_, index)| index)
		}

		pub fn get_idx_from_id(id: &T::AccountId) -> Option<u128> {
			PalletId::try_from_sub_account(id).map(|(_, val)| val)
		}

		pub fn is_user_in_dorg(dorg_id: u128, user: &T::AccountId) -> bool {
			match Self::dorgs(dorg_id).map(|dorg| dorg.members.contains(user)) {
				None => false,
				Some(r) => r,
			}
		}
	}
}
