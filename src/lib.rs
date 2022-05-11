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
    traits::Currency,
};

// pub use sp_runtime::traits::BlakeTwo256;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct Dorg<AccountId> {
    account: AccountId,
    members: Vec<AccountId>,
}

#[derive(Clone, Encode, Decode, TypeInfo)]
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
	}

	#[pallet::storage]
	#[pallet::getter(fn dorg)]
	pub type Dorgs<T: Config> = StorageValue<_, Vec<Dorg<T::AccountId>>>;

    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_256,
        Dorg<T::AccountId>,
        Blake2_256,
        CallHash,
        u32
    >;

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
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Ok(().into())
		}
	}
}
