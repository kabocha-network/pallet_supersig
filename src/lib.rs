//! # Supersig Pallet
//!
//! The supersig pallet is a multisig with super powers.
//! It allows you to add and remove members of the multisig.
//! It extends the capabilities of a multisig so it can be fit for governance of larger funds.
//!
//! A multisig transaction acts more like a funding proposal.
//! And the signatures become votes, with a quorum that can be changed
//!
//! Good to know: the multisig addresses won’t change even though the members can be added or
//! removed.
//!
//! ## Overview
//!
//! The Supersig pallet provide function for:
//!
//! - Creating a supersig
//! - Submit proposal to a supersig
//! - Vote the proposal
//! - Remove a current proposal
//!
//!
//! ### Dispatchable Functions
//!
//! - `create_supersig` - create a supersig, with specified members and threshold -> note of
//!   caution: the creator of the supersig will NOT be added by default, he have to pass his adress
//!   into the list of users.
//! - `submit_call` - make a proposal on the specified supersig
//! - `approve_call` - give a positive vote to a call. if the number of vote = threshold, the call
//! is executed
//! - `remove_call` - remove a call from the poll

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

pub use sp_runtime::traits::{AccountIdConversion, Dispatchable, Hash, Saturating};
pub use sp_std::{boxed::Box, prelude::Vec};

pub mod types;
pub mod weights;

pub use types::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
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
		/// Weigths module
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce_supersig)]
	pub type NonceSupersig<T: Config> = StorageValue<_, SigIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn supersigs)]
	pub type Supersigs<T: Config> =
		StorageMap<_, Blake2_256, SigIndex, Supersig<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_call)]
	pub type NonceCall<T: Config> = StorageMap<_, Blake2_256, SigIndex, CallIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn calls)]
	pub type Calls<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		SigIndex,
		Blake2_256,
		CallIndex,
		PreimageCall<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageDoubleMap<_, Blake2_256, SigIndex, Blake2_256, CallIndex, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn users_votes)]
	pub type UsersVotes<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		(SigIndex, CallIndex),
		Blake2_256,
		T::AccountId,
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supersig has been created [supersig]
		SupersigCreated(T::AccountId),
		/// a Call has been submited [supersig, call_nonce, submiter]
		CallSubmitted(T::AccountId, CallIndex, T::AccountId),
		/// a Call has been voted [supersig, call_nonce, voter]
		CallVoted(T::AccountId, CallIndex, T::AccountId),
		/// a Call has been executed [supersig, call_nonce, result]
		CallExecuted(T::AccountId, CallIndex, DispatchResult),
		/// a Call has been removed [supersig, call_nonce]
		CallRemoved(T::AccountId, CallIndex),
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
		/// the signatory is not the supersig.
		NotAllowed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_supersig())]
		pub fn create_supersig(
			origin: OriginFor<T>,
			members: Vec<T::AccountId>,
			threshold: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig = Supersig::new(members, threshold).ok_or(Error::<T>::InvalidSupersig)?;
			let index = Self::nonce_supersig();
			let supersig_id: T::AccountId = T::PalletId::get().into_sub_account(index);

			let minimum_balance = T::Currency::minimum_balance();
			T::Currency::transfer(
				&who,
				&supersig_id,
				minimum_balance,
				ExistenceRequirement::KeepAlive,
			)?;

			// let supersig = Supersig { members, threshold };

			Supersigs::<T>::insert(index, supersig);
			NonceSupersig::<T>::put(index + 1);

			Self::deposit_event(Event::<T>::SupersigCreated(supersig_id));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::submit_call(call.encode().len() as u32))]
		pub fn submit_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call: Box<<T as pallet::Config>::Call>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

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

		#[pallet::weight(T::WeightInfo::approve_call())]
		pub fn approve_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call_index: CallIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

			if !Self::is_user_in_supersig(sindex, &who) {
				return Err(Error::<T>::NotMember.into())
			}
			if Self::calls(sindex, call_index).is_none() {
				return Err(Error::<T>::CallNotFound.into())
			}
			if Self::users_votes((sindex, call_index), who.clone()) {
				return Err(Error::<T>::AlreadyVoted.into())
			}

			UsersVotes::<T>::insert((sindex, call_index), who.clone(), true);
			Votes::<T>::mutate(sindex, call_index, |val| *val += 1);

			Self::deposit_event(Event::<T>::CallVoted(supersig_id, call_index, who));

			// cannot fail, as the supersig referenced by sindex exist (checked in
			// get_supersig_index_from_id)
			let threshold = Self::supersigs(sindex).unwrap().threshold;
			let total_votes = Self::votes(sindex, call_index);

			if total_votes >= threshold {
				Self::execute_call(sindex, call_index);
			}

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::remove_call())]
		pub fn remove_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call_index: CallIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;
			let preimage = Self::calls(sindex, call_index).ok_or(Error::<T>::CallNotFound)?;
			if who != supersig_id && who != preimage.provider {
				return Err(Error::<T>::NotAllowed.into())
			}
			Self::unchecked_remove_call(sindex, call_index);
			Self::deposit_event(Event::<T>::CallRemoved(supersig_id, call_index));
			T::Currency::unreserve(&preimage.provider, preimage.deposit);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_supersig_index_from_id(id: &T::AccountId) -> Option<u128> {
			if let Some((account, index)) = PalletId::try_from_sub_account(id) {
				if account != T::PalletId::get() {
					return None;
				}
				if index < Self::nonce_supersig() {
					Some(index)
				} else {
                    None
                }
			} else {
				None
			}
		}

		pub fn get_supersig_id_from_index(index: u128) -> T::AccountId {
			T::PalletId::get().into_sub_account(index)
		}

		pub fn is_user_in_supersig(supersig_id: u128, user: &T::AccountId) -> bool {
		    Self::supersigs(supersig_id).map(|supersig| supersig.members.contains(user)).unwrap_or(false)
		}

		pub fn execute_call(supersig_index: u128, call_index: u128) {
			if let Some(preimage) = Self::calls(supersig_index, call_index) {
				let supersig_id = Self::get_supersig_id_from_index(supersig_index);
				if let Ok(call) = <T as Config>::Call::decode(&mut &preimage.data[..]) {
					T::Currency::unreserve(&preimage.provider, preimage.deposit);

					let res = call
						.dispatch(frame_system::RawOrigin::Signed(supersig_id.clone()).into())
						.map(|_| ())
						.map_err(|e| e.error);
					Self::deposit_event(Event::<T>::CallExecuted(supersig_id, call_index, res));
					Self::unchecked_remove_call(supersig_index, call_index);
				}
			}
		}

		pub fn unchecked_remove_call(supersig_index: u128, call_index: u128) {
			Calls::<T>::remove(supersig_index, call_index);
		}
	}
}
