///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	io::{BufRead, BufReader, ErrorKind, Read},
	net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
	thread::JoinHandle,
	time::Duration,
};

use either::{Either, Left, Right};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use besked::{Identifier, Message,  Nickname};

use crate::proto::{spaces::Private, MessageLog};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
const TRIES: usize = 16;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Singelton struct for client-mode startup.
pub struct Client {
	stream: BufReader<TcpStream>,
	log: MessageLog,

	message_handler: JoinHandle<()>,

	buffer: Vec<u8>,

	nick: Nickname,
	id: Identifier,
}

pub struct ClientBuilder {
	stream: Option<BufReader<TcpStream>>,
	buffer: Vec<u8>,
	log: Message,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Partition the iterator based on whether a connection attempt succeeds or
/// not.
fn part(it: impl ParallelIterator<Item = Ipv4Addr>, duration: Duration, port: u16) -> Either<Vec<Ipv4Addr>, Vec<TcpStream>> {
	let (possible, active): (Vec<_>, Vec<_>) = it
		.filter_map(gen_filter(duration, port))
		.partition(|either| either.is_left());
	if active.is_empty() {
		let addrs = possible
			.into_iter()
			.map(|left| unsafe {
				left.left()
					.unwrap_unchecked()
			})
			.map(|sock| *sock.ip())
			.collect();
		Left(dbg![addrs])
	} else {
		let streams = active
			.into_iter()
			.map(|right| unsafe {
				right.right()
					.unwrap_unchecked()
			})
			.collect();
		Right(dbg![streams])
	}
}

/// Function that returns filter that filters connections based on a timeout.
fn gen_filter(dur: Duration, port: u16) -> impl Fn(Ipv4Addr) -> Option<Either<SocketAddrV4, TcpStream>> {
	move |addr| {
		let sock = SocketAddrV4::new(addr, port);
		match TcpStream::connect_timeout(&SocketAddr::V4(sock), dur).map_err(|iofault| iofault.kind()) {
			// [202408181621+0200] NOTE(by: @OST-Gh): might be better to throw a full error.
			Err(ErrorKind::Interrupted | ErrorKind::TimedOut | ErrorKind::WouldBlock) => Some(Left(sock)),
			// [202408181629+0200] (by: @): Is all you need, aaa, the city is calling. it's easier if you
			// don't look down . You make a fortune, don't look down now.
			Err(_other) => None,

			Ok(stream) => Some(Right(stream)),
		}
	}
}

/// Try to establish a connection to all open servers on a given port in the
/// current net.-space as a client.
pub fn find_from(self_addr: Ipv4Addr, port: impl Into<u16>) -> Vec<TcpStream> {
	let port = port.into();
	// [202408181606+0200] NOTE(by: @OST-Gh): DONT YOU FORGET ABOUT MEEEEE

	let mut dur_millis = 5;
	const DUR_FACTOR: u64 = 2;
	let it = Private::try_from(self_addr).unwrap();

	let mut it = match part(
		it.into_par_iter()
			.filter(|addr| *addr != self_addr),
		Duration::from_millis(1),
		port,
	) {
		Right(connections) => return connections,
		Left(address) => address.into_par_iter(),
	};

	for _ in 0 .. TRIES {
		match part(it, Duration::from_millis(dur_millis), port) {
			Right(connections) => return connections,
			Left(address) => it = address.into_par_iter(),
		}
		dur_millis *= DUR_FACTOR;
	}

	Vec::new()
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl TryFrom<TcpStream> for Client {
	type Error = ();

	fn try_from(value: TcpStream) -> Result<Self, Self::Error> {
		let mut buffer = Vec::with_capacity(Message::MIN_LENGTH);

		let mut stream = BufReader::new(value);

		let msg = Message::recv(&mut stream)?;

		let msg = Message::from_bytes(buffer.as_slice())?;
		let ident = *msg.get_identifier();

		Self {
			stream,
			handler,
			buffer,
			log,
			nick: 
		}
	}
}
