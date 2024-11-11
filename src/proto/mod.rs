///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//! Functionality relating to server-client communications.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	collections::HashMap,
	io::{self, BufRead, BufReader, Write},
	net::{SocketAddr, TcpListener, TcpStream},
	str::{FromStr, Utf8Error},
	string::FromUtf8Error,
};

use chrono::{DateTime, Utc};
use client::Client;
use lazy_regex::regex;
use server::UserId;
use stream::{FromStream, Streamable};
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub mod client;
pub mod messages;
pub mod server;
pub mod spaces;
pub mod stream;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
const MIN_BYTES: usize = 2;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Username(String);

pub struct User {
	name: Username,

	inbound_on: (BufReader<TcpStream>, SocketAddr),
}

struct AnonymousMessage<T>
where
	T: Streamable + ?Sized,
{
	written_on: DateTime<Utc>,
	contents: T,
}

pub struct Message<T>
where
	T: Streamable + ?Sized,
{
	author: UserId,
	message: AnonymousMessage<T>,
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
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum UsernameFromStrError {
	#[error(r"`{0}` doesn't match `^@?[[:alnum:][:punct:]--[\{{\}}\(\)\[\]@]]{{1,31}}`")]
	NoMatch(Box<str>),
	#[error(r"`{0}` is `{1}` bytes long, but it shouldn't exceed `31`.")]
	TooLong(Box<str>, usize),
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

	#[error("Waiting for a message timed out.")]
	Timeout,

	#[error("Cannot parse to `{1}` from `{0}`.")]
	UIDMismatch(u8, u8),
	#[error("A uid cannot be equal to zero")]
	UIDIsZero,
	#[error(
		"A message cannot be less than {min} bytes, but a stream with {0} bytes was provided.",
		min = MIN_BYTES,
	)]
	TooShort(usize),

	#[error("A message's stream `{0:#?}` isn't null-terminated.")]
	NoNull(Box<[u8]>),

	#[error("Some error in the header occurred.")]
	HeaderMissmatch,

	#[error("{0}")]
	IO(#[from] io::Error),
	#[error("{0}")]
	OwnedUTF8(#[from] FromUtf8Error),
	#[error("{0}")]
	UTF8(#[from] Utf8Error),
	#[error("{0}")]
	Username(#[from] UsernameFromStrError),
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Protocol.
///
/// The first transaction that will be sent from the Client is always the
/// username.
pub trait Protocol {
	type Sender: Write;
	type Receiver: BufRead;

	fn get_username(&self) -> &str;
	fn get_sender(&mut self) -> &Self::Sender;
	fn get_sender_mut(&mut self) -> &mut Self::Sender;
	fn get_receiver(&self) -> &Self::Receiver;
	fn get_receiver_mut(&mut self) -> &mut Self::Receiver;

	fn hang_up(&mut self) -> Result<(), MessageError>;

	fn transmit(&mut self, message: impl Streamable) -> Result<(), MessageError> {
		self.get_sender_mut()
			.write_all(
				message.as_stream()
					.as_ref(),
			)?;
		Ok(())
	}
	fn register(&mut self) -> Result<(), MessageError> {
		let user_b = String::from(self.get_username()).into_bytes();
		self.get_sender_mut()
			.write_all(user_b.as_slice())?;
		Ok(())
	}

	fn wait<T>(&mut self) -> Result<<T as FromStream>::Return, MessageError>
	where
		T: FromStream,
	{
		let mut buf = Vec::with_capacity(2);
		self.get_receiver_mut()
			.read_until(b'\0', &mut buf)?;
		T::from_stream(&buf)
	}

	fn politely_hang_up(&mut self) -> Result<(), MessageError> {
		self.transmit(messages::Disconnect)?;
		self.hang_up()?;
		Ok(())
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Username {
	const MAX_LEN: usize = 31;

	/// Forcibly change the username without any checks for duplicates.
	pub fn change_username_unchecked(&mut self, mut stream: Client) -> Result<(), MessageError> {
		// [202407180827+0200] TODO(by: @OST-Gh): reimplement.
		// let mut buf = Vec::with_capacity(2);
		// stream.read_until(0x00, &mut buf)?;
		// let new = String::from_utf8(buf)?.parse::<Username>()?;
		// *self = new;
		todo!();
	}
}

impl FromStr for Username {
	type Err = UsernameFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let Some(cap) = regex!(r"^@?(?<name>[[:alnum][:punct:]--[\{\}\(\)\[\]@]])").captures(s) else {
			Err(UsernameFromStrError::NoMatch(s.into()))?
		};
		let Some(valid) = cap.name("name") else {
			Err(Self::Err::NoMatch(s.into()))?
		};

		let usr = valid.as_str();
		let len = usr.len();

		if len > Self::MAX_LEN {
			Err(Self::Err::TooLong(usr.into(), len))?
		}

		Ok(Username(String::from(usr)))
	}
}

impl<T> From<(UserId, AnonymousMessage<T>)> for Message<T>
where
	T: Streamable,
{
	#[inline(always)]
	fn from((author, message): (UserId, AnonymousMessage<T>)) -> Self { Self { author, message } }
}
