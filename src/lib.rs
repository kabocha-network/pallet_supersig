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
    weights::{GetDispatchInfo, PostDispatchInfo},
	PalletId,
};
pub use sp_core::Hasher;

pub use codec::{Decode, Encode};
pub use sp_runtime::traits::{AccountIdConversion, Dispatchable, Hash, Saturating};
pub use sp_std::{
    prelude::Vec,
    boxed::Box,
};

use scale_info::TypeInfo;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq, Eq)]
pub struct Supersig<AccountId> {
	members: Vec<AccountId>,
	threshold: u128,
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug)]
pub struct PreimageCall<AccountId, Balance> {
	data: Vec<u8>,
	provider: AccountId,
	deposit: Balance,
}

pub type SigIndex = u128;
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
        type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

		/// The amount of balance that must be deposited per byte of preimage stored.
		#[pallet::constant]
		type PreimageByteDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce_supersig)]
	pub type NonceSupersig<T: Config> = StorageValue<_, SigIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn supersigs)]
	pub type Supersigs<T: Config> =
		StorageMap<_, Blake2_256, SigIndex, Supersig<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_call)]
	pub type NonceCall<T: Config> = StorageMap<_, Blake2_256, SigIndex, CallIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn calls)]
	pub type Calls<T: Config> =
		StorageDoubleMap<_, Blake2_256, SigIndex, Blake2_256, CallIndex, PreimageCall<T::AccountId, BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageDoubleMap<_, Blake2_256, SigIndex, Blake2_256, CallIndex, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users_votes)]
	pub type UsersVotes<T: Config> =
		StorageDoubleMap<_, Blake2_256, (SigIndex, CallIndex), Blake2_256, T::AccountId, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supersig has been created [supersig]
		SupersigCreated(T::AccountId),
		/// a Call has been submited [supersig, call_nonce, submiter]
		CallSubmitted(T::AccountId, u128, T::AccountId),
		/// a Call has been voted [supersig, call_nonce, voter]
		CallVoted(T::AccountId, u128, T::AccountId),
		/// a Call has been executed [supersig, call_nonce, result]
		CallExecuted(T::AccountId, u128, DispatchResult),
		/// a Call has been removed [supersig, call_nonce]
		CallRemoved(T::AccountId, u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// supersig either have no member or have an invalid treshold (0)
		InvalidSupersig,
		/// the supersig doesn't exist
		SupersigNotFound,
		/// the call already exists
		CallAlreadyExists,
		/// the call doesn't exist
		CallNotFound,
		/// the user is not a member of the supersig
		NotMember,
		/// the user already voted for the call
		AlreadyVoted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_supersig(
			origin: OriginFor<T>,
			members: Vec<T::AccountId>,
			threshold: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if members.is_empty() || threshold == 0 {
				return Err(Error::<T>::InvalidSupersig.into())
			};
			let index = Self::nonce_supersig();
			let supersig_id: T::AccountId = T::PalletId::get().into_sub_account(index);

			let minimum_balance = T::Currency::minimum_balance();
			T::Currency::transfer(
				&who,
				&supersig_id,
				minimum_balance,
				ExistenceRequirement::KeepAlive,
			)?;

			let supersig = Supersig { members, threshold };

			Supersigs::<T>::insert(index, supersig);
			NonceSupersig::<T>::put(index + 1);

			Self::deposit_event(Event::<T>::SupersigCreated(supersig_id));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call: Box<<T as pallet::Config>::Call>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id).ok_or(Error::<T>::SupersigNotFound)?;

			if !Self::is_user_in_supersig(sindex, &who) {
				return Err(Error::<T>::NotMember.into())
			}
			let nonce = Self::nonce_call(sindex);
			let data = call.encode();
			let deposit = <BalanceOf<T>>::from(data.len() as u32)
				.saturating_mul(T::PreimageByteDeposit::get());

            T::Currency::reserve(&who, deposit)?;

			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data,
				provider: who.clone(),
				deposit,
			};

            Calls::<T>::insert(sindex, nonce, preimage);
            Self::deposit_event(Event::<T>::CallSubmitted(supersig_id, nonce, who));

            NonceCall::<T>::insert(sindex, nonce + 1);
			Ok(())
		}

        // ^
        // |
        // |
        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!
        //
        // DEPENDING ON PERFS, ONLY ONE OF THOSE CALL WILL BE KEPT
        //
        // !!!!!!!!!!!!!!!!!!!!!!!!!!!!
        // |
        // |
        // v

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_encoded_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			encoded_call: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id).ok_or(Error::<T>::SupersigNotFound)?;

			if !Self::is_user_in_supersig(sindex, &who) {
				return Err(Error::<T>::NotMember.into())
			}
			let nonce = Self::nonce_call(sindex);
			let deposit = <BalanceOf<T>>::from(encoded_call.len() as u32)
				.saturating_mul(T::PreimageByteDeposit::get());

            T::Currency::reserve(&who, deposit)?;

			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data: encoded_call,
				provider: who.clone(),
				deposit,
			};

            Calls::<T>::insert(sindex, nonce, preimage);
            Self::deposit_event(Event::<T>::CallSubmitted(supersig_id, nonce, who));

            NonceCall::<T>::insert(sindex, nonce + 1);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn approve_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call_index: CallIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id).ok_or(Error::<T>::SupersigNotFound)?;

			if !Self::is_user_in_supersig(sindex, &who) {
				return Err(Error::<T>::NotMember.into())
			}
            if Self::users_votes((sindex, call_index), who.clone()) {
                return Err(Error::<T>::AlreadyVoted.into())
            }

            UsersVotes::<T>::insert((sindex, call_index), who.clone(), true);
            Votes::<T>::insert(sindex, call_index, Self::votes(sindex, call_index) + 1);

            Self::deposit_event(Event::<T>::CallVoted(supersig_id, call_index, who));

            let threshold = Self::supersigs(sindex).unwrap().threshold;
            let total_votes = Self::votes(sindex, call_index);

            if total_votes >= threshold {
                Self::execute_call(sindex, call_index);
            }

            Ok(())
        }
	}

	impl<T: Config> Pallet<T> {
		pub fn get_supersig_index_from_id(id: &T::AccountId) -> Option<u128> {
			PalletId::try_from_sub_account(id).map(|(_, index)| index)
		}
		pub fn get_supersig_id_from_index(index: u128) -> Option<T::AccountId> {
            T::PalletId::get().into_sub_account(index)
		}
		pub fn is_user_in_supersig(supersig_id: u128, user: &T::AccountId) -> bool {
			match Self::supersigs(supersig_id).map(|supersig| supersig.members.contains(user)) {
				None => false,
				Some(r) => r,
			}
		}

        pub fn execute_call(supersig_index: u128, call_index: u128) {
            let preimage = Self::calls(supersig_index, call_index).unwrap();
            let supersig_id = Self::get_supersig_id_from_index(supersig_index).unwrap();
            if let Ok(call) = <T as Config>::Call::decode(&mut &preimage.data[..]) {

                T::Currency::unreserve(&preimage.provider, preimage.deposit);

                let res = call.dispatch(frame_system::RawOrigin::Signed(supersig_id.clone()).into())
                    .map(|_| ())
					.map_err(|e| e.error);
                Self::deposit_event(Event::<T>::CallExecuted(supersig_id, call_index, res));
                Self::remove_call(supersig_index, call_index);
            }
        }

        pub fn remove_call(supersig_index: u128, call_index: u128) {
            Calls::<T>::remove(supersig_index, call_index);
            Votes::<T>::remove(supersig_index, call_index);
            UsersVotes::<T>::remove_prefix((supersig_index, call_index), None);
        }
	}
}
