///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::str;

use crate::proto::{
	messages::{Connect, Disconnect},
	stream::{
		header::{Header, Offset, Uid},
		validate,
	},
	AnonymousMessage,
	FromStream,
	Message,
	MessageError,
	Streamable,
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// impl<T> Streamable for AnonymousMessage<T>
// where
// 	T: Streamable + ?Sized,
// {
// 	const TYPE_HEADER: u16 = 0b1000_;
// 	fn content_as_stream(&self) -> &[u8] {
// 		let Self { written_on, contents }
// 	}
// }

const _: () = assert!(Connect::HEADER.uid() == 0b01);

impl Streamable for Connect {
	const HEADER: Header = Header::new_offset_uid(Offset::new_unsized(), Uid::new_lossy(0b01 << Uid::SHIFT));
	#[inline(always)]
	fn content_as_stream(&self) -> &[u8] { &[] }
}

impl FromStream for Connect {
	type Return = Self;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		let _ = validate::<Self>(&sliceref)?;
		Ok(Self)
	}
}

impl Streamable for Disconnect {
	const HEADER: Header = Header::new_offset_uid(Offset::new_unsized(), Uid::new_lossy(0b10 << Uid::SHIFT));
	#[inline(always)]
	fn content_as_stream(&self) -> &[u8] { &[] }
}

impl FromStream for Disconnect {
	type Return = Self;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		let _ = validate::<Self>(&sliceref)?;
		Ok(Self)
	}
}

impl Streamable for str {
	const HEADER: Header = Header::new_offset_uid(Offset::new_unsized(), Uid::new_lossy(0b00));

	#[inline(always)]
	fn content_as_stream(&self) -> &[u8] { self.as_bytes() }
}

impl FromStream for str {
	type Return = Box<Self>;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		// [202407281654+0200] NOTE(by: @OST-Gh): putain lifetimes.
		let stream = validate::<str>(&sliceref)?;
		let len = stream.len();
		let slice = str::from_utf8(&stream[3 .. len - 1])?;
		let ptr = (slice as *const str).cast_mut();
		Ok(unsafe { Box::from_raw(ptr) })
	}
}

impl<T> Streamable for T
where
	T: AsRef<str>,
{
	const HEADER: Header = <str as Streamable>::HEADER;

	#[inline(always)]
	fn content_as_stream(&self) -> &[u8] { <str as Streamable>::content_as_stream(self.as_ref()) }
}

impl<T> FromStream for T
where
	T: AsRef<str> + for<'a> From<&'a str>,
{
	type Return = Self;

	#[inline(always)]
	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		<str>::from_stream(sliceref).map(|boxs| T::from(&*boxs))
	}
}
