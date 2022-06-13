use super::mock::*;
use sp_runtime::traits::AccountIdConversion;

pub fn get_supersig_account(index: u64) -> <Test as frame_system::Config>::AccountId {
	SupersigPalletId::get().into_sub_account(index)
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}
