#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::Debug;
#[allow(unused_imports)]
use std_shims::prelude::*;
use std_shims::io::{self, Read, Write};

use curve25519_dalek::{scalar::Scalar, edwards::EdwardsPoint};

mod compressed_point;
pub use compressed_point::CompressedPoint;

const VARINT_CONTINUATION_MASK: u8 = 0b1000_0000;

mod sealed {
  /// A trait for a number readable/writable as a VarInt.
  ///
  /// This is sealed to prevent unintended implementations.
  pub trait VarInt: TryFrom<u64> + Copy {
    const BITS: usize;
    fn into_u64(self) -> u64;
  }

  impl VarInt for u8 {
    const BITS: usize = 8;
    fn into_u64(self) -> u64 {
      self.into()
    }
  }
  impl VarInt for u32 {
    const BITS: usize = 32;
    fn into_u64(self) -> u64 {
      self.into()
    }
  }
  impl VarInt for u64 {
    const BITS: usize = 64;
    fn into_u64(self) -> u64 {
      self
    }
  }
  // Don't compile for platforms where `usize` exceeds `u64`, preventing various possible runtime
  // exceptions
  const _NO_128_BIT_PLATFORMS: [(); (u64::BITS - usize::BITS) as usize] =
    [(); (u64::BITS - usize::BITS) as usize];
  impl VarInt for usize {
    const BITS: usize = core::mem::size_of::<usize>() * 8;
    fn into_u64(self) -> u64 {
      self.try_into().expect("compiling on platform with <64-bit usize yet value didn't fit in u64")
    }
  }
}

/// The amount of bytes this number will take when serialized as a VarInt.
///
/// This function will panic if the VarInt exceeds u64::MAX.
pub fn varint_len<V: sealed::VarInt>(varint: V) -> usize {
  let varint_u64 = varint.into_u64();
  ((usize::try_from(u64::BITS - varint_u64.leading_zeros())
    .expect("64 > usize::MAX")
    .saturating_sub(1)) /
    7) +
    1
}

/// Write a byte.
///
/// This is used as a building block within generic functions.
pub fn write_byte<W: Write>(byte: &u8, w: &mut W) -> io::Result<()> {
  w.write_all(&[*byte])
}

/// Write a number, VarInt-encoded.
///
/// This will panic if the VarInt exceeds u64::MAX.
pub fn write_varint<W: Write, U: sealed::VarInt>(varint: &U, w: &mut W) -> io::Result<()> {
  let mut varint: u64 = varint.into_u64();
  while {
    let mut b = u8::try_from(varint & u64::from(!VARINT_CONTINUATION_MASK))
      .expect("& eight_bit_mask left more than 8 bits set");
    varint >>= 7;
    if varint != 0 {
      b |= VARINT_CONTINUATION_MASK;
    }
    write_byte(&b, w)?;
    varint != 0
  } {}
  Ok(())
}

/// Write a scalar.
pub fn write_scalar<W: Write>(scalar: &Scalar, w: &mut W) -> io::Result<()> {
  w.write_all(&scalar.to_bytes())
}

/// Write a point.
pub fn write_point<W: Write>(point: &EdwardsPoint, w: &mut W) -> io::Result<()> {
  CompressedPoint(point.compress().to_bytes()).write(w)
}

/// Write a list of elements, without length-prefixing.
pub fn write_raw_vec<T, W: Write, F: FnMut(&T, &mut W) -> io::Result<()>>(
  mut f: F,
  values: &[T],
  w: &mut W,
) -> io::Result<()> {
  for value in values {
    f(value, w)?;
  }
  Ok(())
}

/// Write a list of elements, with length-prefixing.
pub fn write_vec<T, W: Write, F: FnMut(&T, &mut W) -> io::Result<()>>(
  f: F,
  values: &[T],
  w: &mut W,
) -> io::Result<()> {
  write_varint(&values.len(), w)?;
  write_raw_vec(f, values, w)
}

/// Read a constant amount of bytes.
pub fn read_bytes<R: Read, const N: usize>(r: &mut R) -> io::Result<[u8; N]> {
  let mut res = [0; N];
  r.read_exact(&mut res)?;
  Ok(res)
}

/// Read a single byte.
pub fn read_byte<R: Read>(r: &mut R) -> io::Result<u8> {
  Ok(read_bytes::<_, 1>(r)?[0])
}

/// Read a u16, little-endian encoded.
pub fn read_u16<R: Read>(r: &mut R) -> io::Result<u16> {
  read_bytes(r).map(u16::from_le_bytes)
}

/// Read a u32, little-endian encoded.
pub fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
  read_bytes(r).map(u32::from_le_bytes)
}

/// Read a u64, little-endian encoded.
pub fn read_u64<R: Read>(r: &mut R) -> io::Result<u64> {
  read_bytes(r).map(u64::from_le_bytes)
}

/// Read a canonically-encoded VarInt.
pub fn read_varint<R: Read, U: sealed::VarInt>(r: &mut R) -> io::Result<U> {
  let mut bits = 0;
  let mut res = 0;
  while {
    let b = read_byte(r)?;
    if (bits != 0) && (b == 0) {
      Err(io::Error::other("non-canonical varint"))?;
    }
    if ((bits + 7) >= U::BITS) && (b >= (1 << (U::BITS - bits))) {
      Err(io::Error::other("varint overflow"))?;
    }

    res += u64::from(b & (!VARINT_CONTINUATION_MASK)) << bits;
    bits += 7;
    b & VARINT_CONTINUATION_MASK == VARINT_CONTINUATION_MASK
  } {}
  res.try_into().map_err(|_| io::Error::other("VarInt does not fit into integer type"))
}

/// Read a canonically-encoded scalar.
///
/// Some scalars within the Monero protocol are not enforced to be canonically encoded. For such
/// scalars, they should be represented as `[u8; 32]` and later converted to scalars as relevant.
pub fn read_scalar<R: Read>(r: &mut R) -> io::Result<Scalar> {
  Option::from(Scalar::from_canonical_bytes(read_bytes(r)?))
    .ok_or_else(|| io::Error::other("unreduced scalar"))
}

/// Read a canonically-encoded Ed25519 point.
///
/// This internally calls [`CompressedPoint::decompress`] and has the same definition of canonicity.
/// This function does not check the resulting point is within the prime-order subgroup.
pub fn read_point<R: Read>(r: &mut R) -> io::Result<EdwardsPoint> {
  CompressedPoint::read(r)?.decompress().ok_or_else(|| io::Error::other("invalid point"))
}

/// Read a variable-length list of elements, without length-prefixing.
pub fn read_raw_vec<R: Read, T, F: FnMut(&mut R) -> io::Result<T>>(
  mut f: F,
  len: usize,
  r: &mut R,
) -> io::Result<Vec<T>> {
  let mut res = vec![];
  for _ in 0 .. len {
    res.push(f(r)?);
  }
  Ok(res)
}

/// Read a constant-length list of elements.
pub fn read_array<R: Read, T: Debug, F: FnMut(&mut R) -> io::Result<T>, const N: usize>(
  f: F,
  r: &mut R,
) -> io::Result<[T; N]> {
  read_raw_vec(f, N, r).map(|vec| {
    vec.try_into().expect(
      "read vector of specific length yet couldn't transform to an array of the same length",
    )
  })
}

/// Read a length-prefixed variable-length list of elements.
///
/// An optional bound on the length of the result may be provided. If `None`, the returned `Vec`
/// will be of the length read off the reader, if successfully read. If `Some(_)`, an error will be
/// raised if the length read off the read is greater than the bound.
pub fn read_vec<R: Read, T, F: FnMut(&mut R) -> io::Result<T>>(
  f: F,
  length_bound: Option<usize>,
  r: &mut R,
) -> io::Result<Vec<T>> {
  let declared_length: usize = read_varint(r)?;
  if let Some(length_bound) = length_bound {
    if declared_length > length_bound {
      Err(io::Error::other("vector exceeds bound on length"))?;
    }
  }
  read_raw_vec(f, declared_length, r)
}
