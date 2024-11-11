///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	hint::unreachable_unchecked,
	io::{self, BufReader, ErrorKind},
	net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
	time::Duration,
};

use either::{Either, Left, Right};

use crate::proto::Username;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Singelton struct for client-mode startup.
pub struct Client {
	receiver_stream: BufReader<TcpStream>,
	sender_stream: TcpStream,

	username: Username,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Partition the iterator based on whether a connection attempt succeeds or
/// not.
fn part(it: impl IntoIterator<Item = Ipv4Addr>, duration: Duration, port: u16) -> Either<Vec<Ipv4Addr>, Vec<TcpStream>> {
	let (possible, active): (Vec<_>, Vec<_>) = it
		.into_iter()
		.filter_map(gen_filter(duration, port))
		.partition(|either| either.is_left());
	if active.len() == 0 {
		let addrs = possible
			.into_iter()
			.map(|left| unsafe {
				left.left()
					.unwrap_unchecked()
			})
			.map(|sock| *sock.ip())
			.collect();
		Left(addrs)
	} else {
		let streams = active
			.into_iter()
			.map(|right| unsafe {
				right.right()
					.unwrap_unchecked()
			})
			.collect();
		Right(streams)
	}
}

/// Function that returns filter that filters connections based on a timeout.
fn gen_filter(dur: Duration, port: u16) -> impl FnMut(Ipv4Addr) -> Option<Either<SocketAddrV4, TcpStream>> {
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
pub fn open_connection<I, const TRYS: usize>(
	it: impl IntoIterator<Item = Ipv4Addr, IntoIter = I>,
	port: impl Into<u16>,
	self_addr: Ipv4Addr,
) -> Vec<TcpStream>
where
	I: Iterator<Item = Ipv4Addr>,
{
	let port = port.into();
	// [202408181606+0200] NOTE(by: @OST-Gh): DONT YOU FORGET ABOUT MEEEEE

	let mut dur_millis = 5;
	const DUR_FACTOR: u64 = 2;

	let mut it = match part(
		it.into_iter()
			.filter(|addr| *addr != self_addr),
		Duration::from_millis(1),
		port,
	) {
		Right(connections) => return connections,
		Left(address) => address.into_iter(),
	};

	for _ in 0 .. TRYS {
		match part(it, Duration::from_millis(dur_millis), port) {
			Right(connections) => return connections,
			Left(address) => it = address.into_iter(),
		}
		dur_millis *= DUR_FACTOR;
	}

	Vec::new()
}
