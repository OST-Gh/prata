///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	io::{self, BufRead, BufReader, Write},
	net::{Ipv4Addr, TcpListener, TcpStream, ToSocketAddrs},
	str::from_utf8_unchecked,
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		Arc,
	},
	thread::{park, sleep, spawn, Builder, JoinHandle, Thread},
};

use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use parking_lot::{Mutex, RwLock};

use super::stream::Streamable;
use crate::proto::Username;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
static mut USER_ID: u32 = 0;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub type UserId = u32;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
struct ConnectClientBuilder {
	stream: TcpStream,

	user_id: Option<UserId>,
}

struct Message {
	from_user: UserId,
	content: Box<[u8]>,
}

struct ConnectedClient {
	id: UserId,
	handler: JoinHandle<io::Result<()>>,
}

struct Registry {
	names: Vec<u8>, // \0 terminated list
	ids: Vec<UserId>,
}

pub struct Server {
	incoming: JoinHandle<io::Result<TcpListener>>,

	registry: Registry,

	message_handler: JoinHandle<()>,

	message_log: Arc<RwLock<Vec<Message>>>,

	connected: RwLock<Vec<ConnectedClient>>,

	distributor: Receiver<Message>,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl ConnectClientBuilder {
	/// Connect to a handler via a sender.
	pub const fn and_user(mut self, id: UserId) -> Self {
		self.user_id = Some(id);
		self
	}

	fn bubble(self) -> Option<Self> {
		self.user_id?;
		Some(self)
	}

	pub fn spawn_thread_on(self, server: &Server) -> Option<()> {
		let Self { stream, user_id } = self.bubble()?;

		let mut stream = BufReader::new(stream);
		let id = unsafe { user_id.unwrap_unchecked() };
		let message_log = server
			.message_log
			.clone();

		let handler = spawn(move || {
			let mut buffer = Vec::with_capacity(2);
			let mut written = 0;
			loop {
				let messages = message_log.read();
				let amount = messages.len();
				if amount > written {
					for i in written .. amount {
						stream.get_mut()
							.write_all(unsafe { messages.get_unchecked(i) }.stream())?
					}
					written = amount;
				}
				drop(messages);

				let content = stream.buffer();
				if content.len() != 0 && content.contains(&b'\0') {
					match stream.read_until(b'\0', &mut buffer) {
						Ok(_) => message_log
							.write()
							.push(Message {
								from_user: id,
								content: buffer
									.as_slice()
									.into(),
							}),
						Err(_) => {},
					}
				}
			}
		});
		server.connected
			.write()
			.push(ConnectedClient { handler, id });
		Some(())
	}
}

impl Message {
	#[inline(always)]
	const fn who(&self) -> UserId { self.from_user }

	#[inline(always)]
	fn stream(&self) -> &[u8] { &self.content }
}

impl Registry {
	fn get_user(&self, n: usize) -> Option<Username> {
		let mut iter = self
			.names
			.iter()
			.enumerate()
			.filter_map(|(i, byte)| (byte == &b'\0').then_some(i));

		let lower = iter
			.by_ref()
			.nth(n - 1)?;
		let upper = iter.next()?;
		let name = unsafe {
			from_utf8_unchecked(
				self.names
					.get_unchecked(lower .. upper),
			)
		};
		Some(Username(String::from(name)))
	}

	fn new_user_id(&self) -> u32 {
		let inc: u32;
		unsafe {
			let ptr = &raw mut USER_ID;
			inc = ptr.read_unaligned() + 1;
			ptr.write_unaligned(inc);
			
		}
		inc
	}

	fn new() -> Self {
		Self { names: Vec::with_capacity(16), ids: Vec::with_capacity(16) }
	}
}

impl Default

impl Server {
	/// Create a nonblocking [`TcpListener`] on a given I.P.A. and port.
	pub fn host_on(addr: impl ToSocketAddrs) -> Result<Self, io::Error> {
		let incoming_receiver = TcpListener::bind(addr)?;

		let connected = Mutex::new(Vec::with_capacity(2));


		let incoming_handler = spawn(move || {
			let (stream, addr) = incoming_receiver.accept()?;
			connected
				.lock()
				.push()
		});
	}
}
