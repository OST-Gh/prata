///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::protocol::Username;
use std::{
	io::BufReader,
	net::{Ipv4Addr, TcpStream},
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Singelton struct for client-mode startup.
pub struct Client {
	receiver_stream: BufReader<TcpStream>,
	sender_stream: TcpStream,

	username: Username,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Try to establish a connection to all open servers on a given port in the current net.-space as a client.
pub fn try_connect_all<I>(
	it: impl IntoIterator<Item = Ipv4Addr, IntoIter = I>,
	port: impl Into<u16>,
) -> Vec<TcpStream>
where
	I: Iterator<Item = Ipv4Addr>,
{
	let port = port.into();
	it.into_iter()
		.filter_map(|addr| TcpStream::connect((addr, port)).ok())
		.collect()
}
