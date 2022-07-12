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

pub use sp_runtime::traits::{
	AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Dispatchable, Hash,
	Saturating,
};
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

		/// The amount of balance that must be deposited per bytes stored
		#[pallet::constant]
		type DepositPerByte: Get<BalanceOf<Self>>;

		/// The maximum number of account that can added or removed in a single call
		#[pallet::constant]
		type MaxAccountsPerTransaction: Get<u32>;

		/// Weigths module
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn nonce_supersig)]
	pub type NonceSupersig<T: Config> = StorageValue<_, SupersigId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config> =
		StorageDoubleMap<_, Twox64Concat, SupersigId, Twox64Concat, T::AccountId, Role, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_members)]
	pub type TotalMembers<T: Config> = StorageMap<_, Twox64Concat, SupersigId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_deposit)]
	pub type TotalDeposit<T: Config> =
		StorageMap<_, Twox64Concat, SupersigId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nonce_call)]
	pub type NonceCall<T: Config> = StorageMap<_, Twox64Concat, SupersigId, CallId, ValueQuery>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn calls)]
	pub type Calls<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		SupersigId,
		Twox64Concat,
		CallId,
		PreimageCall<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn votes)]
	pub type Votes<T: Config> =
		StorageDoubleMap<_, Twox64Concat, SupersigId, Twox64Concat, CallId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn members_votes)]
	pub type MembersVotes<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Twox64Concat, SupersigId>,
			NMapKey<Twox64Concat, CallId>,
			NMapKey<Twox64Concat, T::AccountId>,
		),
		bool,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supersig has been created [supersig]
		SupersigCreated(T::AccountId),
		/// a supersig have been removed [supersig]
		SupersigRemoved(T::AccountId),
		/// a Call has been submited [supersig, call_nonce, submiter]
		CallSubmitted(T::AccountId, CallId, T::AccountId),
		/// a Call has been voted [supersig, call_nonce, voter]
		CallVoted(T::AccountId, CallId, T::AccountId),
		/// a Call has been executed [supersig, call_nonce, result]
		CallExecutionAttempted(T::AccountId, CallId, Result<DispatchResult, DispatchError>),
		/// a Call has been removed [supersig, call_nonce]
		CallRemoved(T::AccountId, CallId),
		/// the list of users added to the supersig [supersig, [(user, role)]]
		/// Users that were already in the supersig wont appear
		MembersAdded(T::AccountId, Vec<(T::AccountId, Role)>),
		/// the list of users removed from the supersig [supersig, removed_users]
		MembersRemoved(T::AccountId, Vec<T::AccountId>),
		/// a member leaved the supersig [supersig, member]
		SupersigLeaved(T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// supersig must have at least one member
		InvalidNumberOfMembers,
		/// the call origin is not an existing supersig
		NotSupersig,
		/// the call doesn't exist
		CallNotFound,
		/// the user is not a member of the supersig
		NotMember,
		/// the member already voted for the call
		AlreadyVoted,
		/// the signatory is not allowed to perform this call
		NotAllowed,
		/// the supersig couldn't be deleted. This is due to the supersig having locked tokens
		SupersigHaveLockedFunds,
		/// conversion
		Conversion,
		/// overflow
		Overflow,
		/// could not execute the call because it was incorrectly encoded
		BadEncodedCall,
		/// no more valid supersig nonce
		InvalidNonce,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a supersig.
		///
		/// `create_supersig` will create a supersig with specified parameters, and transfer
		/// currencies from the creator to the generated supersig:
		///     - the existencial deposit (minimum amount to make an account alive)
		///     - the price corresponding to the size (in bytes) of the members times the
		///       DepositPerByte
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
			members: BoundedVec<(T::AccountId, Role), T::MaxAccountsPerTransaction>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// A supersig should at least have one member
			let member_length = members.len();
			if member_length < 1 {
				return Err(Error::<T>::InvalidNumberOfMembers.into())
			}

			// Get it id and associated account
			let supersig_id = Self::nonce_supersig();
			let supersig_account: T::AccountId = T::PalletId::get()
				.try_into_sub_account(supersig_id)
				.ok_or(Error::<T>::InvalidNonce)?;

			// Update Members and TotalMembers storages
			let added_members = Self::internal_add_members(supersig_id, members)?;

			// Bring account to existence
			let deposit = Self::compute_deposit(size_of::<T::AccountId>() * added_members.len())?;
			let amount_to_transfer = max(T::Currency::minimum_balance(), deposit);
			T::Currency::transfer(
				&who,
				&supersig_account,
				amount_to_transfer,
				ExistenceRequirement::AllowDeath,
			)?;

			// Prevent the supersig account to sign transaction that would kill it
			frame_system::Pallet::<T>::inc_consumers(&supersig_account)?;

			// Incentive to delete supersigs that are no longer used
			Self::reserve_and_record_deposit(supersig_id, &supersig_account, deposit)?;

			NonceSupersig::<T>::put(supersig_id + 1);

			Self::deposit_event(Event::<T>::SupersigCreated(supersig_account.clone()));
			Self::deposit_event(Event::<T>::MembersAdded(supersig_account, added_members));

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
			supersig_account: T::AccountId,
			call: Box<<T as pallet::Config>::Call>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			let data = call.encode();

			// Incentive to remove proposal that won't be accepted
			let deposit = Self::compute_deposit(data.len())?;
			T::Currency::reserve(&who, deposit)?;

			// The encoded call is stored, along with the infos needed to unreserve the funds
			// associated with it
			let call_id = Self::nonce_call(supersig_id);
			NonceCall::<T>::insert(supersig_id, call_id + 1);
			let preimage = PreimageCall::<T::AccountId, BalanceOf<T>> {
				data,
				provider: who.clone(),
				deposit,
			};
			Calls::<T>::insert(supersig_id, call_id, preimage);

			Self::deposit_event(Event::<T>::CallSubmitted(supersig_account, call_id, who));

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
			supersig_account: T::AccountId,
			call_id: CallId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			if Self::calls(supersig_id, call_id).is_none() {
				return Err(Error::<T>::CallNotFound.into())
			}
			if Self::members_votes((supersig_id, call_id, who.clone())) {
				return Err(Error::<T>::AlreadyVoted.into())
			}

			// Different roles have different voting weight
			let vote_weight = Self::compute_vote_weight(supersig_id, &who)?;

			// Update storages with the user vote
			MembersVotes::<T>::insert((supersig_id, call_id, who.clone()), true);
			Votes::<T>::mutate(supersig_id, call_id, |val| {
				*val = val.saturating_add(vote_weight)
			});

			Self::deposit_event(Event::<T>::CallVoted(
				supersig_account.clone(),
				call_id,
				who,
			));

			let total_votes = Self::votes(supersig_id, call_id);
			if total_votes >= (Self::total_members(supersig_id) / 2 + 1) {
				if let Some(preimage) = Self::calls(supersig_id, call_id) {
					// free storage and unreserve deposit
					Self::unchecked_remove_call_from_storages(supersig_id, call_id);
					T::Currency::unreserve(&preimage.provider, preimage.deposit);

					// Try to decode and execute the call
					let res = if let Ok(call) = <T as Config>::Call::decode(&mut &preimage.data[..])
					{
						Ok(call
							.dispatch(
								frame_system::RawOrigin::Signed(supersig_account.clone()).into(),
							)
							.map(|_| ())
							.map_err(|e| e.error))
					} else {
						Err(Error::<T>::BadEncodedCall.into())
					};

					Self::deposit_event(Event::<T>::CallExecutionAttempted(
						supersig_account,
						call_id,
						res,
					));
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
			supersig_account: T::AccountId,
			call_id: CallId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;
			let preimage = Self::calls(supersig_id, call_id).ok_or(Error::<T>::CallNotFound)?;

			// Either the supersig or the user that created the vote can remove a call
			if who != supersig_account && who != preimage.provider {
				return Err(Error::<T>::NotAllowed.into())
			}

			// Clean up storage and release reserved funds
			Self::unchecked_remove_call_from_storages(supersig_id, call_id);
			T::Currency::unreserve(&preimage.provider, preimage.deposit);

			Self::deposit_event(Event::<T>::CallRemoved(supersig_account, call_id));

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
		#[transactional]
		#[pallet::weight(T::WeightInfo::add_members(new_members.len() as u32))]
		pub fn add_members(
			origin: OriginFor<T>,
			new_members: BoundedVec<(T::AccountId, Role), T::MaxAccountsPerTransaction>,
		) -> DispatchResult {
			let supersig_account = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			// Update Members and TotalMembers storages
			let added_members = Self::internal_add_members(supersig_id, new_members)?;

			// Incentive to delete supersigs that are no longer used
			let deposit = Self::compute_deposit(size_of::<T::AccountId>() * added_members.len())?;
			Self::reserve_and_record_deposit(supersig_id, &supersig_account, deposit)?;

			Self::deposit_event(Event::<T>::MembersAdded(supersig_account, added_members));

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
		#[transactional]
		#[pallet::weight(T::WeightInfo::remove_members(members_to_remove.len() as u32))]
		pub fn remove_members(
			origin: OriginFor<T>,
			members_to_remove: BoundedVec<T::AccountId, T::MaxAccountsPerTransaction>,
		) -> DispatchResult {
			let supersig_account = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			// Remeber the storage state before we remove the members from it
			let total_deposit = Self::total_deposit(supersig_id);
			let initial_total_members = Self::total_members(supersig_id);

			let removed_members = Self::internal_remove_members(supersig_id, members_to_remove)?;

			// amount = total_deposit / initial_n_members * n_removed_members
			let amount_to_unreserve = Self::compute_proportional_amount_to_unreserve(
				total_deposit,
				initial_total_members,
				removed_members.len(),
			)?;

			// Release a proportional amount of deposit
			Self::unreserve_and_record_deposit(supersig_id, &supersig_account, amount_to_unreserve);

			Self::deposit_event(Event::<T>::MembersRemoved(
				supersig_account,
				removed_members,
			));

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
		pub fn delete_supersig(origin: OriginFor<T>, beneficiary: T::AccountId) -> DispatchResult {
			let supersig_account = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			// Release all member related deposits
			let total_deposit = TotalDeposit::<T>::take(supersig_id);
			T::Currency::unreserve(&supersig_account, total_deposit);

			// Release all call related deposits
			Calls::<T>::iter_prefix_values(supersig_id).for_each(|preimage| {
				T::Currency::unreserve(&preimage.provider, preimage.deposit);
			});

			// Erase trace of this supersis in storage and decrement the account reference counter
			Self::unchecked_remove_supersig_from_storages(supersig_id, &supersig_account);

			// Empty the supersig account balance
			// Will cause death of the account
			// Will fail and revert the transaction if this account is not allowed to die
			// due to an other pallet reference counter, reserved or frozen funds
			T::Currency::transfer(
				&supersig_account,
				&beneficiary,
				T::Currency::total_balance(&supersig_account),
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| Error::<T>::SupersigHaveLockedFunds)?;

			Self::deposit_event(Event::<T>::SupersigRemoved(supersig_account));

			Ok(())
		}

		/// leave a supersig
		///
		/// `leave_supersig` will remove caller from selected supersig
		///
		/// The dispatch origin for this call must be `Signed` by the user.
		///
		/// # <weight>
		#[pallet::weight(T::WeightInfo::leave_supersig())]
		pub fn leave_supersig(
			origin: OriginFor<T>,
			supersig_account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let supersig_id = Self::get_supersig_id_from_account(&supersig_account)?;

			if Self::members(supersig_id, &who) == Role::NotMember {
				return Err(Error::<T>::NotMember.into())
			}

			// Remeber the storage state before we remove the members from it
			let total_deposit = Self::total_deposit(supersig_id);
			let initial_total_members = Self::total_members(supersig_id);

			// amount = total_deposit / initial_n_members * n_removed_members
			let amount_to_unreserve = Self::compute_proportional_amount_to_unreserve(
				total_deposit,
				initial_total_members,
				1,
			)?;

			// A supersig should at least have one member
			TotalMembers::<T>::try_mutate(supersig_id, |nb| {
				if *nb == 1 {
					return Err(Error::<T>::InvalidNumberOfMembers)
				};
				*nb -= 1;
				Ok(())
			})?;

			// Note that the votes the user made stays in storage
			Members::<T>::remove(supersig_id, &who);

			// Release a proportional amount of deposit
			Self::unreserve_and_record_deposit(supersig_id, &supersig_account, amount_to_unreserve);

			Self::deposit_event(Event::<T>::SupersigLeaved(supersig_account, who));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn get_supersig_id_from_account(
			supersig_account: &T::AccountId,
		) -> Result<SupersigId, pallet::Error<T>> {
			if let Some((account, supersig_id)) = PalletId::try_from_sub_account(supersig_account) {
				if account != T::PalletId::get() || Self::total_members(supersig_id) == 0 {
					return Err(Error::<T>::NotSupersig)
				}
				Ok(supersig_id)
			} else {
				Err(Error::<T>::NotSupersig)
			}
		}

		fn unchecked_remove_call_from_storages(supersig_id: SupersigId, call_id: CallId) {
			Calls::<T>::remove(supersig_id, call_id);
			Votes::<T>::remove(supersig_id, call_id);
			MembersVotes::<T>::remove_prefix((supersig_id, call_id), None);
		}

		fn unchecked_remove_supersig_from_storages(
			supersig_id: SupersigId,
			supersig_account: &T::AccountId,
		) {
			NonceCall::<T>::remove(supersig_id);
			Members::<T>::remove_prefix(supersig_id, None);
			TotalMembers::<T>::remove(supersig_id);
			Calls::<T>::remove_prefix(supersig_id, None);
			Votes::<T>::remove_prefix(supersig_id, None);
			MembersVotes::<T>::remove_prefix((supersig_id,), None);

			frame_system::Pallet::<T>::dec_consumers(supersig_account);
		}

		fn internal_add_members(
			supersig_id: SupersigId,
			members: BoundedVec<(T::AccountId, Role), T::MaxAccountsPerTransaction>,
		) -> Result<Vec<(T::AccountId, Role)>, Error<T>> {
			let mut added = Vec::new();

			for (member, role) in members {
				if Self::members(supersig_id, &member) == Role::NotMember {
					added.push((member.clone(), role.clone()));
				}
				Members::<T>::insert(supersig_id, member, role);
			}

			TotalMembers::<T>::try_mutate(supersig_id, |n| {
				*n = n
					.checked_add(added.len().try_into().map_err(|_| Error::<T>::Conversion)?)
					.ok_or(Error::<T>::Overflow)?;

				Ok(())
			})?;

			Ok(added)
		}

		fn internal_remove_members(
			supersig_id: SupersigId,
			members: BoundedVec<T::AccountId, T::MaxAccountsPerTransaction>,
		) -> Result<Vec<T::AccountId>, pallet::Error<T>> {
			let mut removed = Vec::new();

			for member in members {
				if Self::members(supersig_id, &member) != Role::NotMember {
					Members::<T>::remove(supersig_id, member.clone());
					removed.push(member);
				}
			}

			TotalMembers::<T>::try_mutate(supersig_id, |n| {
				let new_total_members =
					n.saturating_sub(removed.len().try_into().map_err(|_| Error::<T>::Conversion)?);
				if new_total_members < 1 {
					return Err(Error::<T>::InvalidNumberOfMembers)
				}

				*n = new_total_members;

				Ok(())
			})?;

			Ok(removed)
		}

		fn compute_deposit(data_size: usize) -> Result<BalanceOf<T>, Error<T>> {
			let bytes_stored: u32 = data_size.try_into().map_err(|_| Error::<T>::Conversion)?;

			Ok(<BalanceOf<T>>::from(bytes_stored).saturating_mul(T::DepositPerByte::get()))
		}

		// this function can fail after a storage mutation.
		// extrinsics that use it should have the #[transactional] annotation.
		fn reserve_and_record_deposit(
			supersig_id: SupersigId,
			supersig_account: &T::AccountId,
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			T::Currency::reserve(supersig_account, deposit)?;
			TotalDeposit::<T>::try_mutate(supersig_id, |val| {
				*val = val.checked_add(&deposit).ok_or(Error::<T>::Overflow)?;
				Ok(())
			})
		}

		fn unreserve_and_record_deposit(
			supersig_id: SupersigId,
			supersig_account: &T::AccountId,
			amount: BalanceOf<T>,
		) {
			T::Currency::unreserve(supersig_account, amount);
			TotalDeposit::<T>::mutate(supersig_id, |val| *val = val.saturating_sub(amount));
		}

		fn compute_vote_weight(
			supersig_id: SupersigId,
			who: &T::AccountId,
		) -> Result<u32, Error<T>> {
			match Self::members(supersig_id, &who) {
				Role::Standard => Ok(1),
				Role::Master => Ok(max(Self::total_members(supersig_id) / 2, 1)),
				Role::NotMember => Err(Error::<T>::NotMember),
			}
		}

		pub fn compute_proportional_amount_to_unreserve(
			total_deposit: BalanceOf<T>,
			initial_total_members: u32,
			removed_members: usize,
		) -> Result<BalanceOf<T>, Error<T>> {
			let amount_to_unreserve = total_deposit
				.checked_div(
					&<BalanceOf<T>>::try_from(initial_total_members)
						.map_err(|_| Error::<T>::Conversion)?,
				)
				.ok_or(Error::<T>::Overflow)?
				.checked_mul(
					&<BalanceOf<T>>::try_from(removed_members)
						.map_err(|_| Error::<T>::Conversion)?,
				)
				.ok_or(Error::<T>::Overflow)?;
			Ok(amount_to_unreserve)
		}
	}
}

// RPC calls

#[allow(dead_code)]
impl<T: Config> Pallet<T> {
	pub fn get_account_supersigs(who: T::AccountId) -> Vec<SupersigId> {
		Members::<T>::iter()
			.filter_map(|(supersig_id, member_id, _)| {
				if member_id == who {
					Some(supersig_id)
				} else {
					None
				}
			})
			.collect()
	}

	pub fn get_members_connected(which: SupersigId) -> Vec<(T::AccountId, Role)> {
		Members::<T>::iter_prefix(which).collect()
	}

	// Return :
	// Tuple :
	// Vec<((Vec<u8>, T::AccountId, BalanceOf<T>), Vec<T::AccountId>)> :
	// The tuple inside the vec is just a Call that is unwrap.
	// The vec inside the vec is all the account id that have voted.
	// The second parameter of the tuple is the total amount of members into the supersig.
	pub fn get_proposals(
		which: SupersigId,
	) -> (
		Vec<((Vec<u8>, T::AccountId, BalanceOf<T>), Vec<T::AccountId>)>,
		u32,
	) {
		let member_count = Self::total_members(which);
		let proposal_state = Calls::<T>::iter_prefix(which)
			.map(|(call_id, call)| {
				(
					(call.data, call.provider, call.deposit),
					MembersVotes::<T>::iter_prefix((which, call_id))
						.filter_map(
							|(account_id, vote)| {
								if vote { Some(account_id) } else { None }
							},
						)
						.collect(),
				)
			})
			.collect();
		(proposal_state, member_count)
	}

	// Return :
	// Tuple :
	// The bool is to define if the Call that is asked for state still exists.
	// The vec is all the account id that have voted.
	// The first u32 is the total amount of members in the supersig
	// The second u32 is the total number of votes (not necessary because == members_votes.len())
	pub fn get_proposal_state(
		which: SupersigId,
		call_id: CallId,
	) -> (bool, Vec<T::AccountId>, u32, u32) {
		let member_count = Self::total_members(which);
		let votes = Self::votes(which, call_id);
		let exists = !Self::calls(which, call_id).is_none();
		let member_votes = MembersVotes::<T>::iter_prefix((which, call_id))
			.filter_map(
				|(account_id, vote)| {
					if vote { Some(account_id) } else { None }
				},
			)
			.collect();
		(exists, member_votes, member_count, votes)
	}
}
