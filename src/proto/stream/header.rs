///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#![allow(clippy::unusual_byte_groupings)]
use std::num::NonZero;

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::proto::stream::{Headed, Streamable};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// macro_rules! head {
// 	($module: ident $id: ident = $id_n: literal $(<< $sh: ident $(= $sh_n:
// expr)?)?) => { 		const _: () = {
// 			pub(in self::$module) const $id: core::primitive::u16 = (raw::$id as u16)
// $(<< offset::$sh)?;

// 			mod raw {
// 				pub(in super::$module::raw) const $id: core::primitive::u8 = $id_n;
// 			}
// 		};

// 		$($(const _: () = {
// 			pub(in super) const $sh: core::primitive::u8 = $sh_n;
// 		};)?)?
// 	};
// }
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod uid {}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Debug)]
#[derive(Default)]
#[repr(transparent)]
/// 0b0_00_00000
pub struct Header(u8);

#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Debug)]
#[repr(transparent)]
pub struct Uid(NonZero<u8>);

#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Debug)]
#[repr(transparent)]
pub struct Offset(u8);
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Header {
	pub const MASK: u8 = Uid::MASK | Offset::MASK;

	#[inline]
	const unsafe fn new_unchecked(unsanitised: u8) -> Self { Self(unsanitised) }

	#[inline]
	pub const fn new(unsanitised: u8) -> Option<Self> {
		let offset = Offset::new_lossy(unsanitised);
		let Some(uid) = Uid::new_lossy(unsanitised) else {
			return None
		};
		Some(Self::new_offset_uid(offset, uid))
	}

	#[inline(always)]
	pub const fn new_offset_uid(offset: Offset, uid: Uid) -> Self { Self(uid.to_raw_u8() | offset.to_u8()) }

	#[inline(always)]
	pub const fn has_two_elem(&self) -> bool { self.0 & Offset::MASK != 0 }

	#[inline(always)]
	pub const fn has_one_elem(&self) -> bool { self.0 & Offset::MASK == 0 }

	pub const fn to_u8(&self) -> u8 { self.offset() | self.uid_raw() }

	#[inline(always)]
	pub const fn offset(&self) -> u8 { self.0 & Offset::MASK }

	#[inline(always)]
	pub const fn uid_raw(&self) -> u8 { self.0 & Uid::MASK }

	#[inline(always)]
	pub const fn uid(&self) -> u8 { self.uid_raw() >> Uid::SHIFT }

	#[inline(always)]
	pub fn is<T>(&self) -> bool
	where
		T: ?Sized + Headed + Streamable,
	{
		if !Uid::valid::<T>(None) {
			return false;
		}

		self.uid_raw() == T::HEADER.uid_raw()
	}
}

impl Uid {
	pub const MASK: u8 = 0b111_00000;
	pub const SHIFT: u32 = Self::MASK.trailing_zeros();

	#[inline(always)]
	/// Create a new instance of `Uid`
	///
	/// Equivalent to masking the input to `new`
	pub const fn new_lossy(unsanitised: u8) -> Option<Self> {
		let partially_sanitised = unsanitised & Self::MASK;
		Self::new(partially_sanitised)
	}

	#[inline(always)]
	const unsafe fn new_unchecked(uid: u8) -> Self { Self(NonZero::new_unchecked(uid)) }

	#[inline]
	/// Create a new instance of `Uid`
	///
	/// Returns `None` if n, the input is zero.
	pub const fn new(unsanitised: u8) -> Option<Self> {
		if unsanitised & !Self::MASK != 0 {
			return None;
		}
		if unsanitised >> Self::SHIFT != 0 {
			return None
		}
		let sanitised = unsafe { Self::new_unchecked(unsanitised >> Self::SHIFT) };
		Some(sanitised)
	}

	#[inline]
	/// Create a new instance of `Uid`
	///
	/// Is equivalent to `new`, but instead panics when an error occurs.
	pub const fn new_panicking(unsanitised: u8) -> Self {
		if unsanitised & !Self::MASK != 0 {
			panic!("Input contains parts exclusive to other `Header` components.");
		}
		if unsanitised >> Self::SHIFT == 0 {
			panic!("Input cannot be equal to zero when aligned");
		}
		unsafe { Self::new_unchecked(unsanitised >> Self::SHIFT) }
	}

	#[inline(always)]
	pub const fn valid_by_val(uid: u8) -> bool { uid & !Self::MASK == 0 }

	#[inline(always)]
	pub fn valid<T>(_: Option<&T>) -> bool
	where
		T: ?Sized + Headed + Streamable,
	{
		Self::valid_by_val(<T>::HEADER.uid_raw())
	}

	#[inline(always)]
	pub const fn to_u8(self) -> u8 { self.0.get() }

	#[inline(always)]
	pub const fn to_raw_u8(self) -> u8 { self.to_u8() << Self::SHIFT }
}

impl Offset {
	pub const MASK: u8 = 0b000_11111;

	#[inline(always)]
	const unsafe fn new_unchecked(offset: u8) -> Self { Self(offset) }

	#[inline(always)]
	pub const fn new_lossy(unsanitised: u8) -> Self {
		let sanitised = unsanitised & Self::MASK;
		unsafe { Self::new_unchecked(sanitised) }
	}

	#[inline(always)]
	pub const fn new(unsanitised: u8) -> Option<Self> {
		if unsanitised & !Self::MASK != 0 {
			return None;
		}

		Some(Self::new_lossy(unsanitised))
	}

	#[inline(always)]
	pub const fn new_unsized() -> Self { unsafe { Self::new_unchecked(0b00000) } }

	#[inline(always)]
	pub const fn to_u8(self) -> u8 { self.0 }
}
