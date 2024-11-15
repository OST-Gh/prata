///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	env::{args, vars, Args, Vars},
	num::ParseIntError,
	str::FromStr,
};

use lazy_regex::regex;
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
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
	// [202407161159+0200] NOTE(by: @OST-Gh): no difference but clearness of origin for the dev.
	Specific(u16),
}

#[derive(Debug)]
#[derive(Copy, Clone)]
#[derive(Eq, PartialEq, PartialOrd, Ord)]
#[derive(Hash)]
pub enum StartupOption {
	Server,
	Client,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
impl Port {
	pub const DEFAULT_PORT: u16 = 49434;

	#[inline(always)]
	fn args_or_default(args: Args) -> Result<Self, Self> {
		args.try_into()
			.map_err(|_| Self::Default)
	}

	#[inline(always)]
	fn vars_or_default(vars: Vars) -> Result<Self, Self> {
		vars.try_into()
			.map_err(|_| Self::Default)
	}

	#[inline(always)]
	/// Construct a new instance of [`Port`] by querying through both
	/// environment and arguments.
	///
	/// If the two do not contain a match, return the default path.
	pub fn new() -> Self { Self::default() }
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
			Self::Specific(p) => p,
			Self::Default => Self::DEFAULT_PORT,
		}
	}
}

impl FromStr for Port {
	type Err = ParseIntError;

	#[inline(always)]
	fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self::Specific(s.parse::<u16>()?)) }
}

impl TryFrom<Vars> for Port {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut vars: Vars) -> Result<Self, Self::Error> {
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
		m.parse()
			.map_err(Self::Error::Parse)
	}
}

impl TryFrom<Args> for Port {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut args: Args) -> Result<Self, Self::Error> {
		let Some(_bin_path) = args.next() else { unreachable!() };
		let mut it = args.peekable();
		if it.peek()
			.is_none()
		{
			Err(Self::Error::NoArguments)?
		}
		let rx = regex!(r#"(-{1,2}|\+)p(ort)?[-_]?(n(um(ber)?)?)?(( )?[=: ]?( )?(?<port>[0-9]{1,5})?)?"#i);

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
		m.parse()
			.map_err(Self::Error::Parse)
	}
}

impl StartupOption {
	#[inline(always)]
	/// Parse a new instance from the passed in [`Args`] or default to
	/// [`Client`]
	///
	/// [`Client`]: Self::Client
	pub fn new() -> Self { Self::default() }

	/// Check whether the instance of
	#[doc = concat!('`', env!("CARGO_PKG_NAME"), '`')]
	/// should start as a server.
	pub fn as_server(&self) -> bool {
		let Self::Server = self else { return false };
		true
	}

	/// Check whether the instance of
	#[doc = concat!('`', env!("CARGO_PKG_NAME"), '`')]
	/// should start as a client.
	pub fn as_client(&self) -> bool {
		let Self::Client = self else { return false };
		true
	}
}

impl Default for StartupOption {
	#[inline(always)]
	fn default() -> Self { Self::try_from(args()).unwrap_or(Self::Client) }
}

impl FromStr for StartupOption {
	type Err = FromCallError;

	#[inline(always)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.starts_with('s') {
			Ok(Self::Server)
		} else if s.starts_with('c') {
			Ok(Self::Client)
		} else {
			Err(Self::Err::NotFound)
		}
	}
}

impl TryFrom<Args> for StartupOption {
	type Error = FromCallError;

	#[inline]
	fn try_from(mut args: Args) -> Result<Self, Self::Error> {
		let Some(_bin_path) = args.next() else { unreachable!() };
		let mut it = args.peekable();
		if it.peek()
			.is_none()
		{
			Err(Self::Error::NoArguments)?
		}
		let Some(Some(m)) = it.find_map(|s| {
			let cap = regex!(r"(-{1,2}|\+)(?<init_as>s(erve(r)?)?|c(lient)?)").captures(s.as_str())?;
			Some(cap.name("init_as")
				.map(|m| String::from(m.as_str())))
		}) else {
			Err(Self::Error::NotFound)?
		};
		m.parse()
	}
}
