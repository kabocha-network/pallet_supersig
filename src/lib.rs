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
//! - `create_supersig` - create a supersig, with specified members -> note of caution: the creator
//!   of the supersig will NOT be added by default, he have to pass his adress into the list of
//!   users.
//! - `submit_call` - make a proposal on the specified supersig
//! - `approve_call` - give a positive vote to a call. if the number of vote >= SimpleMajority, the
//!   call
//! is executed
//! - `remove_call` - remove a call from the poll

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use frame_support::{
	dispatch::DispatchResult,
	traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
	transactional,
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
	pub type UsersVotes<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_256, SigIndex>,
			NMapKey<Blake2_256, CallIndex>,
			NMapKey<Blake2_256, T::AccountId>,
		),
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
		/// the list of users added to the supersig. Users that were already
		/// in the supersig wont appear [supersig, added_users]
		UsersAdded(T::AccountId, Vec<T::AccountId>),
		/// the list of users removed from the supersig. [supersig, removed_users]
		UsersRemoved(T::AccountId, Vec<T::AccountId>),
		/// a supersig have been removed [supersig_id]
		SupersigRemoved(T::AccountId),
		/// a member leaved the supersig [supersig_id, member]
		SupersigLeaved(T::AccountId, T::AccountId),
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
		/// the supersig couldn't be deleted. This is due to the supersig having locked tokens
		CannotDeleteSupersig,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a supersig.
		///
		/// `create_supersig` will create a supersig with specified parameters, and transfer an
		/// existencial deposit from the creator to the generated supersig account, to bring the
		/// account to life.
		///
		/// The dispatch origin for this call must be `Signed`.
		///
		/// # <weight>
		///
		/// Related functions:
		/// - `Currency::transfer` will be called once to deposit an existencial amount on supersig
		#[transactional]
		#[pallet::weight(T::WeightInfo::create_supersig())]
		pub fn create_supersig(origin: OriginFor<T>, members: Vec<T::AccountId>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig = Supersig::new(members).ok_or(Error::<T>::InvalidSupersig)?;
			let index = Self::nonce_supersig();
			let supersig_id: T::AccountId = T::PalletId::get().into_sub_account(index);

			let minimum_balance = T::Currency::minimum_balance();
			T::Currency::transfer(
				&who,
				&supersig_id,
				minimum_balance,
				ExistenceRequirement::KeepAlive,
			)?;

			Supersigs::<T>::insert(index, supersig);
			NonceSupersig::<T>::put(index + 1);

			// This mean the supersig account cannot be emptied while existing in this storage
			frame_system::Pallet::<T>::inc_consumers(&supersig_id)?;

			Self::deposit_event(Event::<T>::SupersigCreated(supersig_id));

			Ok(())
		}

		/// submit a call to a specific supersig.
		///
		/// `submit_call` will create a proposal on the supersig, that members can approve.
		/// this will lock an amount that depend on the lenght of the encoded call, to prevent spam
		///
		/// The dispatch origin for this call must be `Signed`, and the origin must be a
		/// supersig's member
		///
		/// # <weight>
		///
		/// Related functions:
		/// - `Currency::reserve` will be called once to lock the deposit amount
		#[pallet::weight(T::WeightInfo::submit_call(call.encode().len() as u32))]
		pub fn submit_call(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			call: Box<<T as pallet::Config>::Call>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

			let nonce = Self::nonce_call(sindex);
			let data = call.encode();
			let deposit = <BalanceOf<T>>::from(data.len() as u32)
				.saturating_mul(T::PreimageByteDeposit::get());

			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data: data.clone(),
				provider: who.clone(),
				deposit,
			};

			if Calls::<T>::iter_prefix_values(sindex).any(|elem| elem.data == data) {
				return Err(Error::<T>::CallAlreadyExists)?
			}

			T::Currency::reserve(&who, deposit)?;

			Calls::<T>::insert(sindex, nonce, preimage);
			Self::deposit_event(Event::<T>::CallSubmitted(supersig_id, nonce, who));
			NonceCall::<T>::insert(sindex, nonce + 1);
			Ok(())
		}

		/// vote for a call in the supersig
		///
		/// `approve_call` will add a positive, unique vote to the specified call proposal.
		/// if the numbers of votes on this proposal = SimpleMajority (51%), then the call is
		/// executed
		///
		/// The dispatch origin for this call must be `Signed`, and the origin must be a
		/// supersig's member
		///
		/// # <weight>
		///
		/// Related functions:
		/// - `Currency::unreserve` will be called once IF SimpleMajority is reached
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
			if Self::users_votes((sindex, call_index, who.clone())) {
				return Err(Error::<T>::AlreadyVoted.into())
			}

			UsersVotes::<T>::insert((sindex, call_index, who.clone()), true);
			Votes::<T>::mutate(sindex, call_index, |val| *val += 1);

			Self::deposit_event(Event::<T>::CallVoted(supersig_id, call_index, who));

			// cannot fail, as the supersig referenced by sindex exist (checked in
			// get_supersig_index_from_id)
			let supersig = Self::supersigs(sindex).unwrap();
			let total_votes = Self::votes(sindex, call_index);

			if total_votes >= (supersig.members.len() as u128 / 2 + 1) {
				Self::execute_call(sindex, call_index);
			}

			Ok(())
		}

		/// remove a call from the supersig
		///
		/// `remove_call` will remove a call from the poll. For trensparency reason, the votes
		/// wont be removed. This will aslo unlock deposited funds
		///
		/// The dispatch origin for this call must be `Signed` by either the supersig or the
		/// account who submited the call
		///
		/// # <weight>
		///
		/// Related functions:
		/// - `Currency::unreserve` will be called once
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

			T::Currency::unreserve(&preimage.provider, preimage.deposit);

			Self::deposit_event(Event::<T>::CallRemoved(supersig_id, call_index));
			Ok(())
		}

		/// add members the supersig
		///
		/// `add members` will add a list of addesses to the members list of the supersig.
		/// if an address is already present, it will be ignored.
		///
		/// The dispatch origin for this call must be `Signed` by the supersig
		///
		/// # <weight>
		#[pallet::weight(T::WeightInfo::add_members(new_members.len() as u32))]
		pub fn add_members(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			new_members: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;
			let mut new_members = new_members;

			Supersigs::<T>::mutate(sindex, |wrapped_supersig| {
				if let Some(supersig) = wrapped_supersig {
					new_members.retain(|memb| !supersig.members.contains(memb));
					for _ in 0..new_members.len() {
						// this will never fail, because we set provider to 1 when creating the
						// supersig
						let _ = frame_system::Pallet::<T>::inc_consumers(&supersig_id);
					}
					supersig.members.append(new_members.as_mut());
				}
			});

			Self::deposit_event(Event::<T>::UsersAdded(supersig_id, new_members));

			Ok(())
		}

		/// remove members from the supersig
		///
		/// `remove_members` will remove a list of addesses from the members list of the supersig.
		/// if an address is not present, it will be ignored.
		///
		/// The dispatch origin for this call must be `Signed` by the supersig
		///
		/// # <weight>
		#[pallet::weight(T::WeightInfo::remove_members(members_to_remove.len() as u32))]
		pub fn remove_members(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			members_to_remove: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

			Supersigs::<T>::mutate(sindex, |wrapped_supersig| {
				if let Some(supersig) = wrapped_supersig {
					let old_len = supersig.members.len();
					supersig.members.retain(|memb| !members_to_remove.contains(memb));
					// this will never fail, because we set provider to 1 when creating the
					// supersig
					for _ in 0..(old_len - supersig.members.len()) {
						frame_system::Pallet::<T>::dec_consumers(&supersig_id);
					}
				}
			});

			Self::deposit_event(Event::<T>::UsersRemoved(supersig_id, members_to_remove));

			Ok(())
		}

		/// remove the supersig
		///
		/// `remove_supersig` will remove every members, transfer every remanent funds to the
		/// target account, remove the supersig from storage, and set the consumers and providers
		/// to 0
		///
		/// The dispatch origin for this call must be `Signed` by the supersig
		///
		/// # <weight>
		#[pallet::weight(T::WeightInfo::remove_supersig())]
		pub fn remove_supersig(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

			let balance = T::Currency::total_balance(&supersig_id);
			if balance != T::Currency::free_balance(&supersig_id) {
				return Err(Error::<T>::CannotDeleteSupersig.into())
			}
			// the supersig exist
			let nb_members = Self::supersigs(sindex).unwrap().members.len();
			NonceCall::<T>::remove(sindex);
			Supersigs::<T>::remove(sindex);
			Calls::<T>::remove_prefix(sindex, None);
			Votes::<T>::remove_prefix(sindex, None);
			UsersVotes::<T>::remove_prefix((sindex,), None);
			// this will never fail, because we set provider to 1 when creating the
			// supersig
			for _ in 0..nb_members {
				frame_system::Pallet::<T>::dec_consumers(&supersig_id);
			}
			// the source account have enough funds.
			let _ = T::Currency::transfer(
				&supersig_id,
				&beneficiary,
				balance,
				ExistenceRequirement::AllowDeath,
			);
			// the account exist, and there is no consumers anymore, so we can ignore result
			let _ = frame_system::Pallet::<T>::dec_providers(&supersig_id);
			Ok(())
		}

		/// leave a supersig
		///
		/// `leave_supersig` will remove caller from selected supersig
		///
		/// The dispatch origin for this call must be `Signed` by the user.
		///
		/// # <weight>
		#[transactional]
		#[pallet::weight(T::WeightInfo::create_supersig())]
		pub fn leave_supersig(origin: OriginFor<T>, supersig_id: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let sindex = Self::get_supersig_index_from_id(&supersig_id)
				.ok_or(Error::<T>::SupersigNotFound)?;

			if !Self::is_user_in_supersig(sindex, &who) {
				return Err(Error::<T>::NotMember.into())
			}

			Supersigs::<T>::mutate(sindex, |wrapped_supersig| {
				if let Some(supersig) = wrapped_supersig {
					supersig.members.retain(|memb| memb != &who);
				}
			});
			Self::deposit_event(Event::<T>::SupersigLeaved(supersig_id, who));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_supersig_index_from_id(id: &T::AccountId) -> Option<u128> {
			if let Some((account, index)) = PalletId::try_from_sub_account(id) {
				if account != T::PalletId::get() {
					return None
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
			Self::supersigs(supersig_id)
				.map(|supersig| supersig.members.contains(user))
				.unwrap_or(false)
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
			Votes::<T>::remove(supersig_index, call_index);
			UsersVotes::<T>::remove_prefix((supersig_index, call_index), None);
		}
	}
}
