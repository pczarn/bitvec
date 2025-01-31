/*! `bitvec` – `[bool]` in overdrive.

This crate provides views into slices of bits that are truly `[u1]`. Each bit in
the data segment is used, unlike `[bool]` which ignores seven bits out of every
byte.

`bitvec`’s data structures provide strong guarantees about, and fine-grained
control of, the bit-level representation of a contiguous region of memory. The
user is empowered to choose the fundamental type underlying the store – `u8`,
`u16`, `u32`, or `u64` – and the order in which each primitive is traversed –
big-endian, from the most significant bit to the least; or little-endian, from
the least significant bit to the most.

This level of control is not necessary for most use cases where users just want
to put bits in a sequence, but it is critically important for users making
packets that leave main memory and hit some external device like a peripheral
controller or a network socket. In order to provide convencienc to users for
whom the storage details do not matter, `bitvec` types default to using
big-endian bit order on `u8`. This means that the bits you would write down on
paper match up with the bits as they are stored in memory.

For example, the bit sequence `[0, 1, 1, 0, 1, 0, 0, 1]` inserted into `bitvec`
structures with no extra type specification will produce the `<BigEndian, u8>`
variant, so the bits in memory are `0b01101001`. With little-endian bit order,
the memory value would be `0b10010110` (reversed order!).

In addition to providing compact, efficient, and powerful storage and
manipulation of bits in memory, the `bitvec` structures are capable of acting as
a queue, set, or stream of bits. They implement the bit-wise operators for
Boolean arithmetic, arithmetic operators for 2’s-complement numeric arithmetic,
read-only indexing, bit shifts, and access to the underlying storage fundamental
elements as a slice.
!*/

#![deny(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(test, feature = "serde"))]
extern crate serde_test;

#[macro_use]
mod macros;

pub mod bits;
pub mod cursor;
mod domain;
pub mod indices;
mod pointer;
pub mod prelude;
pub mod slice;
pub mod store;

#[cfg(feature = "alloc")]
#[cfg_attr(all(not(feature = "alloc"), tarpaulin), skip)]
pub mod boxed;

#[cfg(feature = "alloc")]
#[cfg_attr(all(not(feature = "alloc"), tarpaulin), skip)]
pub mod vec;

#[cfg(feature = "serde")]
mod serdes;

/// Expose crate internals for use in doctests and external tests.
#[cfg(feature = "testing")]
pub mod testing {
	pub use crate::{
		bits::*,
		boxed::*,
		cursor::*,
		domain::*,
		indices::*,
		macros::*,
		pointer::*,
		slice::*,
		store::*,
		vec::*,
	};
}

/** Perform single-bit ripple-carry addition.

This function performs carry-aware binary addition on single bits of each
addend. It is used in multiple places throughout the library, and so is pulled
here for deduplication.

# Parameters

- `a`: One bit of addend.
- `b`: One bit of addend.
- `c`: The carry-bit input.

# Returns

- `.0`: The sum of `a + b + c`.
- `.1`: The carry-out of `a + b + c`.

# Truth Table

`a` and `b` are the addends, `c` is the carry-input, `y` is the sum, and `z` is
the carry-output.

```text
a + b + c => y, z
-----------------
0 + 0 + 0 => 0, 0
0 + 0 + 1 => 1, 0
0 + 1 + 0 => 1, 0
0 + 1 + 1 => 0, 1
1 + 0 + 0 => 1, 0
1 + 0 + 1 => 0, 1
1 + 1 + 0 => 0, 1
1 + 1 + 1 => 1, 1
**/
#[inline]
fn rca1(a: bool, b: bool, c: bool) -> (bool, bool) {
	/// Ripple-carry addition is a reduction operation from three bits of input
	/// (a, b, carry-in) to two outputs (sum, carry-out). This table contains
	/// the map of all possible inputs to their output.
	//  Note: I checked in Godbolt, and the jump table lookup comes out to ~ten
	//  simple instructions with the table baked in as immediate values. The
	//  more semantically clear match statement does not optimize nearly as
	//  well.
	const RCA: [u8; 8] = [
		//      a + b + c => (y, z)
		0,  //  0 + 0 + 0 => (0, 0)
		2,  //  0 + 1 + 0 => (1, 0)
		2,  //  1 + 0 + 0 => (1, 0)
		1,  //  1 + 1 + 0 => (0, 1)
		2,  //  0 + 0 + 1 => (1, 0)
		1,  //  0 + 1 + 1 => (0, 1)
		1,  //  1 + 0 + 1 => (0, 1)
		3,  //  1 + 1 + 1 => (1, 1)
	];
	//  Compute the lookup index from carry-in, left, and right
	let jmp = ((c as u8) << 2) | ((a as u8) << 1) | (b as u8);
	//  Look up the output bits
	let yz = RCA[jmp as usize];
	//  Split them
	(yz & 2 != 0, yz & 1 != 0)
}
