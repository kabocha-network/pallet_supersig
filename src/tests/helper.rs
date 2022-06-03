use super::mock::*;
use sp_runtime::traits::AccountIdConversion;

pub fn get_account_id(index: u64) -> <Test as frame_system::Config>::AccountId {
	SupersigPalletId::get().into_sub_account(index)
}

// fn events() -> Vec<Event> {
// 	let evt = System::events().into_iter().map(|evt| evt.event).collect::<Vec<_>>();
//
// 	System::reset_events();
//
// 	evt
// }

pub fn last_event() -> Event {
	frame_system::Pallet::<Test>::events().pop().expect("Event expected").event
}
