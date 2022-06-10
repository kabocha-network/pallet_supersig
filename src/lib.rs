//! # Supersig Pallet
//!
//! The supersig pallet extends the capabilities of a multisig so it can be fit for governance of
//! larger funds. It is a superset of the multisig pallet, adding multiple functionalities and
//! options to the original multi-signature dispatch allowing multiple signed origins (accounts) to
//! coordinate and dispatch a call from the supersig account
//!
//! Note: the multisig addresses won’t change even though the members can be added, removed, or can
//! leave themselves
//!
//! ## Overview
//!
//! The Supersig pallet provide function for:
//!
//! - Creating a supersig
//! - Adding and removing members
//! - Leaving the supersig
//! - Submit transaction to a supersig
//! - Vote for the transaction
//! - Remove a pending transaction
//! - Delete a supersig
//!
//!
//! ### Dispatchable Functions
//!
//! - `create_supersig` - create a supersig, with specified members. The creator will have to
//!   deposit an existencial balance and a deposit that depend on the number of members, in the
//!   supersig account. This last amount will be reserved on the supersig
//!
//!   /!!\ note of caution /!!\ the creator of the supersig will NOT be added by default, he will
//!   have to pass his adress into the list of added users.
//!
//! - `submit_call` - make a proposal on the specified supersig. an amount corresponding to the
//!   length of the encoded call will be reserved.
//!
//! - `approve_call` - give a positive vote to a call. if the number of vote >= SimpleMajority, the
//!   call is executed. An user can only approve a call once.
//!
//! - `remove_call` - remove a call from the poll. The reserved amount of the proposer will be
//!   unreserved
//!
//! - `add_members` - add new members to the supersig. In case some user are already in the
//!   supersig, they will be ignored.
//!
//! - `remove_members` - remove members from the supersig. In case some user are not in the
//!   supersig, they will be ignored.
//!
//! - `delete_supersig` - remove the supersig and all the associated data. Funds will be unreserved
//!   and transfered to specified beneficiary.
//!
//! - `leave_supersig` - remove the caller from the supersig.

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
pub use sp_std::{boxed::Box, cmp::max, mem::size_of, prelude::Vec};

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

		/// The amount of balance that must be deposited per bytes stored.
		#[pallet::constant]
		type PricePerByte: Get<BalanceOf<Self>>;

		///the maximum users allowed per transaction (adding, removing...)
		#[pallet::constant]
		type MaxUsersPerTransaction: Get<u32>;

		/// Weigths module
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn nonce_supersig)]
	pub type NonceSupersig<T: Config> = StorageValue<_, SigIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageDoubleMap<_, Blake2_256, SigIndex, Blake2_256, T::AccountId, Role, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_members)]
	pub type TotalMembers<T: Config> = StorageMap<_, Blake2_256, SigIndex, u32, ValueQuery>;

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
		StorageDoubleMap<_, Blake2_256, SigIndex, Blake2_256, CallIndex, u32, ValueQuery>;

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
		/// in the supersig wont appear [supersig, [(user, role)]]
		UsersAdded(T::AccountId, Vec<(T::AccountId, Role)>),
		/// the list of users removed from the supersig. [supersig, removed_users]
		UsersRemoved(T::AccountId, Vec<T::AccountId>),
		/// a supersig have been removed [supersig_id]
		SupersigRemoved(T::AccountId),
		/// a member leaved the supersig [supersig_id, member]
		SupersigLeaved(T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// supersig have not enough members (minimum 2 members)
		InvalidSupersig,
		/// the supersig doesn't exist
		SupersigNotFound,
		/// the call doesn't exist
		CallNotFound,
		/// the user is not a member of the supersig
		NotMember,
		/// the user already voted for the call
		AlreadyVoted,
		/// the signatory is not the supersig.
		NotAllowed,
		/// the supersig couldn't be deleted. This is due to the supersig having locked tokens
		SupersigHaveLockedFunds,
		/// an user cannot be removed if it leave 0 users in the supersig.
		CannotRemoveUsers,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a supersig.
		///
		/// `create_supersig` will create a supersig with specified parameters, and transfer
		/// currencies from the creator to the generated supersig:
		///     - the existencial deposit (minimum amount to make an account alive)
		///     - the price corresponding to the size (in bytes) of the members times the
		///       PricePerByte
		///
		/// The dispatch origin for this call must be `Signed`.
		///
		/// # <weight>
		///
		/// Related functions:
		/// - `Currency::transfer` will be called once to deposit an existencial amount on supersig
		/// - `frame_system::inc_consumers` will be called once to protect the supersig from
		///   deletion
		#[transactional]
		#[pallet::weight(T::WeightInfo::create_supersig(members.len() as u32))]
		pub fn create_supersig(
			origin: OriginFor<T>,
			members: BoundedVec<(T::AccountId, Role), T::MaxUsersPerTransaction>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let member_length = members.len();
			if member_length < 1 {
				return Err(Error::<T>::InvalidSupersig.into())
			}
			let index = Self::nonce_supersig();
			let supersig_id: T::AccountId = T::PalletId::get().into_sub_account(index);

			let price = <BalanceOf<T>>::from(size_of::<T::AccountId>() as u32)
				.saturating_mul((member_length as u32).into())
				.saturating_mul(T::PricePerByte::get());
			let deposit = max(T::Currency::minimum_balance(), price);

			T::Currency::transfer(
				&who,
				&supersig_id,
				deposit,
				ExistenceRequirement::AllowDeath,
			)?;

			frame_system::Pallet::<T>::inc_consumers(&supersig_id)?;

			T::Currency::reserve(&supersig_id, price)?;

			Self::internal_add_members(index, members);
			NonceSupersig::<T>::put(index + 1);

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

			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;
			let data = call.encode();

			let deposit =
				<BalanceOf<T>>::from(data.len() as u32).saturating_mul(T::PricePerByte::get());
			T::Currency::reserve(&who, deposit)?;

			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data,
				provider: who.clone(),
				deposit,
			};
			let nonce = Self::nonce_call(supersig_index);
			Calls::<T>::insert(supersig_index, nonce, preimage);
			NonceCall::<T>::insert(supersig_index, nonce + 1);

			Self::deposit_event(Event::<T>::CallSubmitted(supersig_id, nonce, who));
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
			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;

			let role = Self::members(supersig_index, &who);
			if role == Role::NotMember {
				return Err(Error::<T>::NotMember.into())
			}

			let member_number = Self::total_members(supersig_index);
			let vote_weight = match role {
				Role::Standard => 1,
				Role::Master => max(member_number / 2, 1),
				_ => return Err(Error::<T>::NotMember.into()),
			};

			if role == Role::NotMember {}
			if Self::calls(supersig_index, call_index).is_none() {
				return Err(Error::<T>::CallNotFound.into())
			}
			if Self::users_votes((supersig_index, call_index, who.clone())) {
				return Err(Error::<T>::AlreadyVoted.into())
			}

			UsersVotes::<T>::insert((supersig_index, call_index, who.clone()), true);
			Votes::<T>::mutate(supersig_index, call_index, |val| {
				*val += vote_weight
			});

			Self::deposit_event(Event::<T>::CallVoted(supersig_id.clone(), call_index, who));

			let total_votes = Self::votes(supersig_index, call_index);
			if total_votes >= (member_number / 2 + 1) {
				if let Some(preimage) = Self::calls(supersig_index, call_index) {
					if let Ok(call) = <T as Config>::Call::decode(&mut &preimage.data[..]) {
						T::Currency::unreserve(&preimage.provider, preimage.deposit);

						let res = call
							.dispatch(frame_system::RawOrigin::Signed(supersig_id.clone()).into())
							.map(|_| ())
							.map_err(|e| e.error);
						Self::unchecked_remove_call(supersig_index, call_index);
						Self::deposit_event(Event::<T>::CallExecuted(supersig_id, call_index, res));
					}
				}
			}

			Ok(())
		}

		/// remove a call from the supersig
		///
		/// `remove_call` will remove a call from the poll.
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
			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;
			let preimage =
				Self::calls(supersig_index, call_index).ok_or(Error::<T>::CallNotFound)?;
			if who != supersig_id && who != preimage.provider {
				return Err(Error::<T>::NotAllowed.into())
			}
			Self::unchecked_remove_call(supersig_index, call_index);

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
			new_members: BoundedVec<(T::AccountId, Role), T::MaxUsersPerTransaction>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;
			let new_members = new_members;

			let added = Self::internal_add_members(supersig_index, new_members);

			let deposit = <BalanceOf<T>>::from(size_of::<T::AccountId>() as u32)
				.saturating_mul((added.len() as u32).into())
				.saturating_mul(T::PricePerByte::get());
			T::Currency::reserve(&supersig_id, deposit)?;

			Self::deposit_event(Event::<T>::UsersAdded(supersig_id, added));

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
			members_to_remove: BoundedVec<T::AccountId, T::MaxUsersPerTransaction>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;

			let removed = Self::internal_remove_members(supersig_index, members_to_remove)?;
			let reserve = <BalanceOf<T>>::from(size_of::<T::AccountId>() as u32)
				.saturating_mul((removed.len() as u32).into())
				.saturating_mul(T::PricePerByte::get());
			T::Currency::unreserve(&who, reserve);

			Self::deposit_event(Event::<T>::UsersRemoved(supersig_id, removed));

			Ok(())
		}

		/// remove the supersig
		///
		/// `delete_supersig` will remove every members, transfer every remanent funds to the
		/// target account, remove the supersig from storage, and set the consumers and providers
		/// to 0
		///
		/// The dispatch origin for this call must be `Signed` by the supersig
		///
		/// # <weight>
		#[transactional]
		#[pallet::weight(T::WeightInfo::delete_supersig())]
		pub fn delete_supersig(
			origin: OriginFor<T>,
			supersig_id: T::AccountId,
			beneficiary: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if who != supersig_id {
				return Err(Error::<T>::NotAllowed.into())
			}
			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;

			let nb_members = Self::total_members(supersig_index);
			let reserve = <BalanceOf<T>>::from(size_of::<T::AccountId>() as u32)
				.saturating_mul((nb_members as u32).into())
				.saturating_mul(T::PricePerByte::get());

			let total_balance = T::Currency::total_balance(&supersig_id);
			if total_balance != T::Currency::free_balance(&supersig_id).saturating_add(reserve) {
				return Err(Error::<T>::SupersigHaveLockedFunds.into())
			}
			T::Currency::unreserve(&who, reserve);

            Calls::<T>::iter_prefix_values(supersig_index).for_each(|preimage| {
			    T::Currency::unreserve(&preimage.provider, preimage.deposit);
            });

			NonceCall::<T>::remove(supersig_index);
			Members::<T>::remove_prefix(supersig_index, None);
			TotalMembers::<T>::remove(supersig_index);
			Calls::<T>::remove_prefix(supersig_index, None);
			Votes::<T>::remove_prefix(supersig_index, None);
			UsersVotes::<T>::remove_prefix((supersig_index,), None);

			frame_system::Pallet::<T>::dec_consumers(&supersig_id);

			T::Currency::transfer(
				&supersig_id,
				&beneficiary,
				total_balance,
				ExistenceRequirement::AllowDeath,
			)?;

			Self::deposit_event(Event::<T>::SupersigRemoved(supersig_id));
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
		#[pallet::weight(T::WeightInfo::leave_supersig())]
		pub fn leave_supersig(origin: OriginFor<T>, supersig_id: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let supersig_index = Self::get_supersig_index_from_id(&supersig_id)?;
			if Self::members(supersig_index, &who) == Role::NotMember {
				return Err(Error::<T>::NotMember.into())
			}

			TotalMembers::<T>::try_mutate(supersig_index, |nb| {
				if *nb - 1 == 0 {
					return Err(Error::<T>::CannotRemoveUsers)
				};
				*nb -= 1;
				Ok(*nb)
			})?;
			Members::<T>::remove(supersig_index, &who);
			Self::deposit_event(Event::<T>::SupersigLeaved(supersig_id, who));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_supersig_index_from_id(id: &T::AccountId) -> Result<u128, pallet::Error<T>> {
			if let Some((account, index)) = PalletId::try_from_sub_account(id) {
				if account != T::PalletId::get() || Self::total_members(index) == 0 {
					return Err(Error::<T>::SupersigNotFound)
				}
				Ok(index)
			} else {
				Err(Error::<T>::SupersigNotFound)
			}
		}

		pub fn unchecked_remove_call(supersig_index: u128, call_index: u128) {
			Calls::<T>::remove(supersig_index, call_index);
			Votes::<T>::remove(supersig_index, call_index);
			UsersVotes::<T>::remove_prefix((supersig_index, call_index), None);
		}

		pub fn internal_add_members(
			supersig_idx: u128,
			members: BoundedVec<(T::AccountId, Role), T::MaxUsersPerTransaction>,
		) -> Vec<(T::AccountId, Role)> {
			let mut added = Vec::new();
			for (member, role) in members {
				if Self::members(supersig_idx, &member) == Role::NotMember {
					added.push((member.clone(), role.clone()));
				}
				Members::<T>::insert(supersig_idx, member, role);
			}
			TotalMembers::<T>::mutate(supersig_idx, |n| *n += added.len() as u32);

			added
		}

		#[transactional]
		pub fn internal_remove_members(
			supersig_idx: u128,
			members: BoundedVec<T::AccountId, T::MaxUsersPerTransaction>,
		) -> Result<Vec<T::AccountId>, pallet::Error<T>> {
			let mut removed = Vec::new();

			for member in members {
				if Self::members(supersig_idx, &member) != Role::NotMember {
					Members::<T>::remove(supersig_idx, member.clone());
					removed.push(member);
				}
			}
			let new_total_members = Self::total_members(supersig_idx) - (removed.len() as u32);
			if new_total_members < 1 {
				return Err(Error::<T>::CannotRemoveUsers)
			}
			TotalMembers::<T>::insert(supersig_idx, new_total_members);

			Ok(removed)
		}
	}
}
