///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use header::Header;

use crate::proto::{MessageError, MIN_BYTES};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod header;
mod impls;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait Streamable {
	fn header(&self) -> Header;

	fn content_as_stream(&self) -> &[u8];

	fn as_stream(&self) -> Box<[u8]> {
		let mut content = self
			.content_as_stream()
			.to_vec();
		if !content.ends_with(&[b'\0']) {
			content.push(b'\0')
		}
		content.reserve(1);
		content.insert(
			0,
			self.header()
				.to_u8(),
		);
		content.into_boxed_slice()
	}
}

pub trait Headed {
	const HEADER: Header;
}

pub trait FromStream {
	type Return;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError>;
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn validate<'a, T>(sliceref: &'a impl AsRef<[u8]>) -> Result<Box<[u8]>, MessageError>
where
	T: ?Sized + Headed + Streamable,
{
	let stream = sliceref.as_ref();
	let len = stream.len();

	if len < MIN_BYTES {
		Err(MessageError::TooShort(len))?
	}

	let header = {
		let wrapped = stream.first();
		let n = *unsafe { wrapped.unwrap_unchecked() };
		Header::new(n).ok_or(MessageError::UIDIsZero)?
	};

	if header.is::<T>() {
		Err(MessageError::UIDMismatch(header.uid(), <T>::HEADER.uid()))?
	}

	if !stream.ends_with(&[b'\0']) {
		return Err(MessageError::NoNull(stream.into()))
	}

	Ok(stream.into())
}
