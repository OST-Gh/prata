///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::net::{Ipv4Addr, TcpStream};
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
