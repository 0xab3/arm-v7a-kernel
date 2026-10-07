use core::ops::{BitAndAssign, BitOrAssign, Not, Shl, Sub};
#[derive(Default, Clone)]
pub struct BitSet<T>(T);

impl<T> BitSet<T>
where
  T: BitAndAssign<T>
    + BitOrAssign<T>
    + Shl<T, Output = T>
    + From<u8>
    + Not<Output = T>
    + Sub<Output = T>,
{
  pub fn with(inner: T) -> BitSet<T> {
    Self(inner)
  }
  pub fn set(mut self, bit: T) -> BitSet<T> {
    self.0 |= T::from(1u8) << (bit - T::from(1u8));
    return self;
  }
  pub fn unset(mut self, bit: T) -> BitSet<T> {
    self.0 &= !(T::from(1) << (bit - T::from(1u8)));
    return self;
  }
  pub fn mask(mut self, mask: T) -> BitSet<T> {
    self.0 |= mask;
    return self;
  }
}

pub trait IntoInner<T> {
  fn into_inner(self) -> T;
}

impl<T> IntoInner<T> for BitSet<T> {
  fn into_inner(self) -> T {
    self.0
  }
}
