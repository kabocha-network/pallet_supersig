use crate::*;
use codec::{Decode, Encode, FullCodec};
use frame_support::{
	pallet_prelude::{MaxEncodedLen, MaybeSerializeDeserialize},
	traits::{Imbalance, OnUnbalanced, WithdrawReasons},
};
use pallet_transaction_payment::CurrencyAdapter;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Zero},
	transaction_validity::{InvalidTransaction, TransactionValidityError},
};
use sp_std::fmt::Debug;

pub(crate) type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub(crate) type NegativeImbalanceOf<C, T> =
	<C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub type SupersigId = u128;
pub type CallId = u128;

#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, MaxEncodedLen, Debug)]
#[codec(mel_bound())]
pub enum Role {
	Standard,
	Master,
	NotMember,
}

impl Default for Role {
	fn default() -> Self {
		Role::NotMember
	}
}

#[derive(Clone, Encode, Decode, TypeInfo, Debug)]
pub struct PreimageCall<AccountId, Balance> {
	pub data: Vec<u8>,
	pub provider: AccountId,
	pub deposit: Balance,
}

pub trait Nope<T: Config> {
	/// The underlying integer type in which fees are calculated.
	type Balance: AtLeast32BitUnsigned
		+ FullCodec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ scale_info::TypeInfo;
	type LiquidityInfo: Default;

	fn withdraw_fee(
		who: &T::AccountId,
		fee: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError>;

	fn correct_and_deposit_fee(
		who: &T::AccountId,
		corrected_fee: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError>;
}

impl<T, C, OU> Nope<T> for CurrencyAdapter<C, OU>
where
	T: Config,
	C: Currency<<T as frame_system::Config>::AccountId>,
	C::PositiveImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::NegativeImbalance,
	>,
	C::NegativeImbalance: Imbalance<
		<C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
		Opposite = C::PositiveImbalance,
	>,
	OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
{
	type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;
	type Balance = <C as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Withdraw the predicted fee from the transaction origin.
	fn withdraw_fee(
		who: &<T>::AccountId,
		fee: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		if fee.is_zero() {
			return Ok(None);
		}

		match C::withdraw(
			who,
			fee,
			WithdrawReasons::TRANSACTION_PAYMENT,
			ExistenceRequirement::KeepAlive,
		) {
			Ok(imbalance) => Ok(Some(imbalance)),
			Err(_) => Err(InvalidTransaction::Payment.into()),
		}
	}

	/// Hand the fee over to the `[OnUnbalanced]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may
	/// be refunded.
	fn correct_and_deposit_fee(
		who: &<T>::AccountId,
		corrected_fee: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
	) -> Result<(), TransactionValidityError> {
		if let Some(paid) = already_withdrawn {
			// Calculate how much refund we should return
			let refund_amount = paid.peek().saturating_sub(corrected_fee);
			// refund to the the account that paid the fees. If this fails, the
			// account might have dropped below the existential balance. In
			// that case we don't refund anything.
			let refund_imbalance = C::deposit_into_existing(who, refund_amount)
				.unwrap_or_else(|_| C::PositiveImbalance::zero());
			// merge the imbalance caused by paying the fees and refunding parts of it again.
			let adjusted_paid = paid
				.offset(refund_imbalance)
				.same()
				.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
			// Call someone else to handle the imbalance (fee and tip separately)
			OU::on_unbalanceds(Some(adjusted_paid).into_iter());
		}
		Ok(())
	}
}
