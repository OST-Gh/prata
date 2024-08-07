///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use header::Header;

use crate::proto::MessageError;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod header;
mod impls;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait Streamable {
	const HEADER: Header;

	fn content_as_stream(&self) -> &[u8];

	fn as_stream(&self) -> Box<[u8]> {
		let mut content = self
			.content_as_stream()
			.to_vec();
		if !content.ends_with(&[b'\0']) {
			content.push(b'\0')
		}
		content.reserve(1);
		content.insert(0, Self::HEADER.to_u8());
		content.into_boxed_slice()
	}
}

pub trait FromStream {
	type Return;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError>;
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn validate<'a, T>(sliceref: &'a impl AsRef<[u8]>) -> Result<Box<[u8]>, MessageError>
where
	T: ?Sized + Streamable,
{
	let stream = sliceref.as_ref();
	let len = stream.len();

	if len < 1 {
		Err(MessageError::TooShort(len))?
	}

	let header = {
		let wrapped = stream.first();
		Header::new(*unsafe { wrapped.unwrap_unchecked() })
	};

	if header.is::<T>() {
		Err(MessageError::UIDMismatch(header.uid(), <T>::HEADER.uid()))?
	}

	if !stream.ends_with(&[b'\0']) {
		return Err(MessageError::NoNull(stream.into()))
	}

	Ok(stream.into())
}
