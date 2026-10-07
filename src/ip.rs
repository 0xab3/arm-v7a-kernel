pub mod ipv4 {

  //      0                   1                   2                   3
  //  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |Version|  IHL  |Type of Service|          Total Length         |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |         Identification        |Flags|      Fragment Offset    |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |  Time to Live |    Protocol   |         Header Checksum       |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |                       Source Address                          |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |                    Destination Address                        |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
  // |                    Options                    |    Padding    |
  // +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

  use core::net::Ipv4Addr;

  pub struct Header<'a>(&'a [u8]);

  impl<'a> Header<'a> {
    pub fn new(_0: &'a [u8]) -> Self {
      Self(_0)
    }

    pub fn hdr_len(&self) -> usize {
      ((self.0[0] & 0b1111) * 4).into()
    }
    pub fn len(&self) -> u16 {
      u16::from_be_bytes([self.0[2], self.0[3]])
    }

    pub fn proto(&self) -> u8 {
      self.0[9]
    }

    pub fn chksm(&self) -> u16 {
      u16::from_be_bytes([self.0[10], self.0[11]])
    }

    pub fn src_addr(&self) -> Ipv4Addr {
      Ipv4Addr::from_octets([self.0[12], self.0[13], self.0[14], self.0[15]])
    }
    pub fn dst_addr(&self) -> Ipv4Addr {
      Ipv4Addr::from_octets([self.0[16], self.0[17], self.0[18], self.0[19]])
    }
    pub fn payload(&self) -> &'a [u8] {
      &self.0[self.hdr_len()..]
    }
  }
}
