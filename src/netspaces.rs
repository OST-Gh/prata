///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//! # Network-spaces.
//!
//! ## Subspaces
//!
//! Fancy-Reg.Ex. `B(?<bits>[0-3][0-9])Ns(?<ident>[0-2][0-9]{2})|IPv4(?<space>[A-Z])ns::\k<space>(?<ident>[0-2][0-9]{2})b(?<bits>[0-3][0-9])`
//! - `bits`: How many bits (out of 32) stay constant inside this address range.
//! In this program, none should fall beneath 8.
//! - `ident`: In combination with `bits`, makes the range uniquely identifiable.
//!
//! A subspace is a Unit-struct, which is contained inside of a net.-space enum.
//!
//! It represents a section of the full I.P. v.4 range that has been explicitly designated for a special purpose.
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// IPv4ns = Internet Protocol version 4 network space
impl_ipv4ns! {
	IPv4ns = {
		contains;
		start	; start_of	;
		end	; end_of	;
		mask	; mask_of	;
	}

	{
		IPv4nsRange		;
		FromIPv4Error		;
		IPv4ColumnSelector	;
	}

	/// Net.-space for Private-, a.k.a.: Local Networks.
	///
	/// This contains the all-too commonly seen Network-spaces: 192.168.0.0/24; 172.16.0.0/12; et-cetera.
	IPv4Pns = {
		P010b08 = B08Ns010 <-  10   0-255 0-255 0-255;
		P172b12 = B12Ns172 <- 172  16- 31 0-255 0-255;
		P198b15 = B15Ns198 <- 198  18- 19 0-255 0-255;
		/// The most common subspace seen.
		P192b16 = B16Ns192 <- 192 168-168 0-255 0-255;
		P192b24 = B24Ns192 <- 192   0-  0 0-  0 0-255;

	}
	/// Net.-space for on-machine addresses.
	///
	/// These address ranges won't outbound.
	IPv4Lns = {
		L000b08 = B08Ns000 <-   0 0-255 0-255 0-255;
		L127b08 = B08Ns127 <- 127 0-255 0-255 0-255;

	}
}
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
macro_rules! impl_ipv4ns {
	(
		$(#[$trait_attributes: meta])*
		$trait: ident = {
			$(#[$filter_attributes: meta])*
			$filter: ident;

			$(#[$min_attributes: meta])*
			$min: ident;
			$(#[$min_of_attributes: meta])*
			$min_of: ident;

			$(#[$max_attributes: meta])*
			$max: ident;
			$(#[$max_of_attributes: meta])*
			$max_of: ident;

			$(#[$mask_attributes: meta])*
			$mask: ident;
			$(#[$mask_of_attributes: meta])*
			$mask_of: ident $(;)?
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
				$short: ident = $long: ident <- $prefix: literal $lower2: literal - $upper2: literal $lower3: literal - $upper3: literal $lower4: literal - $upper4: literal
			);+ $(;)?}
		)+
	) => {
		///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		#[derive(core::fmt::Debug)]
		#[derive(core::cmp::Eq, core::cmp::PartialEq, core::cmp::Ord, core::cmp::PartialOrd)]
		#[derive(core::clone::Clone)]
		#[derive(core::hash::Hash)]
		$(#[$iterator_attributes])*
		pub struct $iterator {
			permanent: *mut core::primitive::u8,
			columns: *mut core::primitive::u8,
		}

		$($(
			#[derive(core::fmt::Debug)]
			#[derive(core::cmp::Eq, core::cmp::PartialEq, core::cmp::Ord, core::cmp::PartialOrd)]
			#[derive(core::clone::Clone, core::marker::Copy)]
			#[derive(core::hash::Hash)]
			#[derive(core::default::Default)]
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
		///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		$(
			$(#[$netspace_attributes])*
			pub enum $netspace {
				$($short($long)),+
			}
		)+

		#[derive(thiserror::Error)]
		#[derive(core::fmt::Debug)]
		#[derive(core::cmp::Eq, core::cmp::PartialEq, core::cmp::Ord, core::cmp::PartialOrd)]
		#[derive(core::clone::Clone)]
		#[derive(core::hash::Hash)]
		$(#[$try_from_error_attributes])*
		pub enum $try_from_error {
			#[error(r#"The IPv4. address "{0}" is not contained within any relative net.-space."#)]
			NotInRange(std::net::Ipv4Addr),
		}

		#[derive(core::fmt::Debug)]
		#[derive(core::cmp::Eq, core::cmp::PartialEq, core::cmp::Ord, core::cmp::PartialOrd)]
		#[derive(core::clone::Clone, core::marker::Copy)]
		#[derive(core::hash::Hash)]
		$(#[$column_choice_attributes])*
		pub enum $column_choice {
			First,
			Second,
			Third,
			Fourth,
		}
		///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		pub trait IPv4ns {
			$(#[$min_attributes])*
			fn $min(&self) -> core::net::Ipv4Addr;
			$(#[$min_of_attributes])*
			fn $min_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8;

			$(#[$max_attributes])*
			fn $max(&self) -> core::net::Ipv4Addr;
			$(#[$max_of_attributes])*
			fn $max_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8;

			$(#[$mask_attributes])*
			fn $mask(&self) -> core::net::Ipv4Addr;
			$(#[$mask_of_attributes])*
			fn $mask_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8;


			$(#[$filter_attributes])*
			fn $filter(address: &core::net::Ipv4Addr) -> bool
			where
				Self: core::marker::Sized;
		}
		///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
		impl $iterator {
			#[inline]
			pub fn has_ended(&self) -> bool {
				let (a, b, c) = self.are_equal();
				a && b && c

			}

			fn get_ident(&self) -> u8 {
				unsafe { *self.permanent }
			}

			fn get_ptrs(&self) -> (*mut core::primitive::u8, *mut core::primitive::u8, *mut core::primitive::u8, *mut core::primitive::u8) {
				unsafe {
					(
						self.columns,
						self.columns.add(1),
						self.columns.add(2),
						self.columns.add(3),
					)
				}
			}

			fn are_equal(&self) -> (core::primitive::bool, core::primitive::bool, core::primitive::bool) {
				let getter = |i| unsafe { self.permanent.add(i).read() == self.columns.add(i).read() };
				(getter(1), getter(2), getter(3))
			}

			fn to_address(&self) -> core::net::Ipv4Addr {
				unsafe {
					<core::net::Ipv4Addr as core::convert::From::<[core::primitive::u8; 4]>>::from([
						self.get_ident(),
						self.columns.add(1).read(),
						self.columns.add(2).read(),
						self.columns.add(3).read(),
					])
				}
			}
		}

		impl core::iter::Iterator for $iterator {
			type Item = core::net::Ipv4Addr;

			fn next(&mut self) -> core::option::Option<core::net::Ipv4Addr> {
				let ptrs = self.get_ptrs();

				if self.has_ended() && unsafe { ptrs.0.read() } == 4 {
					core::option::Option::None?
				}

				let before = self.to_address();
				match self.are_equal() {
					(_, false, true) => unsafe {
						ptrs.3.write(0);
						ptrs.2.write(ptrs.2.read() + 1);
					},
					(false, true, true) => unsafe {
						ptrs.3.write(0);
						ptrs.2.write(0);
						ptrs.1.write(ptrs.1.read() + 1);
					},
					(true, true, true) => unsafe { ptrs.0.write(4) },
					(_, _, _) => unsafe { ptrs.3.write(ptrs.3.read() + 1) },
				}
				core::option::Option::Some(before)
			}
		}
		$(
			impl $trait for $netspace {
				#[inline(always)]
				fn $min(&self) -> core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$min()),+ }
				}
				#[inline(always)]
				fn $min_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match self { $(Self::$short(space) => space.$min_of(colidx)),+ }
				}

				#[inline(always)]
				fn $max(&self) -> core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$max()),+ }
				}
				#[inline(always)]
				fn $max_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match self { $(Self::$short(space) => space.$max_of(colidx)),+ }
				}

				#[inline(always)]
				fn $mask(&self) -> core::net::Ipv4Addr {
					match self { $(Self::$short(space) => space.$mask()),+ }
				}
				#[inline(always)]
				fn $mask_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match self { $(Self::$short(space) => space.$mask_of(colidx)),+ }
				}

				#[inline(always)]
				fn $filter(address: &core::net::Ipv4Addr) -> bool
				where
					Self: core::marker::Sized,
				{ $($long::$filter(address))||+ }
			}

			impl core::convert::TryFrom<core::net::Ipv4Addr> for $netspace {
				type Error = $try_from_error;

				fn try_from(address: std::net::Ipv4Addr) -> Result<Self, Self::Error> {
					let space = match address {
						$(_ if $long::$filter(&address) => Self::$short($long),)+
						_ => core::result::Result::Err($try_from_error::NotInRange(address))?,
					};
					core::result::Result::Ok(space)
				}
			}

			impl std::iter::IntoIterator for $netspace {
				type Item = core::net::Ipv4Addr;
				type IntoIter = $iterator;

				fn into_iter(self) -> Self::IntoIter {
					let [_, column_2, column_3, column_4] = self
						.$min()
						.octets();
					let [ident, max_2, max_3, max_4] = self
						.$max()
						.octets();

					let lay = unsafe { core::alloc::Layout::from_size_align_unchecked(4, 1) };
					let allocator = std::alloc::System;
					let permanent = unsafe { <std::alloc::System as core::alloc::GlobalAlloc>::alloc_zeroed(&allocator, lay) };
					let columns =  unsafe { <std::alloc::System as core::alloc::GlobalAlloc>::alloc_zeroed(&allocator, lay) };

					unsafe {
						*permanent = ident;
						*permanent.add(1) = max_2;
						*permanent.add(2) = max_3;
						*permanent.add(3) = max_4;
					}

					unsafe {
						*columns = 0; // [202407141226+0100] NOTE(by: @OST-Gh): overflow-buffer, it will be important for proper bound checks.
						*columns.add(1) = column_2;
						*columns.add(2) = column_3;
						*columns.add(3) = column_4;
					}

					Self::IntoIter {
						permanent,
						columns,
					}
				}
			}
		)+

		impl $column_choice {
			fn cast(n: core::primitive::u8) -> Self {
				match n % 4 {
					0 => Self::First,
					1 => Self::Second,
					2 => Self::Third,
					3 => Self::Fourth,
					_ => core::unreachable!(),
				}
			}
		}

		impl std::convert::From<core::primitive::usize> for $column_choice {
			fn from(value: usize) -> Self {
				Self::cast((value % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::isize> for $column_choice {
			fn from(value: core::primitive::isize) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::u128> for $column_choice {
			fn from(value: core::primitive::u128) -> Self {
				Self::cast((value % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::i128> for $column_choice {
			fn from(value: core::primitive::i128) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::u64> for $column_choice {
			fn from(value: core::primitive::u64) -> Self {
				Self::cast((value % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::i64> for $column_choice {
			fn from(value: core::primitive::i64) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::u32> for $column_choice {
			fn from(value: core::primitive::u32) -> Self {
				Self::cast((value % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::i32> for $column_choice {
			fn from(value: core::primitive::i32) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::u16> for $column_choice {
			fn from(value: core::primitive::u16) -> Self {
				Self::cast((value % 4) as u8)
			}
		}
		impl std::convert::From<core::primitive::i16> for $column_choice {
			fn from(value: core::primitive::i16) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}
		impl std::convert::From<core::primitive::u8> for $column_choice {
			fn from(value: core::primitive::u8) -> Self {
				Self::cast(value % 4)
			}
		}
		impl std::convert::From<core::primitive::i8> for $column_choice {
			fn from(value: core::primitive::i8) -> Self {
				Self::cast((value.abs() % 4) as core::primitive::u8)
			}
		}

		$($(
			impl $trait for $long {
				#[inline(always)]
				fn $min(&self) -> core::net::Ipv4Addr {
					<core::net::Ipv4Addr as core::convert::From::<[u8; 4]>>::from([$prefix, $lower2, $lower3, $lower4])
				}
				#[inline(always)]
				fn $min_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match colidx.into() {
						$column_choice::First	=> $prefix,
						$column_choice::Second	=> $lower2,
						$column_choice::Third	=> $lower3,
						$column_choice::Fourth	=> $lower4,
					}
				}

				#[inline(always)]
				fn $max(&self) -> core::net::Ipv4Addr {
					<core::net::Ipv4Addr as core::convert::From::<[u8; 4]>>::from([$prefix, $upper2, $upper3, $upper4])
				}
				#[inline(always)]
				fn $max_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match colidx.into() {
						$column_choice::First	=> $prefix,
						$column_choice::Second	=> $upper2,
						$column_choice::Third	=> $upper3,
						$column_choice::Fourth	=> $upper4,
					}
				}
				#[inline(always)]
				fn $mask(&self) -> core::net::Ipv4Addr {
					<core::net::Ipv4Addr as core::convert::From::<[u8; 4]>>::from(
						const {
							[0xFF, !($upper2 - $lower2), !($upper3 - $lower3), !($upper4 - $lower4)]
						}
					)
				}
				#[inline(always)]
				fn $mask_of(&self, colidx: impl core::convert::Into<$column_choice>) -> core::primitive::u8 {
					match colidx.into() {
						$column_choice::First	=> 0xFF,
						$column_choice::Second	=> !($upper2 - $lower2),
						$column_choice::Third	=> !($upper3 - $lower3),
						$column_choice::Fourth	=> !($upper4 - $lower4),
					}
				}

				#[inline]
				fn $filter(address: &core::net::Ipv4Addr) -> core::primitive::bool {
					let [$prefix, $lower2..=$upper2, $lower3..=$upper3, $lower4..=$upper4] = address.octets() else { return false };
					true
				}
			}

			impl core::iter::IntoIterator for $long {
				type Item = <$netspace as core::iter::IntoIterator>::Item;
				type IntoIter = <$netspace as core::iter::IntoIterator>::IntoIter;

				#[inline(always)]
				fn into_iter(self) -> Self::IntoIter {
					<$netspace as core::iter::IntoIterator>::into_iter(<$netspace as core::convert::From<Self>>::from(self))
				}
			}

				impl core::convert::From<$long> for $netspace {
					#[inline(always)] fn from(space: $long) -> Self { Self::$short(space) }
				}
		)+)+
		///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
	}
}
use impl_ipv4ns;
