///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::client::Client;
use chrono::{DateTime, Utc};
use lazy_regex::regex;
use std::{
	collections::HashMap,
	io::{self, BufRead, BufReader, Read, Write},
	net::{SocketAddr, TcpListener, TcpStream},
	str::FromStr,
	string::FromUtf8Error,
};
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
static mut ID: u16 = 0;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Username(String);

pub struct User {
	name: Username,
	id: u16, /* :IMUTABLE: */
	inbound_on: (BufReader<TcpStream>, SocketAddr),
}

struct AnonymousMessage {
	concern: Concern,
	written_on: DateTime<Utc>,
	contents: Box<str>,
}

pub struct Message {
	author: User,
	message: AnonymousMessage,
}

/// Network byte protocol stream.
pub struct Bytes {}

pub struct MessageBuffer {
	messages: HashMap<Username, Vec<AnonymousMessage>>,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub enum Concern {
	Everyone,
	SingleSpecific(User),
	Specific(Box<[User]>),
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

pub enum MessageError {
	/* [202407180813+0200] NOTE(by: @OST-Gh):
	 *	RET_ON: Other party has gracefully closed the stream.
	 */
	ConnectionClosed,
	/* [202407180814+0200] NOTE(by: @OST-Gh):
	 *	RET_ON: Other party hung up without prior notification.
	 */
	ConnectionInterrupted,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait Protocol {
	type Sender: Write;
	type Receiver: BufRead;

	fn get_sender(&mut self) -> &Self::Sender;
	fn get_sender_mut(&mut self) -> &mut Self::Sender;
	fn get_receiver(&self) -> &Self::Receiver;
	fn get_receiver_mut(&self) -> &mut Self::Receiver;

	fn send(&mut self, message: impl Into<Message>) -> Result<(), MessageError> {
		self.get_sender_mut()
			.write_all(message)
	}
	fn wait(&mut self) -> Result<Message, MessageError>;

	fn hang_up() -> Result<(), MessageError>;

	fn politely_hang_up(&mut self) -> Result<(), MessageError> {
		self.send()
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
		// [202407161557+0200] NOTE(by: @OST-Gh): send change-acknowledgement-response back to the enquirer.
		stream.get_mut()
			.write_all(&[0x06, 0x00])?;
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
			.read_until(0x00, &mut buf)?;
		let name = String::from_utf8(buf)?.parse::<Username>()?;

		Ok(Self {
			name,
			inbound_on,
			id,
		})
	}
}

impl From<(User, AnonymousMessage)> for Message {
	#[inline(always)]
	fn from((author, message): (User, AnonymousMessage)) -> Self {
		Self { author, message }
	}
}
