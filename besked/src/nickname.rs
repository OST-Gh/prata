///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	alloc::{Layout, alloc, dealloc, realloc},
	fmt::{self, Debug, Display, Formatter},
	slice::from_raw_parts,
	str::{FromStr, from_utf8, from_utf8_unchecked},
};

use crate::{FromHeader, Header, HeaderComponent, errors::NicknameError};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// A buffer for UTF-8 encoded usernicknames.
///
/// Each nickname cannot be [`MAX_GLYPH_COUNT`] codepoints long.
///
/// [`MAX_GLYPH_COUNT`]: Nickname::MAX_GLYPH_COUNT
pub struct Nickname {
	buffer: *mut u8,
	length: u8,
	allocated: u8,
	glyph_count: u8,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
fn char_from_utf8_ptr_and_size(utf8: *const u8, size: usize) -> char {
	unsafe {
		// [202411182123+0100] TODO(by: @OST-Gh): find a smarter way?

		// from_utf8_unchecked(
		from_utf8_unchecked(from_raw_parts(utf8, size))
			.chars()
			.next()
			.unwrap_unchecked()
	}
}

#[inline(always)]
fn get_bearings(origin: *mut u8, offset: usize) -> (usize, usize, *mut u8) {
	unsafe {
		let mut start = origin.add(offset);
		for i in 0 .. 4 {
			start = start.sub(i);
			if start.read() & 0xC0 != 0x80 {
				break
			}
		}
		let size = ((start.read() & 0xF0).leading_ones() as usize).max(1);
		(
			size,
			origin.offset_from(start)
				.unsigned_abs(),
			start,
		)
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Nickname {
	pub const MAX_LENGTH: usize = 4 * Self::MAX_GLYPH_COUNT;
	pub const MIN_LENGTH: usize = 1;

	pub const MAX_GLYPH_COUNT: usize = Self::POSSEBILITIES as usize;

	const DEFAULT_CAPACITY: u8 = 8;

	#[inline(always)]
	pub const fn capacity(&self) -> usize { self.allocated as usize }
	#[inline(always)]
	pub const fn available_bytes(&self) -> usize { self.len() - self.capacity() }
	#[inline(always)]
	pub const fn len(&self) -> usize { self.length as usize }
	#[inline(always)]
	pub const fn glyph_count(&self) -> usize { self.glyph_count as usize }

	#[inline(always)]
	pub const fn is_empty(&self) -> bool { self.len() == 0 }

	pub fn with_capacity(capacity: u8) -> Option<Self> {
		let max_length = Self::MAX_LENGTH as u8;
		if capacity > max_length {
			None?
		}
		Some(unsafe { Self::with_capacity_unchecked(max_length.min(capacity << 2)) })
	}

	/// Allocate a buffer with no checks.
	///
	/// # Safety
	///
	/// Due to the nature of the buffer, its maximum allocation-size is 128
	/// bytes. Though, the added `char`'s cannot exceed 32.
	/// This function also only allocates the exact amount of bytes
	/// specified.
	pub unsafe fn with_capacity_unchecked(capacity: u8) -> Self {
		Self {
			buffer: unsafe { alloc(Layout::array::<u8>(capacity as usize).unwrap_unchecked()) },
			length: 0,
			glyph_count: 0,
			allocated: capacity,
		}
	}

	#[inline(always)]
	pub fn new() -> Self { unsafe { Self::with_capacity_unchecked(0) } }

	#[inline]
	pub const fn as_ptr(&self) -> *const u8 {
		self.buffer
			.cast_const()
	}
	pub fn as_mut_ptr(&mut self) -> *mut u8 { self.buffer }

	#[inline]
	pub fn to_ptr(self) -> *const u8 {
		self.to_mut_ptr()
			.cast_const()
	}
	pub fn to_mut_ptr(self) -> *mut u8 { self.buffer }

	#[inline(always)]
	pub const fn as_bytes(&self) -> &[u8] {
		unsafe {
			from_raw_parts(
				self.buffer
					.cast_const(),
				self.len(),
			)
		}
	}

	#[inline(always)]
	pub const fn as_str(&self) -> &str { unsafe { from_utf8_unchecked(self.as_bytes()) } }

	#[inline]
	pub fn reserve_exact(&mut self, additional: u8) {
		let additional = additional as usize;
		self.realloc_if_not(false, Self::MAX_LENGTH.min(self.capacity() + additional));
		// let Some(buffer) = NonNull::new(unchecked) else {
		// 	// [202411162024+0100] NOTE(by: @OST-Gh):
		// 	// This is bad
		// 	// SAFETY:
		// 	// At this point the pointer's
		// 	return 0;
		// };
	}

	#[inline]
	pub fn reserve(&mut self, additional: u8) {
		let additional = additional as usize;
		if self.len() < self.capacity() {
			return;
		}
		self.realloc_if_not(false, Self::MAX_LENGTH.min((self.capacity() + additional) << 1));
	}

	#[inline(always)]
	fn realloc_if_not(&mut self, cond: bool, new_capacity: usize) {
		if new_capacity >= Self::MAX_LENGTH || cond {
			return;
		}

		unsafe {
			self.buffer = realloc(
				self.as_mut_ptr(),
				Layout::array::<u8>(self.capacity()).unwrap_unchecked(),
				new_capacity,
			);
		}
	}

	pub fn shrink_to(&mut self, new_capacity: u8) {
		let new_capacity = new_capacity as usize;
		self.realloc_if_not(new_capacity > self.capacity() || new_capacity < self.len(), new_capacity)
	}

	pub fn shrink_to_fit(&mut self) {
		let length = self.len();
		self.realloc_if_not(length == self.capacity(), length)
	}

	pub fn push(&mut self, glyph: char) -> bool {
		let len = glyph.len_utf8();
		let new_len = len + self.len();

		if new_len > Self::MAX_LENGTH || self.glyph_count() >= Self::MAX_GLYPH_COUNT {
			return false;
		}
		if new_len > self.capacity() {
			self.reserve(new_len as u8);
		}

		let mut utf8 = [0; 4];

		glyph.encode_utf8(&mut utf8);
		self.glyph_count += 1;

		unsafe {
			self.as_mut_ptr()
				.add(self.len())
				.copy_from(utf8.as_ptr(), len)
		}

		self.length = new_len as u8;

		true
	}

	#[inline(always)]
	pub fn pop(&mut self) -> Option<char> {
		if self.is_empty() {
			None?;
		}
		let (size, _, overwrite) = get_bearings(self.as_mut_ptr(), self.len() - 1);

		self.length -= size as u8;
		self.glyph_count -= 1;

		Some(char_from_utf8_ptr_and_size(overwrite, size))
	}

	pub fn clear(&mut self) {
		self.length = 0;
		self.glyph_count = 0;
	}

	pub fn insert(&mut self, index: u8, glyph: char) -> bool {
		let index_hint = index as usize;
		let len = glyph.len_utf8();
		let new_len = len + self.len();

		if new_len > Self::MAX_LENGTH || self.glyph_count() >= Self::MAX_GLYPH_COUNT || index_hint >= self.len() {
			return false;
		}
		if new_len > self.capacity() {
			self.reserve(new_len as u8);
		}

		let (_, index, overwrite) = get_bearings(self.as_mut_ptr(), index_hint);
		let copy_amount = self.len() - index;
		let mut utf8 = [0; 4];

		glyph.encode_utf8(&mut utf8);

		unsafe {
			overwrite.copy_to(overwrite.add(len), copy_amount);
			overwrite.copy_from(utf8.as_ptr(), len);
		}

		self.length = new_len as u8;
		self.glyph_count += 1;

		true
	}

	pub fn remove(&mut self, index: u8) -> Option<char> {
		let index_hint = index as usize;

		if index_hint >= self.len() {
			None?;
		}

		let (size, index, overwrite) = get_bearings(self.as_mut_ptr(), index_hint);
		let copy_amount = self.len() - index;
		let residual = char_from_utf8_ptr_and_size(overwrite, size);

		unsafe {
			overwrite.copy_from(overwrite.add(size), copy_amount);
		}

		self.length -= size as u8;
		self.glyph_count -= 1;

		Some(residual)
	}

	pub fn replace(&mut self, index: u8, glyph: char) -> Option<char> {
		let index_hint = index as usize;
		let len = glyph.len_utf8();

		if index_hint >= self.len() {
			None?;
		}

		let (size, index, overwrite) = get_bearings(self.as_mut_ptr(), index_hint);
		let copy_amount = self.len() - index;
		let residual = char_from_utf8_ptr_and_size(overwrite, size);
		let mut utf8 = [0; 4];

		self.glyph_count -= 1;
		self.length -= size as u8;
		self.length += len as u8;
		if self.len() > self.capacity() {
			self.reserve(self.len() as u8);
		}
		glyph.encode_utf8(&mut utf8);

		unsafe {
			overwrite.copy_from(utf8.as_ptr(), len);
			overwrite
				.add(len)
				.copy_from(overwrite.add(size), copy_amount);
		}
		Some(residual)
	}

	/// Copy bytes from the source to a new instance.
	///
	/// Prefer [`Self::from_str`].
	///
	/// # Safety
	///
	/// A buffer can only keep [`Self::MAX_LENGTH`] bytes.
	pub unsafe fn from_str_unchecked(utf8: impl AsRef<str>, glyph_count: u8) -> Self {
		let s = utf8.as_ref();
		let len = s.len() as u8;

		let mut instance: Self;
		unsafe {
			instance = Self::with_capacity_unchecked(len);
			instance.buffer
				.copy_from(s.as_ptr(), s.len());
		}
		instance.glyph_count = glyph_count;
		instance.length = len;

		instance
	}
}

impl Extend<char> for Nickname {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = char>,
	{
		for c in iter
			.into_iter()
			.take(Self::MAX_GLYPH_COUNT - self.glyph_count())
		{
			self.push(c);
		}
	}
}

impl FromIterator<char> for Nickname {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = char>,
	{
		let iter = iter.into_iter();
		let (min, max_opt) = iter.size_hint();
		let mut instance =
			unsafe { Self::with_capacity_unchecked((max_opt.unwrap_or(min)).min(Self::MAX_GLYPH_COUNT) as u8) };
		instance.extend(iter);
		instance
	}
}

impl HeaderComponent for Nickname {
	const MASK: u8 = 0b11111;

	#[inline(always)]
	fn as_header_component(&self) -> u8 {
		if self.is_empty() {
			panic!("Cannot serialise an empty `{}`", stringify!(Nickname));
		}
		self.glyph_count() as u8 - 1
	}
}

impl FromHeader for Nickname {
	type Error = NicknameError;

	fn from_header(header: &Header, buf: impl AsRef<[u8]>) -> Result<Self, Self::Error> {
		let take = header.get_bits::<Self>(None) as usize + 1;

		let text = from_utf8(buf.as_ref())?;
		let glyph_count = text
			.chars()
			.count();

		if glyph_count != take {
			Err(Self::Error::TooManyGlyphs(text.into(), glyph_count))?;
		}

		Ok(unsafe { Self::from_str_unchecked(text, glyph_count as u8) })
	}
}

impl Display for Nickname {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			f.write_str("@")?;
		}
		f.write_str(self.as_str())?;
		Ok(())
	}
}

impl Debug for Nickname {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			f.debug_struct(stringify!(UserNickname))
				.field("codepoint_amount", &self.glyph_count())
				.field("length", &self.len())
				.field("capacity", &self.capacity())
				.field("buffer_addr", &self.as_ptr())
				.field("utf8_repr", &self.as_bytes())
				.field("text_repr", &self.as_str())
				.finish()
		} else {
			f.debug_struct(stringify!(UserNickname))
				.field("codepoint_amount", &self.glyph_count())
				.field("length", &self.len())
				.field("capacity", &self.capacity())
				.field("buffer_addr", &self.as_ptr())
				.finish()
		}
	}
}

impl FromStr for Nickname {
	type Err = NicknameError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let glyph_count = s
			.chars()
			.count();
		if glyph_count > Self::MAX_GLYPH_COUNT {
			Err(Self::Err::TooManyGlyphs(s.into(), glyph_count))?
		}

		Ok(unsafe { Self::from_str_unchecked(s, glyph_count as u8) })
	}
}

unsafe impl Sync for Nickname {}
unsafe impl Send for Nickname {}

impl AsRef<[u8]> for Nickname {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] { self.as_bytes() }
}

impl AsRef<str> for Nickname {
	#[inline(always)]
	fn as_ref(&self) -> &str { self.as_str() }
}

impl Drop for Nickname {
	#[inline(always)]
	fn drop(&mut self) {
		unsafe {
			self.as_mut_ptr()
				.write_bytes(0, self.capacity());
			dealloc(self.as_mut_ptr(), Layout::array::<u8>(self.capacity()).unwrap_unchecked());
		}
	}
}

impl Default for Nickname {
	#[inline(always)]
	fn default() -> Self { unsafe { Self::with_capacity(Self::DEFAULT_CAPACITY).unwrap_unchecked() } }
}
