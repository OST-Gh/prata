///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::str;

use crate::proto::{
	messages::{Connect, Disconnect},
	stream::{
		header::{Header, Offset, Uid},
		validate,
		FromStream,
		Headed,
		Streamable,
	},
	MessageError,
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
macro_rules! imp {
	($(<)? $type:ident $(>)? <- $offset:expr, $uid:expr => $as_stream:expr) => {
		impl $crate::proto::stream::Streamable for $type {
			#[inline(always)]
			fn header(&self) -> $crate::proto::stream::header::Header {
				<Self as $crate::proto::stream::Headed>::HEADER
			}

			#[inline(always)]
			fn content_as_stream(&self) -> &[::core::primitive::u8] { &[] }
		}

		impl $crate::proto::stream::Headed for $type {
			const HEADER: $crate::proto::stream::header::Header =
				$crate::proto::stream::header::Header::new_offset_uid($offset, $uid);
		}
	};
}
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

imp! { Connect <- Offset::new_unsized(), Uid::new_panicking(0b010 << Uid::SHIFT) => &[] }
impl FromStream for Connect {
	type Return = Self;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		let _ = validate::<Self>(&sliceref)?;
		Ok(Self)
	}
}

imp! { Disconnect <- Offset::new_unsized(), Uid::new_panicking(0b100 << Uid::SHIFT) => &[] }
impl FromStream for Disconnect {
	type Return = Self;

	fn from_stream(sliceref: impl AsRef<[u8]>) -> Result<Self::Return, MessageError> {
		let _ = validate::<Self>(&sliceref)?;
		Ok(Self)
	}
}

imp! { <str> <- Offset::new_unsized(), Uid::new_panicking(0b001 << Uid::SHIFT) => &[] }
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
	#[inline(always)]
	fn header(&self) -> Header { <Self as Headed>::HEADER }

	#[inline(always)]
	fn content_as_stream(&self) -> &[u8] { <str as Streamable>::content_as_stream(self.as_ref()) }
}

impl<T> Headed for T
where
	T: AsRef<str>,
{
	const HEADER: Header = <str as Headed>::HEADER;
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
