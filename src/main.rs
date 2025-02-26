///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	io::{BufWriter, Write, stdout},
	sync::Arc,
	thread,
	time::Duration,
};

use besked::Message;
use parking_lot::RwLock;

use crate::{
	client::find_from,
	invoke::{StartupOption, port},
	server::host_on,
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod client;
mod invoke;
mod server;
mod spaces;
mod util;
mod visual;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub type MessageLog = Arc<RwLock<Vec<Message>>>;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// [202407161311+0200] TODO(by: @OST-Gh):
// 	move some stuffs from util into its
// 	own module.
fn main() -> util::Result<()> {
	let out = BufWriter::new(stdout().lock());
	let self_addr = util::local_v4ip()?;
	// [202407160951+0200] NOTE(by: @OST-Gh): current test code.

	let port = port()?;
	let start = StartupOption::default();
	if dbg![start].as_server() {
		host_on(self_addr, port)?;
	} else if start.as_client() {
		dbg![find_from(self_addr, port)];
	}

	Ok(())
}
