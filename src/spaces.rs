///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//! # Network-spaces.
//!
//! ## Subspaces
//!
//! A subspace is a Unit-struct, which is contained inside of a net.-space enum.
//!
//! It represents a section of the full I.P. v.4 range that has been explicitly
//! designated for a special purpose.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IPv4ns = Internet Protocol version 4 network space
impl_ipv4ns! {
	Netspace = {
		contains;
		{ start	; start_of	}
		{ end	; end_of	}
		{ mask	; mask_of	}
	}

	{
		Range;
		ParRange;
		FromIPv4Error;
		ColumnSelector
	}

	/// Netspaces for private Networks.
	///
	/// This contains the alltoo commonly seen Netspaces: 192.168.0.0/16; 172.16.0.0/12; et-cetera.
	Private = {
		P10s8	= Private010Bits24 <- { 010-010; 255-000; 255-000; 255-000 }
		P172s12	= Private172Bits20 <- { 172-172; 031-016; 255-000; 255-000 }
		P198s15	= Private198Bits17 <- { 198-198; 019-018; 255-000; 255-000 }
		/// The most common subspace seen.
		P192s16	= Private192Bits16 <- { 192-192; 168-168; 255-000; 255-000 }
		P192s8	= Private192Bits24 <- { 192-192; 000-000; 000-000; 255-000 }
	}
	/// Net.-space for on-machine addresses.
	///
	/// These address ranges won't outbound.
	Local = {
		L0s8	= Local000SBits24 <- { 000-000; 255-000; 255-000; 255-000 }
		L127s8	= Local127SBits24 <- { 127-127; 255-000; 255-000; 255-000 }

	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Gennerate Netspaces.
macro_rules! impl_ipv4ns {
	(
		$(#[$trait_attr: meta])*
		$trait: ident = {
			$(#[$filter_attr: meta])*
			$filter: ident;

			{
				$(#[$min_attr: meta])*
				$min: ident;
				$(#[$min_of_attr: meta])*
				$min_of: ident$(;)?
			}

			{
				$(#[$max_attr: meta])*
				$max: ident;
				$(#[$max_of_attr: meta])*
				$max_of: ident$(;)?
			}

			{
				$(#[$mask_attr: meta])*
				$mask: ident;
				$(#[$mask_of_attr: meta])*
				$mask_of: ident $(;)?
			}
		}
		{
			$(#[$iterator_attr: meta])*
			$iterator: ident;
			$(#[$par_iterator_attr: meta])*
			$par_iterator: ident;
			$(#[$try_from_error_attr: meta])*
			$try_from_error: ident;
			$(#[$column_choice_attr: meta])*
			$column_choice: ident $(;)?
		}
		$(
			$(#[$netspace_attr: meta])*
			$netspace: ident = {$(
				$($(#[$subspace_attr: meta])+)?
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
		#[allow(clippy::identity_op)]
		#[allow(clippy::eq_op)]
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
		$(#[$iterator_attr])*
		pub struct $iterator {
			done: ::core::primitive::bool,

			r: ::core::primitive::u32,
			l: ::core::primitive::u32,

			prefix: ::core::primitive::u32,
			mask: ::core::primitive::u32,
		}

		#[derive(::core::fmt::Debug)]
		#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
		#[derive(::core::clone::Clone)]
		#[derive(::core::hash::Hash)]
		$(#[$par_iterator_attr])*
		pub struct $par_iterator($iterator);

		$($(
			#[derive(::core::fmt::Debug)]
			#[derive(::core::cmp::Eq, ::core::cmp::PartialEq, ::core::cmp::Ord, ::core::cmp::PartialOrd)]
			#[derive(::core::clone::Clone, ::core::marker::Copy)]
			#[derive(::core::hash::Hash)]
			#[derive(::core::default::Default)]
			#[doc = concat!(
				"Unit-Struct representation for the address-range `",
				$lower1,
				'.',
				$lower2,
				'.',
				$lower3,
				'.',
				$lower4,
				"`–`",
				$upper1,
				'.',
				$upper2,
				'.',
				$upper3,
				'.',
				$upper4,
				"`."
			)]
			///
			#[doc = concat!("A subspace of [`", stringify!($netspace), r"`].")]
			$(
				#[doc = r"\"]
				$(#[$subspace_attr])+
			)?
			pub struct $long;
		)+)+
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		$(
			$(#[$netspace_attr])*
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
		$(#[$column_choice_attr])*
		pub enum $column_choice {
			First,
			Second,
			Third,
			Fourth,
		}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		pub trait $trait {
			#[allow(dead_code, unused)]
			$(#[$min_attr])*
			fn $min(&self) -> ::core::net::Ipv4Addr;
			#[allow(dead_code, unused)]
			$(#[$min_of_attr])*
			fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;

			#[allow(dead_code, unused)]
			$(#[$max_attr])*
			fn $max(&self) -> ::core::net::Ipv4Addr;
			#[allow(dead_code, unused)]
			$(#[$max_of_attr])*
			fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;

			#[allow(dead_code, unused)]
			$(#[$mask_attr])*
			fn $mask(&self) -> ::core::net::Ipv4Addr;
			#[allow(dead_code, unused)]
			$(#[$mask_of_attr])*
			fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8;


			#[allow(dead_code, unused)]
			$(#[$filter_attr])*
			fn $filter(address: &::core::net::Ipv4Addr) -> bool
			where
				Self: ::core::marker::Sized;
		}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		impl ::core::iter::Iterator for $iterator {
			type Item = ::core::net::Ipv4Addr;

			fn next(&mut self) -> ::core::option::Option<Self::Item> {
				if self.done {
					return ::core::option::Option::None;
				}
				self.done = self.l >= self.r;

				let item = ::core::option::Option::Some(Self::Item::from_bits(self.prefix | self.l));
				self.l = self.l.wrapping_add(1) & self.mask;
				item
			}

			fn nth(&mut self, n: ::core::primitive::usize) -> ::core::option::Option<Self::Item> {
				if self.done {
					return ::core::option::Option::None;
				}
				if n > self.len() {
					self.done = true;
					self.l = self.r;
					return ::core::option::Option::None;
				}
				self.l = self.l.wrapping_add(n as _) & self.mask;
				let item = ::core::option::Option::Some(Self::Item::from_bits(self.prefix | self.l));
				self.l = self.l.wrapping_add(1) & self.mask;
				item
			}

			#[inline(always)]
			fn min(mut self) -> ::core::option::Option<Self::Item> { self.next() }

			#[inline(always)]
			fn max(mut self) -> ::core::option::Option<Self::Item> { self.next_back() }

			#[inline(always)]
			fn is_sorted(self) -> ::core::primitive::bool { true }

			#[inline(always)]
			fn size_hint(&self) -> (::core::primitive::usize, ::core::option::Option<::core::primitive::usize>) {
				(self.len(), Some(self.mask as ::core::primitive::usize))
			}
		}

		impl ::core::iter::DoubleEndedIterator for $iterator {
			fn next_back(&mut self) -> ::core::option::Option<Self::Item> {
				if self.done {
					return core::option::Option::None;
				}
				self.done = self.r <= self.l;

				let item = ::core::option::Option::Some(Self::Item::from_bits(self.prefix | self.r));
				self.r = self.r.wrapping_sub(1) & self.mask;
				item
			}

			fn nth_back(&mut self, n: ::core::primitive::usize) -> ::core::option::Option<Self::Item> {
				if self.done {
					return ::core::option::Option::None;
				}
				if n > self.len() {
					self.done = true;
					self.r = self.l;
					return ::core::option::Option::None;
				}
				self.r = self.r.wrapping_sub(n as _) & self.mask;
				let item = ::core::option::Option::Some(Self::Item::from_bits(self.prefix | self.r));
				self.r = self.r.wrapping_sub(1) & self.mask;
				item
			}
		}

		impl ::core::iter::ExactSizeIterator for $iterator {
			#[inline(always)]
			fn len(&self) -> ::core::primitive::usize {
				if self.l > self.r || self.r < self.l {
					return 0;
				}
				self.l.abs_diff(self.r) as _
			}
		}

		impl ::rayon::iter::plumbing::Producer for $par_iterator {
			type Item = <Self::IntoIter as ::core::iter::Iterator>::Item;
			type IntoIter = $iterator;

			#[inline(always)]
			fn into_iter(self) -> Self::IntoIter { self.0 }

			fn split_at(self, n: usize) -> (Self, Self) {
				let len = self.0.len();
				let mut end = self.clone();
				let mut start = self;

				start.0.nth_back(len - n - 1);
				end.0.nth(n);

				(start, end)
			}
		}

		impl ::rayon::iter::ParallelIterator for $par_iterator {
			type Item = ::core::net::Ipv4Addr;

			fn drive_unindexed<C>(self, consumer: C) -> C::Result
			where
				C: ::rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
			{
				::rayon::iter::plumbing::bridge(self, consumer)
			}
		}

		impl ::rayon::iter::IndexedParallelIterator for $par_iterator {
			fn len(&self) -> ::core::primitive::usize {
				self.0.len()
			}

			fn drive<C>(self, consumer: C) -> C::Result
			where
				C: ::rayon::iter::plumbing::Consumer<Self::Item>,
			{
				::rayon::iter::plumbing::bridge(self, consumer)
			}

			fn with_producer<B>(self, back: B) -> B::Output
			where
				B: ::rayon::iter::plumbing::ProducerCallback<Self::Item>,
			{
				back.callback(self)
			}
		}

		$(
			::paste::paste!{
				impl $netspace {
					pub const fn iter(&self) -> $iterator {
						match self {
							$(Self::$short(space) => space.iter()),+
						}
					}

					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$min _const>](&self) -> ::core::net::Ipv4Addr {
						match self {
							$(Self::$short(space) => space.[<$min _const>]()),+
						}
					}
					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$min_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match self {
							$(Self::$short(space) => space.[<$min_of _const>](colidx)),+
						}
					}

					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$max _const>](&self) -> ::core::net::Ipv4Addr {
						match self {
							$(Self::$short(space) => space.[<$max _const>]()),+
						}
					}
					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$max_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match self {
							$(Self::$short(space) => space.[<$max_of _const>](colidx)),+
						}
					}

					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$mask _const>](&self) -> ::core::net::Ipv4Addr
					{ match self { $(Self::$short(space) => space.[<$mask _const>]()),+ } }
					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$mask_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match self {
							$(Self::$short(space) => space.[<$mask_of _const>](colidx)),+
						}
					}

					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$filter _const>](addr: &::core::net::Ipv4Addr) -> bool {
						$($long::[<$filter _const>](addr))||+
					}
				}

				impl $trait for $netspace {
					#[inline(always)]
					fn $min(&self) -> ::core::net::Ipv4Addr { self.[<$min _const>]() }
					#[inline(always)]
					fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8
					{ self.[<$min_of _const>](colidx.into()) }

					#[inline(always)]
					fn $max(&self) -> ::core::net::Ipv4Addr { self.[<$max _const>]() }
					#[inline(always)]
					fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8
					{ self.[<$max_of _const>](colidx.into()) }

					#[inline(always)]
					fn $mask(&self) -> ::core::net::Ipv4Addr { self.[<$mask _const>]() }
					#[inline(always)]
					fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8
					{ self.[<$mask_of _const>](colidx.into()) }

					#[inline]
					fn $filter(address: &::core::net::Ipv4Addr) -> ::core::primitive::bool
					{ Self::[<$filter _const>](address) }
				}
			}

			impl ::core::convert::TryFrom<::core::net::Ipv4Addr> for $netspace {
				type Error = $try_from_error;

				fn try_from(address: ::core::net::Ipv4Addr) -> Result<Self, Self::Error> {
					$(if $long::try_from(address).is_ok() { return Ok(Self::$short($long)) })+
					::core::result::Result::Err(Self::Error::NotInRange(address))?
				}
			}

			impl ::core::iter::IntoIterator for $netspace {
				type Item = <Self::IntoIter as ::core::iter::Iterator>::Item;
				type IntoIter = $iterator;

				#[inline(always)]
				fn into_iter(self) -> Self::IntoIter { self.iter() }
			}

			impl ::rayon::iter::IntoParallelIterator for $netspace {
				type Item = <Self::Iter as ::rayon::iter::ParallelIterator>::Item;
				type Iter = $par_iterator;

				#[inline(always)]
				fn into_par_iter(self) -> Self::Iter {
					$par_iterator(self.iter())
				}
			}

			impl<'a> ::rayon::iter::IntoParallelRefIterator<'a> for $netspace
			where
				Self: 'a,
			{
				type Item = <Self::Iter as ::rayon::iter::ParallelIterator>::Item;
				type Iter = $par_iterator;

				fn par_iter(&self) -> Self::Iter {
					$par_iterator(self.iter())
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
			::paste::paste!{
				impl $long {
					pub const fn iter(&self) -> <Self as ::core::iter::IntoIterator>::IntoIter {
						let mask = self.[<$mask _const>]().to_bits();
						let neg_mask = !mask;

						$iterator {
							done: false,

							r: self.[<$max _const>]().to_bits() & neg_mask,
							l: self.[<$min _const>]().to_bits() & neg_mask,

							prefix: self.[<$max _const>]().to_bits() & mask,
							mask: neg_mask,
						}
					}

					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$min _const>](&self) -> ::core::net::Ipv4Addr {
						::core::net::Ipv4Addr::new($lower1, $lower2, $lower3, $lower4)
					}
					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$min_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match colidx {
							$column_choice::First	=> $lower1,
							$column_choice::Second	=> $lower2,
							$column_choice::Third	=> $lower3,
							$column_choice::Fourth	=> $lower4,
						}
					}

					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$max _const>](&self) -> ::core::net::Ipv4Addr {
						::core::net::Ipv4Addr::new($upper1, $upper2, $upper3, $upper4)
					}
					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$max_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match colidx {
							$column_choice::First	=> $upper1,
							$column_choice::Second	=> $upper2,
							$column_choice::Third	=> $upper3,
							$column_choice::Fourth	=> $upper4,
						}
					}

					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					#[allow(clippy::identity_op)]
					#[allow(clippy::eq_op)]
					pub const fn [<$mask _const>](&self) -> ::core::net::Ipv4Addr {
						::core::net::Ipv4Addr::new(!($upper1 - $lower1), !($upper2 - $lower2), !($upper3 - $lower3), !($upper4 - $lower4))
					}
					#[inline]
					#[allow(clippy::zero_prefixed_literal)]
					#[allow(clippy::identity_op)]
					#[allow(clippy::eq_op)]
					pub const fn [<$mask_of _const>](&self, colidx: $column_choice) -> ::core::primitive::u8 {
						match colidx {
							$column_choice::First	=> !($upper1 - $lower1),
							$column_choice::Second	=> !($upper2 - $lower2),
							$column_choice::Third	=> !($upper3 - $lower3),
							$column_choice::Fourth	=> !($upper4 - $lower4),
						}
					}

					#[inline(always)]
					#[allow(clippy::zero_prefixed_literal)]
					pub const fn [<$filter _const>](addr: &::core::net::Ipv4Addr) -> ::core::primitive::bool {
						let [$lower1..=$upper1, $lower2..=$upper2, $lower3..=$upper3, $lower4..=$upper4] = addr.octets() else { return false };
						true
					}
				}
				impl $trait for $long {
					#[inline(always)]
					fn $min(&self) -> ::core::net::Ipv4Addr { self.[<$min _const>]() }
					#[inline(always)]
					fn $min_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 { self.[<$min_of _const>](colidx.into()) }

					#[inline(always)]
					fn $max(&self) -> ::core::net::Ipv4Addr { self.[<$max _const>]() }
					#[inline(always)]
					fn $max_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 { self.[<$max_of _const>](colidx.into()) }

					#[inline(always)]
					fn $mask(&self) -> ::core::net::Ipv4Addr { self.[<$mask _const>]() }
					#[inline(always)]
					fn $mask_of(&self, colidx: impl ::core::convert::Into<$column_choice>) -> ::core::primitive::u8 { self.[<$mask_of _const>](colidx.into()) }

					#[inline]
					fn $filter(address: &::core::net::Ipv4Addr) -> ::core::primitive::bool { Self::[<$filter _const>](address) }
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
