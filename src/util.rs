///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use crate::netspaces::FromIPv4Error;
use lazy_regex::regex;
use local_ip_address::{local_ip, Error as ResolveError};
use std::{
	env::{args, vars, Args, Vars},
	io,
	net::{IpAddr, Ipv4Addr, Ipv6Addr},
	num::ParseIntError,
	result,
};
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod macro_def {
	#[macro_export]
	macro_rules! count {
		($thing: expr) => { 1 };
		($($thing: expr),* $(,)?) => { 0 $(+ $crate::count!($thing))* };
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub const DEFAULT_PORT: u16 = 49434;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub type Result<T> = result::Result<T, AllErrors>;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum AllErrors {
	#[error("{0}")]
	DetermineAddress(#[from] DetermineAddressError),
	#[error("{0}")]
	FromCall(#[from] FromCallError),

	#[error("{0}")]
	IO(#[from] io::Error),

	#[error("{0}")]
	FromIPv4(#[from] FromIPv4Error),
}

#[derive(Debug, Error)]
pub enum DetermineAddressError {
	#[error("{} cannot currently handle I.P.v.6 addresses such as {0}.", env!("CARGO_PKG_NAME"))]
	NotIPv4(Ipv6Addr),

	#[error("{0}")]
	Local(#[from] ResolveError),
}

#[derive(Debug, Error)]
pub enum FromCallError {
	#[error("No value was found in the supplied iterator.")]
	NotFound,
	#[error("No arguments were supplied to {}", env!("CARGO_PKG_NAME"))]
	NoArguments,
	#[error("A flag was raised, but no required value was put-in.")]
	NotSpecified,

	#[error("{0}")]
	Parse(#[from] ParseIntError),
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum Port {
	Default,
	// [202407161159+0200] NOTE(by: @OST-Gh): no difference, but clearness of origin for the dev.
	PerFlag(u16),
	PerEnvironment(u16),
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum StartupOption {
	Server,
	Client,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn determine_address() -> result::Result<Ipv4Addr, DetermineAddressError> {
	let ip = local_ip()?;
	match ip {
		IpAddr::V4(v4) => Ok(v4),
		IpAddr::V6(v6) => Err(DetermineAddressError::NotIPv4(v6))?,
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Port {
	#[inline(always)]
	fn args_or_default(args: Args) -> result::Result<Self, Self> {
		args.try_into()
			.map_err(|_| Self::Default)
	}

	#[inline(always)]
	fn vars_or_default(vars: Vars) -> result::Result<Self, Self> {
		vars.try_into()
			.map_err(|_| Self::Default)
	}

	#[inline(always)]
	/// Construct a new instance of [`Port`] by querying through both environment and arguments.
	///
	/// If the two do not contain a match, return the default path.
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Port {
	#[inline(always)]
	fn default() -> Self {
		match (Self::args_or_default(args()), Self::vars_or_default(vars())) {
			(Ok(p), _) | (Err(_), Ok(p)) => p,
			_ => Self::Default,
		}
	}
}

#[allow(clippy::from_over_into)]
impl Into<u16> for Port {
	fn into(self) -> u16 {
		match self {
			Self::PerFlag(p) | Self::PerEnvironment(p) => p,
			Self::Default => DEFAULT_PORT,
		}
	}
}

impl TryFrom<Vars> for Port {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut vars: Vars) -> result::Result<Self, Self::Error> {
		let rx = regex!(r#"_{1,2}P(ORT)?[-_]?(N(UM(BER)?)?)?"#i);

		let Some(m) = vars
			.by_ref()
			.find_map(|(key, val)| {
				rx.is_match(key.as_str())
					.then_some(val)
			})
		else {
			Err(Self::Error::NotFound)?
		};
		Ok(Port::PerEnvironment(m.parse::<u16>()?))
	}
}

impl TryFrom<Args> for Port {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut args: Args) -> result::Result<Self, Self::Error> {
		let Some(_bin_path) = args.next() else { unreachable!() };
		let mut it = args.peekable();
		if it.peek()
			.is_none()
		{
			Err(Self::Error::NoArguments)?
		}
		let rx = regex!(
			r#"(-{1,2}|\+)p(ort)?[-_]?(n(um(ber)?)?)?([ :=]?(?<port>[0-9]{1,5})?)?"#i
		);

		let Some(potential) = it
			.by_ref()
			.find_map(|s| {
				let cap = rx.captures(s.as_str())?;
				Some(cap.name("port")
					.map(|m| String::from(m.as_str())))
			})
		else {
			Err(Self::Error::NotFound)?
		};
		let m = match potential {
			Some(m) => m,
			None => {
				if it.peek()
					.is_some_and(|m| regex!(r"[:=]").is_match(m))
				{
					it.next();
				}
				let Some(m) = it.next() else { Err(Self::Error::NotSpecified)? };
				m
			},
		};

		Ok(Port::PerFlag(m.parse::<u16>()?))
	}
}

impl StartupOption {
	#[inline(always)]
	/// Parse a new instance from the passed in [`Args`] or default to [`Client`]
	///
	/// [`Client`]: Self::Client
	pub fn new() -> Self {
		Self::default()
	}

	/// Check whether the instance of
	#[doc = concat!('`', env!("CARGO_PKG_NAME"), '`')]
	/// should start as a server.
	pub fn start_as_server(&self) -> bool {
		let Self::Server = self else { return false };
		true
	}

	/// Check whether the instance of
	#[doc = concat!('`', env!("CARGO_PKG_NAME"), '`')]
	/// should start as a client.
	pub fn start_as_client(&self) -> bool {
		let Self::Client = self else { return false };
		true
	}
}

impl Default for StartupOption {
	#[inline(always)]
	fn default() -> Self {
		Self::try_from(args()).unwrap_or(Self::Client)
	}
}

impl TryFrom<Args> for StartupOption {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut args: Args) -> result::Result<Self, Self::Error> {
		let Some(_bin_path) = args.next() else { unreachable!() };
		let mut it = args.peekable();
		if it.peek()
			.is_none()
		{
			Err(Self::Error::NoArguments)?
		}
		let Some(Some(m)) = it.find_map(|s| {
			let cap = regex!(r"(-{1,2}|\+)(?<init_as>s(erve(r)?)?|c(lient)?)")
				.captures(s.as_str())?;
			Some(cap.name("init_as")
				.map(|m| String::from(m.as_str())))
		}) else {
			Err(Self::Error::NotFound)?
		};
		if m.starts_with('s') {
			Ok(Self::Server)
		} else if m.starts_with('c') {
			Ok(Self::Client)
		} else {
			unreachable!()
		}
	}
}
