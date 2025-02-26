///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use std::{io, str::Utf8Error};

use thiserror::Error;

use crate::{Message, Nickname};
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum UsernameFromStrError {
	#[error(r"`{0}` doesn't match `^@?[[:alnum:][:punct:]--[\{{\}}\(\)\[\]@]]{{1,31}}`")]
	NoMatch(Box<str>),
	#[error(r"`{0}` is `{1}` bytes long, but it shouldn't exceed `31`.")]
	TooLong(Box<str>, usize),
}

#[derive(Debug, Error)]
pub enum MessageError {
	#[error("The connection has been shutdown.")]
	// [202407180813+0200] NOTE(by: @OST-Gh):
	// 	RET_ON: Other party has gracefully closed the stream.
	ConnectionClosed,
	#[error("The connection has been dropped.")]
	// [202407180814+0200] NOTE(by: @OST-Gh):
	// 	RET_ON: Other party hung up without prior notification.
	ConnectionInterrupted,

	#[error("Waiting for a message timed out.")]
	Timeout,

	#[error("No Identifier specified.")]
	NoIdentifier,

	#[error(transparent)]
	IO(#[from] io::Error),
	#[error(transparent)]
	User(#[from] UserError),
	#[error(transparent)]
	UTF8(#[from] Utf8Error),

	#[error("The provided buffer of length `{0}` is under the minimum `{}`", Message::MIN_LENGTH)]
	TooShort(usize),
	#[error("A provided timestamp is invalid.")]
	InvalidTimestamp,
	#[error("A message's stream `{0:?}` isn't null-terminated.")]
	NoNull(Box<[u8]>),
}

#[derive(Debug, Error)]
pub enum NicknameError {
	#[error(
		"The text `{0}` (of codepoint-length `{1}`) is exceding the maximum amount `{}`.",
		Nickname::MAX_GLYPH_COUNT
	)]
	TooManyGlyphs(Box<str>, usize),

	#[error(transparent)]
	UTF8(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum IdentifierError {
	#[error("The, from the header extracted, length `{0}` mismatches the actual `{1}`")]
	LengthMismatch(usize, usize),
	#[error("An user-identifer cannot be equal to zero")]
	Zero,
}

#[derive(Debug, Error)]
pub enum UserError {
	#[error(transparent)]
	Nickname(#[from] NicknameError),
	#[error(transparent)]
	Identifier(#[from] IdentifierError),

	#[error("The, from the header extracted, length `{0}` mismatches the actual `{1}`")]
	LengthMismatch(usize, usize),
}
