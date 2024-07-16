///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[macro_export]
macro_rules! create_flags {
	(
		$(#[$structure_attribute: meta])* [[$structure: ident]]
		$($(#[$field_attribute: meta])* $field: ident = $flag: literal)+

		[const]
		$(#[$lone_attribute: meta])* $lone: ident = [..]
		$(#[$shift_attribute: meta])* $shift: ident = $by: literal
		$(#[$length_attribute: meta])* $length: ident = $number: literal
	) => {
		$(#[$structure_attribute])*
		struct $structure(u32);

		impl $structure {
			$(#[$lone_attribute])* const $lone: [char; count!($($flag),+)] = [$($flag),+];
			$(#[$shift_attribute])* const $shift: u32 = $by;
			$(#[$length_attribute])* const $length: u32 = $number;

			#[inline(always)] fn flag_check(symbol: &char) -> bool { symbol.is_ascii_alphabetic() && symbol.is_ascii_lowercase() }
			$(
				#[doc = concat!("Specify using '`-", $flag, "`'.")]
				$(#[$field_attribute])*
				// macro bullshit
				pub(crate) fn $field(&self) -> bool {
					#[cfg(debug_assertions)] if !Self::flag_check(&$flag) { panic!("get a flag  NOT-ALPHA") }
					**self >> Self::from($flag).into_inner() & 1 == 1 // bit hell:)
					// One copy call needed (**)
					// 0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0   0
					//                         z   y   x   w   v   u   t   s   r   q   p   o   n   m   l   k   j   i   h   g   f   e   d   c   b   a
					//                       122 121 120 119 118 117 116 115 114 113 112 111 110 109 108 107 106 105 104 103 102 101 100 099 098 097
					//                       025 024 023 022 021 020 019 018 017 016 015 014 013 012 011 010 009 008 007 006 005 004 003 002 001 000
				}
			)+

			#[inline(always)]
			/// Get the underlying unsigned integer.
			fn into_inner(self) -> u32 { self.0 }

			/// Split the program arguments into files and flags.
			///
			/// # Panics:
			///
			/// - Arguments are empty.
			fn separate_from(initial: impl IntoIterator<Item = String>) -> (Self, std::iter::Peekable<impl Iterator<Item = String>>) {
				let mut flag_count = 0;
				let mut iterator = initial.into_iter();
				let bits = iterator
					.by_ref()
					.map_while(|argument|
						{
							let raw = argument
								.strip_prefix('-')?
								.replace(|symbol| !Self::flag_check(&symbol), "");
							flag_count += 1;
							Some(raw)
						}
					)
					.fold(
						Self(0),
						|mut bits, raw|
						{
							for symbol in raw
								.chars()
								.filter(|symbol| Self::$lone.contains(symbol))
							{ *bits |= 1 << Self::from(symbol).into_inner() }
							bits
						}
					);
				(
					bits,
					iterator
						.skip(flag_count)
						.peekable()
				)
			}
		}

		impl From<char> for $structure {
			#[inline]
			fn from(symbol: char) -> Self {
				#[cfg(debug_assertions)] if !Self::flag_check(&symbol) { panic!("get a flag  NOT-ALPHA") }
				Self((symbol as u32 - Self::$shift) % Self::$length)
			}
		}

		impl Deref for $structure {
			type Target = u32;

			#[inline(always)] fn deref(&self) -> &u32 { &self.0 }
		}

		impl DerefMut for $structure {
			#[inline(always)] fn deref_mut(&mut self) -> &mut u32 { &mut self.0 }
		}
	};
}
