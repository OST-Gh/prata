///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#![allow(clippy::unusual_byte_groupings)]
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::proto::stream::Streamable;
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
#[derive(Default)]
#[repr(transparent)]
pub struct Uid(u8);

#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Debug)]
#[repr(transparent)]
pub struct Offset(u8);
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Header {
	pub const MASK: u8 = Uid::MASK | Offset::MASK;

	#[inline]
	const unsafe fn new_unchecked(unsanitised: u8) -> Option<Self> { Some(Self(unsanitised)) }

	#[inline]
	pub const fn new(unsanitised: u8) -> Self {
		let offset = Offset::new_lossy(unsanitised);
		let uid = Uid::new_lossy(unsanitised);
		Self::new_offset_uid(offset, uid)
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
	pub const fn is<T>(&self) -> bool
	where
		T: ?Sized + Streamable,
	{
		if !Uid::valid::<T>(None) {
			return false;
		}

		self.uid_raw() == T::HEADER.uid_raw()
	}
}

impl Uid {
	pub const MASK: u8 = 0b111_00000;
	pub const SHIFT: u8 = 5;

	#[inline(always)]
	pub const fn new_lossy(unsanitised: u8) -> Self {
		let sanitised = unsanitised & Self::MASK;
		unsafe { Self::new_unchecked(sanitised) }
	}

	#[inline(always)]
	const unsafe fn new_unchecked(uid: u8) -> Self { Self(uid >> Self::SHIFT) }

	#[inline]
	pub const fn new(unsanitised: u8) -> Option<Self> {
		if unsanitised & !Self::MASK != 0 {
			return None;
		}
		Some(Self::new_lossy(unsanitised))
	}

	#[inline(always)]
	pub const fn valid_by_val(uid: u8) -> bool { uid & !Self::MASK == 0 }

	#[inline(always)]
	pub const fn valid<T>(_: Option<&T>) -> bool
	where
		T: ?Sized + Streamable,
	{
		Self::valid_by_val(<T>::HEADER.uid_raw())
	}

	#[inline(always)]
	pub const fn to_u8(self) -> u8 { self.0 }

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
