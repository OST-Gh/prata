///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::io::{stdout, BufWriter, Write};

use crate::{
	invoke::{Port, StartupOption},
	proto::{
		client::open_connection,
		server::host_on,
		spaces::{B16Ns192, IPv4Pns},
		stream::Streamable,
	},
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod invoke;
mod proto;
mod test;
mod util;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/* [202407161311+0200] TODO(by: @OST-Gh):
 *	move some stuffs from util into its
 *	own module.
 */
fn main() -> util::Result<()> {
	let mut out = BufWriter::new(stdout().lock());
	let start = StartupOption::default();
	let port = Port::default();
	let self_addr = util::local_v4ip()?;
	// [202407160951+0200] NOTE(by: @OST-Gh): current test code.
	for addr in IPv4Pns::from(B16Ns192).into_iter() {
		println!("{}", addr);
	}

	if start.as_server() {
		host_on(self_addr, port);
	} else if start.as_client() {
		let connections = open_connection(IPv4Pns::try_from(self_addr)?, port, self_addr);
	}

	"hello".as_stream();

	out.write_all(b"\0")?;
	out.flush()?;
	Ok(())
}
