pub struct Header<'a>(&'a [u8]);
//               0      7 8     15 16    23 24    31
//          +--------+--------+--------+--------+
//          |     Source      |   Destination   |
//          |      Port       |      Port       |
//          +--------+--------+--------+--------+
//          |                 |                 |
//          |     Length      |    Checksum     |
//          +--------+--------+--------+--------+
//          |
//          |          data octets ...
//          +---------------- ...

impl<'a> Header<'a> {
  pub fn new(_0: &'a [u8]) -> Self {
    Self(_0)
  }
  pub fn src_port(&self) -> u16 {
    u16::from_be_bytes([self.0[0], self.0[1]])
  }
  pub fn dst_port(&self) -> u16 {
    u16::from_be_bytes([self.0[2], self.0[3]])
  }
  pub fn len(&self) -> u16 {
    u16::from_be_bytes([self.0[4], self.0[5]])
  }
  pub fn chcksm(&self) -> u16 {
    u16::from_be_bytes([self.0[6], self.0[7]])
  }
  pub fn payload(&self) -> &'a [u8] {
    &self.0[8..self.len().into()]
  }
}
