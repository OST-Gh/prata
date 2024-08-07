///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::io::{stdout, BufWriter, Write};

use proto::spaces::IPv4ns;

use crate::{
	invoke::{Port, StartupOption},
	proto::{
		spaces::{B16Ns192, B24Ns192, IPv4Pns},
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
	let addr_spc = util::determine_address().map(IPv4Pns::try_from)??;
	dbg![StartupOption::new()];
	dbg![Port::new()];
	// [202407160951+0200] NOTE(by: @OST-Gh): current test code.
	for addr in IPv4Pns::from(B16Ns192).into_iter() {
		println!("{}", addr);
	}

	dbg!["hello".as_stream()];

	out.write_all(b"\0")?;
	out.flush()?;
	Ok(())
}
