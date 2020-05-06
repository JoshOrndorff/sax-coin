#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use sp_runtime::RuntimeString;
use frame_support::{
	decl_module, decl_storage, decl_error, ensure,
	traits::Currency,
	weights::SimpleDispatchInfo,
};
use system::ensure_none;
use sp_inherents::{InherentIdentifier, InherentData, ProvideInherent, IsFatalError};
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use codec::{Encode, Decode};
use core::convert::TryFrom;

/// The pallet's configuration trait. Nothing to configure.
pub trait Trait: system::Trait {
	//TODO something to tell me what the per-block reward is
	// Or maybe just integrate that into this same pallet.
	type Currency: Currency<<Self as system::Trait>::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Author already set in block.
		AuthorAlreadySet,
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Rewards {
		Author: Option<T::AccountId>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		#[weight = SimpleDispatchInfo::FixedOperational(10_000)]
		fn set_author(origin, author: T::AccountId) {
			ensure_none(origin)?;
			ensure!(Author::<T>::get().is_none(), Error::<T>::AuthorAlreadySet);

			Author::<T>::put(author);
		}

		fn on_finalize(_n: T::BlockNumber) {

			//TODO grab reward from storage
			let reward = BalanceOf::<T>::try_from(1000).ok().expect("1000 fits");

			if let Some(author) = Author::<T>::get() {
				drop(T::Currency::deposit_creating(&author, reward));
			}

			// Reset the author to none for the next block.
			<Self as Store>::Author::kill();
		}
	}
}

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Debug, Decode))]
pub enum InherentError {
	Other(RuntimeString),
}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		match *self {
			InherentError::Other(_) => true,
		}
	}
}

impl InherentError {
	/// Try to create an instance ouf of the given identifier and data.
	#[cfg(feature = "std")]
	pub fn try_from(id: &InherentIdentifier, data: &[u8]) -> Option<Self> {
		if id == &INHERENT_IDENTIFIER {
			<InherentError as codec::Decode>::decode(&mut &data[..]).ok()
		} else {
			None
		}
	}
}

/// The type of data that the inherent will contain.
/// Just a byte array. It will be decoded to an actual pubkey later
pub type InherentType = Vec<u8>;

#[cfg(feature = "std")]
pub struct InherentDataProvider(pub InherentType);

#[cfg(feature = "std")]
impl ProvideInherentData for InherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

impl<T: Trait> ProvideInherent for Module<T> {
	type Call = Call<T>;
	type Error = InherentError;
	const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

	fn create_inherent(data: &InherentData) -> Option<Self::Call> {
		// Grab the Vec<u8> labelled with "author_" from the map of all inherent data
		let author_raw = data.get_data::<InherentType>(&INHERENT_IDENTIFIER)
			.expect("Gets and decodes authorship inherent data")?;

		// Decode the Vec<u8> into an actual author
		let author = T::AccountId::decode(&mut &author_raw[..])
			.expect("Decodes author raw inherent data");

		Some(Call::set_author(author))
	}
}
