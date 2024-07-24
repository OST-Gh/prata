///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//! Functionality relating to server-client communications.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use chrono::{DateTime, Utc};
use client::Client;
use lazy_regex::regex;
use std::{
	alloc::{alloc, dealloc, Layout},
	collections::HashMap,
	io::{self, BufRead, BufReader, Read, Write},
	mem::align_of,
	net::{SocketAddr, TcpListener, TcpStream},
	slice,
	str::FromStr,
	string::FromUtf8Error,
};
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod client;
pub mod messages;
pub mod server;
pub mod spaces;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
static mut ID: u16 = 0;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Username(String);

pub struct User {
	name: Username,
	id: u16, /* :IMUTABLE: */
	inbound_on: (BufReader<TcpStream>, SocketAddr),
}

struct AnonymousMessage<T>
where
	T: Streamable,
{
	written_on: DateTime<Utc>,
	contents: T,
}

pub struct Message<T>
where
	T: Streamable,
{
	author: User,
	message: AnonymousMessage<T>,
}

#[derive(Debug)]
/// Network byte protocol stream.
pub struct Stream<'a> {
	stream: &'a [u8],
	ptr: *mut u8,
}

pub struct MessageBuffer {
	text_messages: HashMap<Username, Vec<AnonymousMessage<Box<str>>>>,
	connections: HashMap<
		Username,
		(
			AnonymousMessage<messages::Connect>,
			Option<AnonymousMessage<messages::Disconnect>>,
		),
	>,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
pub enum StreamSize {
	Representable(u8),
	Unsized,
}

#[derive(Debug, Error)]
pub enum FromTransmissionError {
	#[error("{0}")]
	IO(#[from] io::Error),
	#[error("{0}")]
	UTF8(#[from] FromUtf8Error),
	#[error("{0}")]
	Username(#[from] UsernameFromStrError),
}

#[derive(Debug, Error)]
pub enum UsernameFromStrError {
	#[error(r"`{0}` doesn't match `^@?[[:alnum][:punct:]--[\{{\}}\(\)\[\]@]]`")]
	NoMatch(Box<str>),
}

#[derive(Debug, Error)]
pub enum MessageError {
	#[error("The connection has been hung up.")]
	/* [202407180813+0200] NOTE(by: @OST-Gh):
	 *	RET_ON: Other party has gracefully closed the stream.
	 */
	ConnectionClosed,
	#[error("The connection has been dropped.")]
	/* [202407180814+0200] NOTE(by: @OST-Gh):
	 *	RET_ON: Other party hung up without prior notification.
	 */
	ConnectionInterrupted,

	#[error("{0}")]
	IO(#[from] io::Error),
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait Streamable {
	/// Three UID bits (b_m = 0b1110_0000)
	const TYPE_HEADER: u8;
	/// Five bits (b_m = 0b0001_1111)
	const SIZE_HEADER: StreamSize;

	fn content_as_stream(&self) -> &[u8];
	fn as_stream(&self) -> Stream<'_> {
		let content = self.content_as_stream();
		let content_length = content.len();
		let has_null = content.ends_with(&[0b0000_0000]);
		let additional = unsafe {
			Layout::from_size_align_unchecked(
				1 + (!has_null as usize),
				align_of::<u8>(),
			)
		};
		let (size, _offset) = unsafe {
			Layout::for_value(content)
				.extend(additional)
				.unwrap_unchecked()
		};

		let ptr = unsafe { alloc(size) };
		let header = Self::TYPE_HEADER + Self::SIZE_HEADER.to_byte();
		unsafe { ptr.write(header) };
		for i in 0..content_length {
			unsafe {
				ptr.add(i + 1)
					.write(*content.get_unchecked(i))
			}
		}
		if has_null {
			unsafe {
				ptr.add(content_length + 1)
					.write(0b0000_0000)
			};
		}
		// [202407241911+0200] NOTE(by: @OST-Gh): leak?
		Stream::from_slice_ptr(unsafe { slice::from_raw_parts(ptr, size.size()) }, ptr)
	}
}

/// Protocol.
///
/// The first transaction that will be sent from the Client is always the username.
pub trait Protocol {
	type Sender: Write;
	type Receiver: BufRead;

	fn get_username(&self) -> &str;
	fn get_sender(&mut self) -> &Self::Sender;
	fn get_sender_mut(&mut self) -> &mut Self::Sender;
	fn get_receiver(&self) -> &Self::Receiver;
	fn get_receiver_mut(&self) -> &mut Self::Receiver;

	fn send(&mut self, message: impl Into<Message>) -> Result<(), MessageError> {
		self.get_sender_mut()
			.write_all(
				message.into()
					.as_stream(),
			)?;
		Ok(())
	}
	fn register(&mut self) -> Result<(), MessageError> {
		let user_b = String::from(self.get_username()).into_bytes();
		self.get_sender_mut()
			.write_all(user_b.as_slice())?;
		Ok(())
	}

	fn wait(&mut self) -> Result<Message, MessageError>;

	fn hang_up() -> Result<(), MessageError>;

	fn politely_hang_up(&mut self) -> Result<(), MessageError> {
		self.send(messages::Disconnect)
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn get_id() -> u16 {
	let prev = unsafe { ID };
	unsafe { ID += 1 }
	prev
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Username {
	/// Forcibly change the username without any checks for duplicates.
	pub fn change_username_unchecked(
		&mut self,
		mut stream: Client,
	) -> Result<(), FromTransmissionError> {
		// [202407180827+0200] TODO(by: @OST-Gh): reimplement.
		let mut buf = Vec::with_capacity(2);
		stream.read_until(0x00, &mut buf)?;
		let new = String::from_utf8(buf)?.parse::<Username>()?;
		*self = new;
		Ok(())
	}
}

impl FromStr for Username {
	type Err = UsernameFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let Some(cap) =
			regex!(r"^@?(?<name>[[:alnum][:punct:]--[\{\}\(\)\[\]@]])").captures(s)
		else {
			Err(UsernameFromStrError::NoMatch(s.into()))?
		};
		let Some(valid) = cap.name("name") else {
			Err(UsernameFromStrError::NoMatch(s.into()))?
		};

		Ok(Username(String::from(valid.as_str())))
	}
}

impl TryFrom<TcpListener> for User {
	type Error = FromTransmissionError;

	fn try_from(listener: TcpListener) -> Result<Self, Self::Error> {
		listener.set_nonblocking(false)?;
		let id = get_id();
		let res = listener.accept();

		let mut inbound_on = match res {
			Ok((socket, addr)) => (BufReader::new(socket), addr),
			Err(err) => Err(Self::Error::IO(err))?,
		};

		let mut buf = Vec::with_capacity(2);

		inbound_on
			.0
			.read_until(0b0000_0000, &mut buf)?;
		let name = String::from_utf8(buf)?.parse::<Username>()?;

		Ok(Self {
			name,
			inbound_on,
			id,
		})
	}
}

impl<T> From<(User, AnonymousMessage<T>)> for Message<T>
where
	T: Streamable,
{
	#[inline(always)]
	fn from((author, message): (User, AnonymousMessage<T>)) -> Self {
		Self { author, message }
	}
}

impl<'a> Stream<'a> {
	fn from_slice_ptr(stream: &'a [u8], ptr: *mut u8) -> Self {
		Self { stream, ptr }
	}
}

impl<'a> From<(&'a [u8], *mut u8)> for Stream<'a> {
	#[inline]
	fn from((stream, ptr): (&'a [u8], *mut u8)) -> Self {
		Self::from_slice_ptr(stream, ptr)
	}
}

impl<'a> Drop for Stream<'a> {
	fn drop(&mut self) {
		unsafe { dealloc(self.ptr, Layout::for_value(self.stream)) };
	}
}

impl StreamSize {
	const UNSIZED_SIGNATURE: u8 = 0b0000_0000;
	const MASK: u8 = 0b0001_1111;

	const fn from_byte(byte: u8) -> Option<Self> {
		if byte & !Self::MASK > Self::MASK {
			return None;
		}
		Some(Self::from_byte_lossy(byte))
	}

	const fn from_byte_lossy(byte: u8) -> Self {
		if byte == Self::UNSIZED_SIGNATURE {
			return Self::Unsized;
		}
		Self::Representable(byte & Self::MASK)
	}

	const fn to_byte(self) -> u8 {
		*self.as_byte_ref()
	}

	const fn as_byte_ref(&self) -> &u8 {
		match self {
			Self::Unsized => &Self::UNSIZED_SIGNATURE,
			// [202407190914+0200] NOTE(by: @OST-Gh): assume valid.
			Self::Representable(x) => x,
		}
	}

	/// Access the mutable reference of a possibly underlying byte.
	///
	/// # Safety
	///
	/// It is inherintely unsafe to hand full control over a structure to the developer: The guarantees of the data-structures cannot be ensured anymore.
	unsafe fn as_byte_ref_mut(&mut self) -> Option<&mut u8> {
		match self {
			Self::Unsized => None,
			Self::Representable(x) => Some(x),
		}
	}
}

impl AsRef<u8> for StreamSize {
	#[inline(always)]
	fn as_ref(&self) -> &u8 {
		match self {
			Self::Unsized => &Self::UNSIZED_SIGNATURE,
			// [202407190914+0200] NOTE(by: @OST-Gh): assume valid.
			Self::Representable(x) => x,
		}
	}
}
