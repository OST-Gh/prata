///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{
	io,
	net::{IpAddr, Ipv4Addr, Ipv6Addr},
	result,
};

use local_ip_address::{local_ip, Error as ResolveError};
use thiserror::Error;

use crate::{
	invoke::FromCallError,
	proto::{spaces::FromIPv4Error, MessageError},
};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod macro_def {
	#[macro_export]
	macro_rules! count {
		($thing: expr) => { 1 };
		($($thing: expr),* $(,)?) => { 0 $(+ $crate::count!($thing))* };
	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub type Result<T> = result::Result<T, AllErrors>;
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
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
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn determine_address() -> result::Result<Ipv4Addr, DetermineAddressError> {
	let ip = local_ip()?;
	match ip {
		IpAddr::V4(v4) => Ok(v4),
		IpAddr::V6(v6) => Err(DetermineAddressError::NotIPv4(v6))?,
	}
}
