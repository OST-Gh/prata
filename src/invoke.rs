///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	env::{args, vars, Args, Vars},
	hint::unreachable_unchecked,
	num::ParseIntError,
	str::FromStr,
};

use lazy_regex::regex;
use thiserror::Error;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
const DEFAULT_PORT: u16 = 49434;
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
pub enum StartupOption {
	Server,
	Client,
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn port() -> Result<u16, FromCallError> {
	let mut args = args();
	let Some(_bin_path) = args.next() else {
		unsafe { unreachable_unchecked() }
	};
	let mut it = args.peekable();
	if it.peek()
		.is_none()
	{
		Err(FromCallError::NoArguments)?
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
		Err(FromCallError::NotFound)?
	};
	let m = match potential {
		Some(m) => m,
		None => {
			if it.peek()
				.is_some_and(|m| regex!(r"[:=]").is_match(m))
			{
				it.next();
			}
			let Some(m) = it.next() else {
				Err(FromCallError::NotSpecified)?
			};
			m
		},
	};
	match m.parse() {
		Ok(val) => return Ok(val),
		Err(_) => {
			let rx = regex!(r#"_{1,2}P(ORT)?[-_]?(N(UM(BER)?)?)?"#i);

			let Some(m) = vars().find_map(|(key, val)| {
				rx.is_match(key.as_str())
					.then_some(val)
			}) else {
				Err(FromCallError::NotFound)?
			};
			m.parse()
				.map_err(FromCallError::Parse)
		},
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
