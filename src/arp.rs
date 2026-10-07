use core::{array::TryFromSliceError, fmt::Display, net::Ipv4Addr};

use crate::eth::MacAddress;

#[derive(Debug)]
pub enum Error {
  Unsupported,
  SliceError(TryFromSliceError),
}

impl Display for Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Error::Unsupported => return write!(f, "unsupported protocol or hardware!"),
      Error::SliceError(e) => return write!(f, "slice error {}", e),
    }
  }
}
impl core::error::Error for Error {}
impl From<TryFromSliceError> for Error {
  fn from(e: TryFromSliceError) -> Self {
    Error::SliceError(e)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Packet<'a>(&'a [u8]);

impl<'a> core::fmt::Display for Packet<'a> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "ARP {{ htype: {}, ptype: {}, opcode: {}, src_mac: {}, src_ip: {}, dst_mac: {}, dst_ip: {} }}",
      self.htype(),
      self.ptype(),
      self.opcode(),
      self.src_mac(),
      self.src_ip(),
      self.dst_mac(),
      self.dst_ip(),
    )
  }
}

impl<'a> Packet<'a> {
  pub fn new(_0: &'a [u8]) -> Self {
    Self(_0)
  }
  pub fn htype(&self) -> u16 {
    u16::from_be_bytes(self.0[0..2].try_into().unwrap())
  }
  pub fn ptype(&self) -> u16 {
    u16::from_be_bytes(self.0[2..4].try_into().unwrap())
  }
  pub fn opcode(&self) -> u16 {
    u16::from_be_bytes(self.0[6..8].try_into().unwrap())
  }
  pub fn src_mac(&self) -> MacAddress {
    MacAddress::new(self.0[8..14].try_into().unwrap())
  }
  pub fn src_ip(&self) -> Ipv4Addr {
    Ipv4Addr::from(<[u8; 4]>::try_from(&self.0[14..18]).unwrap())
  }
  pub fn dst_mac(&self) -> MacAddress {
    MacAddress::new(self.0[18..24].try_into().unwrap())
  }
  pub fn dst_ip(&self) -> Ipv4Addr {
    Ipv4Addr::from(<[u8; 4]>::try_from(&self.0[24..28]).unwrap())
  }
}
