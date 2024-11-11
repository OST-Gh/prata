///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//! # Network-spaces.
//!
//! ## Subspaces
//!
//! Fancy-Reg.Ex.
//! `B(?<bits>[0-3][0-9])Ns(?<ident>[0-2][0-9]{2})|IPv4(?<space>[A-Z])ns::\
//! k<space>(?<ident>[0-2][0-9]{2})b(?<bits>[0-3][0-9])`
//! - `bits`: How many bits (out of 32) stay constant inside this address range.
//!   (In this program, none should fall beneath 8.)
//! - `ident`: In combination with `bits`, makes the range uniquely
//!   identifiable.
//!
//! A subspace is a Unit-struct, which is contained inside of a net.-space enum.
//!
//! It represents a section of the full I.P. v.4 range that has been explicitly
//! designated for a special purpose.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IPv4ns = Internet Protocol version 4 network space
impl_ipv4ns! {
	IPv4ns = {
		contains;
		{ start	; start_of	}
		{ end	; end_of	}
		{ mask	; mask_of	}
	}

	{ IPv4nsRange; FromIPv4Error; IPv4ColumnSelector }

	/// Net.-space for Private-, a.k.a.: Local Networks.
	///
	/// This contains the all-too commonly seen Network-spaces: 192.168.0.0/24; 172.16.0.0/12; et-cetera.
	IPv4Pns = {
		P010b08	= B08Ns010 <- { 010-010; 255-000; 255-000; 255-000 }
		P172b12	= B12Ns172 <- { 172-172; 031-016; 255-000; 255-000 }
		P198b15	= B15Ns198 <- { 198-198; 019-018; 255-000; 255-000 }
		/// The most common subspace seen.
		P192b16 = B16Ns192 <- { 192-192; 168-168; 255-000; 255-000 }
		P192b24 = B24Ns192 <- { 192-192; 000-000; 000-000; 255-000 }
	}
	/// Net.-space for on-machine addresses.
	///
	/// These address ranges won't outbound.
	IPv4Lns = {
		L000b08 = B08Ns000 <- { 000-000; 255-000; 255-000; 255-000 }
		L127b08 = B08Ns127 <- { 127-127; 255-000; 255-000; 255-000 }

	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
macro_rules! impl_ipv4ns {
	(
		$(#[$trait_attributes: meta])*
		$trait: ident = {
			$(#[$filter_attributes: meta])*
			$filter: ident;

			{
				$(#[$min_attributes: meta])*
				$min: ident;
				$(#[$min_of_attributes: meta])*
				$min_of: ident$(;)?
			}

			{
				$(#[$max_attributes: meta])*
				$max: ident;
				$(#[$max_of_attributes: meta])*
				$max_of: ident$(;)?
			}

			{
				$(#[$mask_attributes: meta])*
				$mask: ident;
				$(#[$mask_of_attributes: meta])*
				$mask_of: ident $(;)?
			}
		}
		{
			$(#[$iterator_attributes: meta])*
			$iterator: ident;
			$(#[$try_from_error_attributes: meta])*
			$try_from_error: ident;
			$(#[$column_choice_attributes: meta])*
			$column_choice: ident $(;)?
		}
		$(
			$(#[$netspace_attributes: meta])*
			$netspace: ident = {$(
				$($(#[$subspace_documentation: meta])+)?
				$short: ident = $long: ident <- {
					$upper1: literal - $lower1: literal;
					$upper2: literal - $lower2: literal;
					$upper3: literal - $lower3: literal;
					$upper4: literal - $lower4: literal$(;)?
				}
			)+}
		)+
	) => {
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	$($(
		#[allow(clippy::zero_prefixed_literal)]
		const _: () = {
			assert!($upper1 - $lower1 == 0i16);
			assert!($upper2 - $lower2 >= 0i16);
			assert!($upper3 - $lower3 >= 0i16);
			assert!($upper4 - $lower4 >= 0i16);
		};
	)+)+
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		#[derive(::core::fmt::Debug)]
		#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
		#[derive(::core::clone::Clone)]
		#[derive(::core::hash::Hash)]
		$(#[$iterator_attributes])*
		pub struct $iterator {
			permanent: [::core::primitive::u8; 4],
			columns: [::core::primitive::u8; 4],
		}

		$($(
			#[derive(::core::fmt::Debug)]
			#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
			#[derive(::core::clone::Clone, ::core::marker::Copy)]
			#[derive(::core::hash::Hash)]
			#[derive(::core::default::Default)]
			#[doc = concat!(
				"Unit-Struct representation for the address-range `",
				stringify!($prefix.$lower2.$lower3.$lower4), "`–`", stringify!($prefix.$upper2.$upper3.$upper4),
				"`."
			)]
			///
			#[doc = concat!("A subspace of [`", stringify!($netspace), r"`].")]
			$(
				#[doc = r"\"]
				$(#[$subspace_documentation])+
			)?
			pub struct $long;
		)+)+
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		$(
			$(#[$netspace_attributes])*
			pub enum $netspace {
				$($short($long)),+
			}
		)+

		#[derive(thiserror::Error)]
		#[derive(::core::fmt::Debug)]
		#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
		#[derive(::core::clone::Clone)]
		#[derive(::core::hash::Hash)]
		pub enum $try_from_error {
			#[error(r#"The IPv4. address "{0}" is not contained within any relative net.-space."#)]
			NotInRange(::std::net::Ipv4Addr),
		}

		#[derive(::core::fmt::Debug)]
		#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
		#[derive(::core::clone::Clone, ::core::marker::Copy)]
		#[derive(::core::hash::Hash)]
		$(#[$column_choice_attributes])*
		pub enum $column_choice {
			First,
			Second,
			Third,
			Fourth,
		}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		pub trait IPv4ns {
			$(#[$min_attributes])*
			fn $min(&self) -> ::core::net::Ipv4Addr;
			$(#[$min_of_attributes])*
			fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;

			$(#[$max_attributes])*
			fn $max(&self) -> ::core::net::Ipv4Addr;
			$(#[$max_of_attributes])*
			fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;

			$(#[$mask_attributes])*
			fn $mask(&self) -> ::core::net::Ipv4Addr;
			$(#[$mask_of_attributes])*
			fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;


			$(#[$filter_attributes])*
			fn $filter(address: &::core::net::Ipv4Addr) -> bool
			where
				Self: ::core::marker::Sized;
		}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		impl $iterator {
			#[inline]
			pub const fn has_ended(&self) -> bool {
				let (a, b, c) = self.are_equal();
				a && b && c
			}

			#[inline(always)]
			const fn get_ident(&self) -> u8 { self.permanent[0] }

			#[inline(always)]
			const fn get_ptrs(&self) -> (
				&::core::primitive::u8,
				&::core::primitive::u8,
				&::core::primitive::u8,
				&::core::primitive::u8,
			) {
				let [ref a, ref b, ref c, ref d] = self.columns;
				(a, b, c, d)
			}

			#[inline(always)]
			const fn are_equal(&self) -> (::core::primitive::bool, ::core::primitive::bool, ::core::primitive::bool) {
				(
					self.permanent[1] == self.columns[1],
					self.permanent[2] == self.columns[2],
					self.permanent[3] == self.columns[3],
				)
			}

			#[inline(always)]
			const fn to_address(&self) -> ::core::net::Ipv4Addr {
				::core::net::Ipv4Addr::new(
					self.get_ident(),
					self.columns[1],
					self.columns[2],
					self.columns[3],
				)
			}
		}

		impl ::core::iter::Iterator for $iterator {
			type Item = ::core::net::Ipv4Addr;

			fn next(&mut self) -> ::core::option::Option<::core::net::Ipv4Addr> {
				let ptrs = self.get_ptrs();

				if self.has_ended() && *ptrs.0 == 4 {
					::core::option::Option::None?
				}

				let before = self.to_address();
				match self.are_equal() {
					(_, false, true) => {
						*unsafe { self.columns.get_unchecked_mut(3) } = 0;
						*unsafe { self.columns.get_unchecked_mut(2) } += 1;
					},
					(false, true, true) => {
						*unsafe { self.columns.get_unchecked_mut(3) } = 0;
						*unsafe { self.columns.get_unchecked_mut(2) } = 0;
						*unsafe { self.columns.get_unchecked_mut(1) } += 1;
					},
					(true, true, true) => *unsafe { self.columns.get_unchecked_mut(0) } = 4,
					(_, _, _) => *unsafe { self.columns.get_unchecked_mut(3) } += 1,
				}
				::core::option::Option::Some(before)
			}
		}

		$(
			impl $trait for $netspace {
				#[inline(always)]
				fn $min(&self) -> ::core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$min()),+ }
				}
				#[inline(always)]
				fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					match self { $(Self::$short(space) => space.$min_of(colidx)),+ }
				}

				#[inline(always)]
				fn $max(&self) -> ::core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$max()),+ }
				}
				#[inline(always)]
				fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					match self { $(Self::$short(space) => space.$max_of(colidx)),+ }
				}

				#[inline(always)]
				fn $mask(&self) -> ::core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$mask()),+ }
				}
				#[inline(always)]
				fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					match self { $(Self::$short(space) => space.$mask_of(colidx)),+ }
				}

				#[inline(always)]
				fn $filter(address: &::core::net::Ipv4Addr) -> bool
				where
					Self: ::core::marker::Sized,
				{ $($long::$filter(address))||+ }
			}

			impl ::core::convert::TryFrom<::core::net::Ipv4Addr> for $netspace {
				type Error = $try_from_error;

				fn try_from(address: ::core::net::Ipv4Addr) -> Result<Self, Self::Error> {
					$(if $long::try_from(address).is_ok() { return Ok(Self::$short($long)) })+
					::core::result::Result::Err(Self::Error::NotInRange(address))?
				}
			}

			impl ::std::iter::IntoIterator for $netspace {
				type Item = ::core::net::Ipv4Addr;
				type IntoIter = $iterator;

				fn into_iter(self) -> Self::IntoIter {
					let [_, column_2, column_3, column_4] = self
						.$min()
						.octets();
					let permanent = self
						.$max()
						.octets();


					Self::IntoIter {
						permanent,
						columns: [0, column_2, column_3, column_4],
					}
				}
			}
		)+

		impl $column_choice {
			const fn cast(n: ::core::primitive::u8) -> Self {
				match n % 4 {
					0 => Self::First,
					1 => Self::Second,
					2 => Self::Third,
					3 => Self::Fourth,
					_ => ::core::unreachable!(),
				}
			}
		}

		impl ::std::convert::From<::core::primitive::usize> for $column_choice {
			#[inline(always)]
			fn from(value: usize) -> Self {
				Self::cast((value % 4) as ::core::primitive::u8)
			}
		}
		impl ::std::convert::From<::core::primitive::isize> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::isize) -> Self {
				Self::from(value.unsigned_abs())
			}
		}
		impl ::std::convert::From<::core::primitive::u128> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::u128) -> Self {
				Self::cast((value % 4) as ::core::primitive::u8)
			}
		}
		impl ::std::convert::From<::core::primitive::i128> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::i128) -> Self {
				Self::from(value.unsigned_abs())
			}
		}
		impl ::std::convert::From<::core::primitive::u64> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::u64) -> Self {
				Self::cast((value % 4) as ::core::primitive::u8)
			}
		}
		impl ::std::convert::From<::core::primitive::i64> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::i64) -> Self {
				Self::from(value.unsigned_abs())
			}
		}
		impl ::std::convert::From<::core::primitive::u32> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::u32) -> Self {
				Self::cast((value % 4) as ::core::primitive::u8)
			}
		}
		impl ::std::convert::From<::core::primitive::i32> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::i32) -> Self {
				Self::from(value.unsigned_abs())
			}
		}
		impl ::std::convert::From<::core::primitive::u16> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::u16) -> Self {
				Self::cast((value % 4) as u8)
			}
		}
		impl ::std::convert::From<::core::primitive::i16> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::i16) -> Self {
				Self::from(value.unsigned_abs())
			}
		}
		impl ::std::convert::From<::core::primitive::u8> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::u8) -> Self {
				Self::cast(value % 4)
			}
		}
		impl ::std::convert::From<::core::primitive::i8> for $column_choice {
			#[inline(always)]
			fn from(value: ::core::primitive::i8) -> Self {
				Self::from(value.unsigned_abs())
			}
		}

		$($(
			impl $trait for $long {
				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $min(&self) -> ::core::net::Ipv4Addr {
					<::core::net::Ipv4Addr as ::core::convert::From::<[u8; 4]>>::from([$lower1, $lower2, $lower3, $lower4])
				}
				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					match colidx.into() {
						$column_choice::First	=> $lower1,
						$column_choice::Second	=> $lower2,
						$column_choice::Third	=> $lower3,
						$column_choice::Fourth	=> $lower4,
					}
				}

				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $max(&self) -> ::core::net::Ipv4Addr {
					<::core::net::Ipv4Addr as ::core::convert::From::<[u8; 4]>>::from([$upper1, $upper2, $upper3, $upper4])
				}
				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					match colidx.into() {
						$column_choice::First	=> $upper1,
						$column_choice::Second	=> $upper2,
						$column_choice::Third	=> $upper3,
						$column_choice::Fourth	=> $upper4,
					}
				}

				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $mask(&self) -> ::core::net::Ipv4Addr {
					<::core::net::Ipv4Addr as ::core::convert::From::<[u8; 4]>>::from(
						const {
							[!($upper1 - $lower1), !($upper2 - $lower2), !($upper3 - $lower3), !($upper4 - $lower4)]
						}
					)
				}
				#[inline(always)]
				#[allow(clippy::zero_prefixed_literal)]
				fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 {
					// [202408072115+0200] NOTE(by: @OST-Gh): for some reason,
					//	if i don't wrap those computations in a const-block,
					//	it won't compile
					match colidx.into() {
						$column_choice::First	=> 0xFF,
						$column_choice::Second	=> const { !$upper2 - $lower2 },
						$column_choice::Third	=> const { !$upper3 - $lower3 },
						$column_choice::Fourth	=> const { !$upper4 - $lower4 },
					}
				}

				#[inline]
				#[allow(clippy::zero_prefixed_literal)]
				fn $filter(address: &::core::net::Ipv4Addr) -> ::core::primitive::bool {
					let [$lower1, $lower2..=$upper2, $lower3..=$upper3, $lower4..=$upper4] = address.octets() else { return false };
					true
				}
			}

			impl ::core::iter::IntoIterator for $long {
				type Item = <$netspace as ::core::iter::IntoIterator>::Item;
				type IntoIter = <$netspace as ::core::iter::IntoIterator>::IntoIter;

				#[inline(always)]
				fn into_iter(self) -> Self::IntoIter {
					<$netspace as ::core::iter::IntoIterator>::into_iter(<$netspace as ::core::convert::From<Self>>::from(self))
				}
			}

			impl ::core::convert::TryFrom<::core::net::Ipv4Addr> for $long {
				type Error = $try_from_error;

				fn try_from(ip: ::core::net::Ipv4Addr) -> ::core::result::Result<Self, Self::Error> {
					if Self::$filter(&ip) {
						return ::core::result::Result::Ok(Self);
					}
					::core::result::Result::Err(Self::Error::NotInRange(ip))
				}

			}

			impl ::core::convert::From<$long> for $netspace {
				#[inline(always)]
				fn from(space: $long) -> Self {
					Self::$short(space)
				}
			}
		)+)+
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	}
}
use impl_ipv4ns;
