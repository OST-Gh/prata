///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	collections::HashMap,
	io::{self, BufReader},
	net::{Ipv4Addr, TcpListener},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	thread::{JoinHandle, spawn},
};

use besked::{Identifier, Message, errors::MessageError};
use parking_lot::{RwLock, RwLockReadGuard};

use crate::proto::MessageLog;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Server {
	incoming_handler: JoinHandle<Result<(), MessageError>>,

	message_log: MessageLog,

	connected: Arc<RwLock<HashMap<Identifier, JoinHandle<Result<(), MessageError>>>>>,

	quit: Arc<AtomicBool>,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn host_on(addr: Ipv4Addr, port: impl Into<u16>) -> io::Result<Server> {
	let incoming_receiver = TcpListener::bind((addr, port.into()))?;

	let connected = Arc::new(RwLock::new(HashMap::with_capacity(2)));
	let message_log = MessageLog::new(RwLock::from(Vec::new()));
	let quit = Arc::new(AtomicBool::new(false));

	let message_log_2 = message_log.clone();
	let connected_2 = connected.clone();
	let quit_2 = quit.clone();

	let incoming_handler = spawn(move || -> Result<(), MessageError> {
		loop {
			if quit_2.load(Ordering::Acquire) {
				break
			}

			let (mut stream, _addr) = incoming_receiver.accept()?;
			let id = Identifier::default();

			Message::with_identifier(id).send(&mut stream);

			let mut stream = BufReader::new(stream);

			let message_log_3 = message_log_2.clone();

			let handler = spawn(move || {
				let mut message_buffer = Vec::with_capacity(Message::MIN_LENGTH);
				let mut written = Vec::from([0]);
				loop {
					let maybe_lock = message_log_3.try_read();

					let len = |log: &RwLockReadGuard<'_, Vec<Message>>| -> bool {
						let iter = written.iter();
						iter.sum::<usize>() > log.len()
					};

					if maybe_lock
						.as_ref()
						.map_or(false, len)
					{
						let log = unsafe { maybe_lock.unwrap_unchecked() };
						let mut log = log.iter();
						for i in written.iter() {
							log.nth(*i);
						}
						for message in log {
							let Ok(_) = message.send(stream.get_mut()) else {
								written.push(0);
								break
							};
							let maybe_n = written.last_mut();
							*unsafe { maybe_n.unwrap_unchecked() } += 1;
						}
					}

					match Message::recv_buf(&mut stream, &mut message_buffer) {
						Ok(message) => message_log_3
							.write()
							.push(message),
						Err(MessageError::TooShort(_)) => continue,
						Err(other) => Err(other)?,
					};
				}
			});
			connected_2
				.write()
				.insert(id, handler);
		}
		Ok(())
	});

	Ok(Server {
		incoming_handler,
		message_log,
		connected,
		quit,
	})
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Server {
	/// Create a nonblocking [`TcpListener`] on a given I.P.A. and port.
	// pub fn host_on(addr: impl ToSocketAddrs) -> Result<Self, io::Error> {}

	pub fn quit(&self) {
		self.quit
			.store(true, Ordering::Release);
	}

	pub fn shutdown(self) -> Result<(), MessageError> {
		self.quit();
		unsafe {
			self.incoming_handler
				.join()
				.unwrap_unchecked()
		}
	}
}
