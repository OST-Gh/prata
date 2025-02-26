///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	io::{BufRead, BufReader, Read, Write},
	str::from_utf8,
};

use chrono::{DateTime, Local, Utc};

use crate::{Header, HeaderComponent, Identifier, Nickname, errors};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Message {
	// [202411162045+0100] NOTE(by: @OST-Gh): added in conversion to a byte stream.
	// [202411190014+0100] NOTE(by: @OST-Gh): shouldn't be read, just for serialisation.
	written_on: DateTime<Utc>,

	identifier: Identifier,
	nickname: Nickname,

	// [202411162045+0100] NOTE(by: @OST-Gh): \0 terminated.
	content: String,
}

impl Message {
	pub const MAX_LENGTH: usize = <usize>::MAX;
	pub const MIN_LENGTH: usize =
		size_of::<Header>() + size_of::<i64>() + Identifier::MIN_LENGTH + Nickname::MIN_LENGTH + size_of::<u8>();

	pub fn empty() -> Self {
		Self {
			written_on: DateTime::UNIX_EPOCH,
			identifier: Identifier::empty(),
			nickname: Nickname::default(),
			content: String::with_capacity(8),
		}
	}

	#[inline(always)]
	pub fn with_content(content: impl AsRef<str>) -> Self {
		let slice = content.as_ref();
		let len = slice.len();
		let mut content = String::with_capacity(len);
		unsafe {
			content.as_mut_ptr()
				.copy_from(slice.as_ptr(), len);
		}
		Self {
			written_on: DateTime::UNIX_EPOCH,
			identifier: Identifier::empty(),
			nickname: Nickname::default(),
			content,
		}
	}
	#[inline(always)]
	pub fn get_content(&self) -> &str { &self.content }
	#[inline(always)]
	pub fn get_content_mut(&mut self) -> &mut String { &mut self.content }
	#[inline(always)]
	pub fn set_content(&mut self, content: impl AsRef<str>) {
		let buffer = self.get_content_mut();
		buffer.clear();
		buffer.push_str(content.as_ref());
	}
	#[inline(always)]
	pub fn content(mut self, content: impl AsRef<str>) -> Self {
		self.set_content(content);
		self
	}

	#[inline(always)]
	pub fn with_identifier(identifier: Identifier) -> Self {
		Self {
			written_on: DateTime::UNIX_EPOCH,
			identifier,
			nickname: Nickname::default(),
			content: String::with_capacity(8),
		}
	}
	#[inline(always)]
	pub const fn get_identifier(&self) -> &Identifier { &self.identifier }
	#[inline(always)]
	pub fn set_identifier(&mut self, identifier: Identifier) { self.identifier = identifier; }
	#[inline(always)]
	pub fn identifier(mut self, identifier: Identifier) -> Self {
		self.set_identifier(identifier);
		self
	}

	#[inline(always)]
	pub fn with_nickname(nickname: Nickname) -> Self {
		Self {
			written_on: DateTime::UNIX_EPOCH,
			identifier: Identifier::empty(),
			nickname,
			content: String::with_capacity(8),
		}
	}
	#[inline(always)]
	pub const fn get_nickname(&self) -> &Nickname { &self.nickname }
	#[inline(always)]
	pub const fn get_nickname_mut(&mut self) -> &mut Nickname { &mut self.nickname }
	#[inline(always)]
	pub fn set_nickname(&mut self, nickname: Nickname) { self.nickname = nickname; }
	#[inline(always)]
	pub fn nickname(mut self, nickname: Nickname) -> Self {
		self.set_nickname(nickname);
		self
	}

	#[inline(always)]
	pub const fn get_utc(&self) -> DateTime<Utc> { self.written_on }
	#[inline(always)]
	pub fn get_local(&self) -> DateTime<Local> {
		self.get_utc()
			.with_timezone(&Local)
	}

	pub fn send(&self, to: &mut impl Write) -> Result<(), errors::MessageError> {
		let send_timestamp = Utc::now().timestamp();
		let identifier = self.get_identifier();
		to.write_all(&[self.as_header_component()])?;
		to.write_all(&send_timestamp.to_be_bytes())?;
		if identifier.is_unset() {
			Err(errors::MessageError::NoIdentifier)?
		} else {
			to.write_all(identifier.as_bytes())?;
		}
		to.write_all(
			self.get_nickname()
				.as_bytes(),
		)?;
		to.write_all(
			self.get_content()
				.as_bytes(),
		)?;
		to.write_all(b"\0")?;
		to.flush()?;
		Ok(())
	}

	pub fn recv(from: &mut impl Read) -> Result<Self, errors::MessageError> {
		let mut buf = Vec::with_capacity(Self::MIN_LENGTH);
		Self::recv_buf(from, &mut buf)
	}

	pub fn recv_buf(from: &mut impl Read, buf: &mut Vec<u8>) -> Result<Self, errors::MessageError> {
		let mut r = BufReader::new(from);
		r.read_until(b'\0', buf)?;
		Self::from_bytes(buf)
	}

	pub fn from_bytes(buf: impl AsRef<[u8]>) -> Result<Self, errors::MessageError> {
		let byteslice = buf.as_ref();
		let len = byteslice.len();
		if len < Self::MIN_LENGTH {
			Err(errors::MessageError::TooShort(len))?
		}
		let mut bytes = byteslice.iter();
		match byteslice
			.iter()
			.rposition(|b| b == &b'\0')
		{
			Some(i) if i < (size_of::<Header>() + size_of::<i64>()) =>
				Err(errors::MessageError::NoNull(byteslice.into()))?,
			None => Err(errors::MessageError::NoNull(byteslice.into()))?,
			Some(i) => {
				bytes.nth_back(len - i - 1);
			},
		}

		let header = Header::from(*unsafe {
			bytes.next()
				.unwrap_unchecked()
		});

		let mut timestamp_buffer = 0i64.to_be_bytes();
		let mut ptr = timestamp_buffer.as_mut_ptr();
		for b in bytes
			.by_ref()
			.take(size_of::<i64>())
		{
			unsafe {
				ptr.write(*b);
				ptr = ptr.add(1);
			}
		}

		let Some(written_on) = DateTime::from_timestamp(<i64>::from_be_bytes(timestamp_buffer), 0) else {
			Err(errors::MessageError::InvalidTimestamp)?
		};

		let data = header.get_bits::<Identifier>(None);
		let identifier_take = (data & 0b011) as usize;
		let mut identifier_buffer = 0u32.to_be_bytes();
		let mut ptr = if data & 0b100 == 0 {
			unsafe {
				identifier_buffer
					.as_mut_ptr()
					.add(identifier_take)
			}
		} else {
			identifier_buffer.as_mut_ptr()
		};
		for b in bytes
			.by_ref()
			.take(4 - identifier_take)
		{
			unsafe {
				ptr.write(*b);
				ptr = ptr.add(1);
			}
		}
		let identifier = Identifier(<u32>::from_be_bytes(identifier_buffer));

		let mut rest = from_utf8(bytes.as_slice())?.chars();

		let mut nickname = Nickname::default();
		nickname.extend(rest
			.by_ref()
			.take(header.get_bits::<Nickname>(None) as usize + 1));

		Ok(Self {
			written_on,
			identifier,
			nickname,
			content: rest.collect(),
		})
	}
}

impl AsRef<str> for Message {
	#[inline(always)]
	fn as_ref(&self) -> &str { self.get_content() }
}

impl AsRef<Identifier> for Message {
	#[inline(always)]
	fn as_ref(&self) -> &Identifier { self.get_identifier() }
}

impl AsRef<Nickname> for Message {
	#[inline(always)]
	fn as_ref(&self) -> &Nickname { self.get_nickname() }
}

impl AsMut<String> for Message {
	#[inline(always)]
	fn as_mut(&mut self) -> &mut String { self.get_content_mut() }
}

impl HeaderComponent for Message {
	const MASK: u8 = Identifier::MASK | Nickname::MASK;

	#[inline(always)]
	fn as_header_component(&self) -> u8 {
		Header::from_component(self.get_identifier())
			.set(self.get_nickname())
			.as_header_component()
	}
}
