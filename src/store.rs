/*! Bit management

The `BitStore` trait defines constants and associated functions suitable for
managing the bit patterns of a fundamental, and is the constraint for the
storage type of the data structures of the rest of the crate.

The other types in this module provide stronger rules about how indices map to
concrete bits in fundamental elements. They are implementation details, and are
not exported in the prelude.
!*/

use crate::{
	access::BitAccess,
	cursor::Cursor,
	indices::*,
};

use core::{
	cmp::Eq,
	convert::From,
	fmt::{
		Binary,
		Debug,
		Display,
		LowerHex,
		UpperHex,
	},
	mem::size_of,
	ops::{
		BitAnd,
		BitAndAssign,
		BitOrAssign,
		Not,
		Shl,
		ShlAssign,
		Shr,
		ShrAssign,
	},
	sync::atomic,
};

/** Generalizes over the fundamental types for use in `bitvec` data structures.

This trait must only be implemented on unsigned integer primitives with full
alignment. It cannot be implemented on `u128` on any architecture, or on `u64`
on 32-bit systems.

The `Sealed` supertrait ensures that this can only be implemented locally, and
will never be implemented by downstream crates on new types.
**/
pub trait BitStore:
	//  Forbid external implementation
	Sealed
	+ Binary
	//  Element-wise binary manipulation
	+ BitAnd<Self, Output=Self>
	+ BitAndAssign<Self>
	+ BitOrAssign<Self>
	//  Permit indexing into a generic array
	+ Copy
	+ Debug
	+ Display
	//  Permit testing a value against 1 in `get()`.
	+ Eq
	//  Rust treats numeric literals in code as vaguely typed and does not make
	//  them concrete until long after trait expansion, so this enables building
	//  a concrete Self value from a numeric literal.
	+ From<u8>
	//  Permit extending into a `u64`.
	+ Into<u64>
	+ LowerHex
	+ Not<Output=Self>
	+ Send
	+ Shl<u8, Output=Self>
	+ ShlAssign<u8>
	+ Shr<u8, Output=Self>
	+ ShrAssign<u8>
	//  Allow direct access to a concrete implementor type.
	+ Sized
	+ Sync
	+ UpperHex
{
	/// The width, in bits, of this type.
	const BITS: u8 = size_of::<Self>() as u8 * 8;

	/// The number of bits required to index a bit inside the type. This is
	/// always log<sub>2</sub> of the type’s bit width.
	const INDX: u8 = Self::BITS.trailing_zeros() as u8;

	/// The bitmask to turn an arbitrary number into a bit index. Bit indices
	/// are always stored in the lowest bits of an index value.
	const MASK: u8 = Self::BITS - 1;

	/// Shared-mutable accessor.
	#[doc(hidden)]
	type Access: BitAccess<Self>;

	/// Sets a specific bit in an element to a given value.
	///
	/// # Safety
	///
	/// This method cannot be called from within a `&mut BitSlice` context; it
	/// may only be called by construction of an `&mut Self` reference directly
	/// from a `Self` element.
	///
	/// # Parameters
	///
	/// - `place`: A bit index in the element, from `0` to `Self::MASK`. The bit
	///   under this index will be set according to `value`.
	/// - `value`: A Boolean value, which sets the bit on `true` and clears it
	///   on `false`.
	///
	/// # Type Parameters
	///
	/// - `C: Cursor`: A `Cursor` implementation to translate the index into a
	///   position.
	#[inline(always)]
	fn set<C>(&mut self, place: BitIdx<Self>, value: bool)
	where C: Cursor {
		let mask = *C::mask(place);
		if value {
			*self |= mask;
		}
		else {
			*self &= !mask;
		}
	}

	/// Gets a specific bit in an element.
	///
	/// # Safety
	///
	/// This method cannot be called from within a `&BitSlice` context; it may
	/// only be called by construction of an `&Self` reference directly from a
	/// `Self` element.
	///
	/// # Parameters
	///
	/// - `place`: A bit index in the element, from `0` to `Self::MASK`. The bit
	///   under this index will be retrieved as a `bool`.
	///
	/// # Returns
	///
	/// The value of the bit under `place`, as a `bool`.
	///
	/// # Type Parameters
	///
	/// - `C: Cursor`: A `Cursor` implementation to translate the index into a
	///   position.
	fn get<C>(&self, place: BitIdx<Self>) -> bool
	where C: Cursor {
		*self & *C::mask(place) != Self::from(0)
	}

	/// Counts how many bits in `self` are set to `1`.
	///
	/// This zero-extends `self` to `u64`, and uses the [`u64::count_ones`]
	/// inherent method.
	///
	/// # Parameters
	///
	/// - `&self`
	///
	/// # Returns
	///
	/// The number of bits in `self` set to `1`. This is a `usize` instead of a
	/// `u32` in order to ease arithmetic throughout the crate.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::prelude::BitStore;
	/// assert_eq!(BitStore::count_ones(&0u8), 0);
	/// assert_eq!(BitStore::count_ones(&128u8), 1);
	/// assert_eq!(BitStore::count_ones(&192u8), 2);
	/// assert_eq!(BitStore::count_ones(&224u8), 3);
	/// assert_eq!(BitStore::count_ones(&240u8), 4);
	/// assert_eq!(BitStore::count_ones(&248u8), 5);
	/// assert_eq!(BitStore::count_ones(&252u8), 6);
	/// assert_eq!(BitStore::count_ones(&254u8), 7);
	/// assert_eq!(BitStore::count_ones(&255u8), 8);
	/// ```
	///
	/// [`u64::count_ones`]: https://doc.rust-lang.org/stable/std/primitive.u64.html#method.count_ones
	#[inline(always)]
	fn count_ones(&self) -> usize {
		u64::count_ones((*self).into()) as usize
	}

	/// Counts how many bits in `self` are set to `0`.
	///
	/// This inverts `self`, so all `0` bits are `1` and all `1` bits are `0`,
	/// then zero-extends `self` to `u64` and uses the [`u64::count_ones`]
	/// inherent method.
	///
	/// # Parameters
	///
	/// - `&self`
	///
	/// # Returns
	///
	/// The number of bits in `self` set to `0`. This is a `usize` instead of a
	/// `u32` in order to ease arithmetic throughout the crate.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::prelude::BitStore;
	/// assert_eq!(BitStore::count_zeros(&0u8), 8);
	/// assert_eq!(BitStore::count_zeros(&1u8), 7);
	/// assert_eq!(BitStore::count_zeros(&3u8), 6);
	/// assert_eq!(BitStore::count_zeros(&7u8), 5);
	/// assert_eq!(BitStore::count_zeros(&15u8), 4);
	/// assert_eq!(BitStore::count_zeros(&31u8), 3);
	/// assert_eq!(BitStore::count_zeros(&63u8), 2);
	/// assert_eq!(BitStore::count_zeros(&127u8), 1);
	/// assert_eq!(BitStore::count_zeros(&255u8), 0);
	/// ```
	///
	/// [`u64::count_ones`]: https://doc.rust-lang.org/stable/std/primitive.u64.html#method.count_ones
	#[inline(always)]
	fn count_zeros(&self) -> usize {
		//  invert (0 becomes 1, 1 becomes 0), zero-extend, count ones
		u64::count_ones((!*self).into()) as usize
	}

	/// Extends a single bit to fill the entire element.
	///
	/// # Parameters
	///
	/// - `bit`: The bit to extend.
	///
	/// # Returns
	///
	/// An element with all bits set to the input.
	#[inline]
	fn bits(bit: bool) -> Self {
		if bit {
			!Self::from(0)
		}
		else {
			Self::from(0)
		}
	}
}

/** Marker trait to seal `BitStore` against downstream implementation.

This trait is public in the module, so that other modules in the crate can use
it, but so long as it is not exported by the crate root and this module is
private, this trait effectively forbids downstream implementation of the
`BitStore` trait.
**/
#[doc(hidden)]
pub trait Sealed {}

macro_rules! store {
	( $( $t:ty , $a:ty $( ; )? );* ) => { $(
		impl Sealed for $t {}

		impl BitStore for $t {
			#[cfg(feature = "atomic")]
			type Access = $a;

			#[cfg(not(feature = "atomic"))]
			type Access = Cell<Self>;
		}
	)* };
}

store![
	u8, atomic::AtomicU8;
	u16, atomic::AtomicU16;
	u32, atomic::AtomicU32;
];

#[cfg(target_pointer_width = "64")]
store![u64, atomic::AtomicU64];

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn jump_far_up() {
		//  isize::max_value() is 0x7f...ff, so the result bit will be one less
		//  than the start bit.
		for n in 1 .. 8 {
			let (elt, bit) = n.idx::<u8>().offset(isize::max_value());
			assert_eq!(elt, (isize::max_value() >> u8::INDX) + 1);
			assert_eq!(*bit, n - 1);
		}
		let (elt, bit) = 0.idx::<u8>().offset(isize::max_value());
		assert_eq!(elt, isize::max_value() >> u8::INDX);
		assert_eq!(*bit, 7);
	}

	#[test]
	fn jump_far_down() {
		//  isize::min_value() is 0x80...00, so the result bit will be equal to
		//  the start bit
		for n in 0 .. 8 {
			let (elt, bit) = n.idx::<u8>().offset(isize::min_value());
			assert_eq!(elt, isize::min_value() >> u8::INDX);
			assert_eq!(*bit, n);
		}
	}

	#[test]
	fn bits() {
		assert_eq!(u8::bits(false), 0);
		assert_eq!(u8::bits(true), u8::max_value());

		assert_eq!(u16::bits(false), 0);
		assert_eq!(u16::bits(true), u16::max_value());

		assert_eq!(u32::bits(false), 0);
		assert_eq!(u32::bits(true), u32::max_value());

		#[cfg(target_pointer_width = "64")]
		assert_eq!(u64::bits(false), 0);
		#[cfg(target_pointer_width = "64")]
		assert_eq!(u64::bits(true), u64::max_value());
	}
}
