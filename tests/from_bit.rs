use bitvec::{index::BitIdx, prelude::*};

#[test]
fn test_from_bit() {
    // let mut bv = BitVec::new();
    let slice: &BitSlice<_, Msb0> = BitSlice::from_bit(&0u32, BitIdx::new(31).unwrap());
    let bit: &BitSlice<u32, Msb0> = BitSlice::from_bool(true);
    assert_eq!(slice[0], false);
    assert_eq!(bit[0], false);
}
