//! Functionality relating to server-client communications.

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{fmt::Debug, slice::from_raw_parts};

pub use message::Message;
pub use nickname::Nickname;
use parking_lot::Mutex;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod errors;
mod message;
mod nickname;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
static CURRENT_IDENTIFIER: Mutex<u32> = Mutex::new(1);
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[repr(transparent)]
#[derive(Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Debug)]
#[derive(Default)]
// 0b	[000	] _ [00000	]
// 	{length	}   {u.n.-chars	}
pub struct Header(u8);

#[repr(transparent)]
#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct Identifier(u32);
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Protocol.
///
/// The first transaction that will be sent from the Client is always the
/// username.
pub trait HeaderComponent
where
	Self: Sized,
{
	const MASK: u8;
	const POSSEBILITIES: u8 = 2u8.pow(Self::MASK.count_ones());
	const SHIFT: u8 = Self::MASK.trailing_zeros() as u8;

	fn as_header_component(&self) -> u8;
}

pub trait FromHeader
where
	Self: Sized,
{
	type Error: Debug;

	fn from_header(header: &Header, buf: impl AsRef<[u8]>) -> Result<Self, Self::Error>;
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl<T> HeaderComponent for &T
where
	T: HeaderComponent,
{
	const MASK: u8 = T::MASK;

	#[inline(always)]
	fn as_header_component(&self) -> u8 { T::as_header_component(self) }
}

impl Header {
	pub fn set_bits<T>(&mut self, component: T)
	where
		T: HeaderComponent,
	{
		// [202411150011+0100] NOTE(by: @OST-GH):
		// Reset changing-bits.
		self.0 &= !T::MASK;
		// Set them.
		self.0 |= component.as_header_component() << T::SHIFT;
	}

	pub fn set(mut self, component: impl HeaderComponent) -> Self {
		self.set_bits(component);
		self
	}

	pub fn get_bits<T>(&self, _: Option<T>) -> u8
	where
		T: HeaderComponent,
	{
		(self.0 & T::MASK) >> T::SHIFT
	}

	pub fn from_component(component: impl HeaderComponent) -> Self { Self::default().set(component) }
	pub const fn from_byte(byte: u8) -> Self { Self(byte) }

	#[inline]
	pub fn to_component<T>(&self, bytes: impl AsRef<[u8]>) -> Result<T, T::Error>
	where
		T: FromHeader,
	{
		T::from_header(self, bytes)
	}

	// pub fn get_content_mut(&mut self) -> &mut String { &mut self.content }
	// pub fn get_content(&self) -> &str { &self.content }
	// pub fn set_content(&mut self, content: String) { self.content = content }
}

impl HeaderComponent for Header {
	const MASK: u8 = 0b11111111;

	#[inline(always)]
	fn as_header_component(&self) -> u8 { self.0 }
}

impl From<u8> for Header {
	#[inline(always)]
	fn from(byte: u8) -> Self { Self(byte) }
}

impl Identifier {
	pub const MIN_LENGTH: usize = 1;
	pub const MAX_LENGTH: usize = 4;

	#[inline(always)]
	pub fn new() -> Self { Self::default() }

	#[inline(always)]
	pub const fn empty() -> Self { Self(0) }

	#[inline(always)]
	pub const fn is_unset(&self) -> bool { self.0 == 0 }

	#[inline(always)]
	pub const fn as_bytes(&self) -> &[u8] {
		let origin = (&raw const self.0).cast::<u8>();

		let lead =
			self.0.leading_zeros();
		let trail =
			self.0.trailing_zeros();
		let (offset, zeroes) = if lead >= trail {
			let lead = lead as usize;
			(unsafe { origin.add(lead) }, lead)
		} else {
			(origin, trail as usize)
		};

		unsafe { from_raw_parts(offset, size_of::<u32>() - zeroes) }
	}
}

impl HeaderComponent for Identifier {
	const MASK: u8 = 0b11100000;

	#[inline(always)]
	fn as_header_component(&self) -> u8 {
		let l =
			self.0.leading_zeros();
		let r =
			self.0.trailing_zeros();
		if l >= r { l as u8 / 8 } else { 0b100 | (r as u8 / 8) }
		// println!("l{l} r{r}  {res:0>8b}");
	}
}

impl FromHeader for Identifier {
	type Error = errors::IdentifierError;

	fn from_header(header: &Header, buf: impl AsRef<[u8]>) -> Result<Self, Self::Error> {
		let data = header.get_bits::<Self>(None);
		let take = (data & 0b011) as usize;

		let bytes = buf.as_ref();

		let len = bytes.len();
		let expected = 4 - take;
		if len != expected {
			Err(Self::Error::LengthMismatch(len, expected))?;
		}

		let mut be = 0u32.to_be_bytes();
		let be_ptr = be.as_mut_ptr();
		let len = bytes.len();

		if data & 0b100 == 0 {
			unsafe {
				be_ptr.add(take)
					.copy_from(bytes.as_ptr(), len);
			}
		} else {
			unsafe {
				be_ptr.copy_from(bytes.as_ptr(), len);
			}
		}

		Ok(Self(<u32>::from_be_bytes(be)))
	}
}

impl Default for Identifier {
	fn default() -> Self {
		let mut identifier = CURRENT_IDENTIFIER.lock();
		let instance = Self(*identifier);
		*identifier += 1;
		instance
	}
}
