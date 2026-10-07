// should be a part of net bru

//     0                   1                   2                   3
//     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |          Source Port          |       Destination Port        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                        Sequence Number                        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                    Acknowledgment Number                      |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |  Data |       |C|E|U|A|P|R|S|F|                               |
//    | Offset| Rsrvd |W|C|R|C|S|S|Y|I|            Window             |
//    |       |       |R|E|G|K|H|T|N|N|                               |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |           Checksum            |         Urgent Pointer        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                           [Options]                           |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                                                               :
//    :                             Data                              :
//    :                                                               |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

use core::convert::Into;

pub struct Packet<'a>(&'a [u8]);

impl<'a> Packet<'a> {
  pub fn new(items: &'a [u8]) -> Self {
    Self(items)
  }

  pub fn src_port(&self) -> u16 {
    u16::from_be_bytes([self.0[0], self.0[1]])
  }

  pub fn data_offset(&self) -> usize {
    ((self.0[12] >> 4) as usize * 4).into()
  }
  pub fn dst_port(&self) -> u16 {
    u16::from_be_bytes([self.0[2], self.0[3]])
  }

  pub fn seq_no(&self) -> u32 {
    u32::from_be_bytes([self.0[4], self.0[5], self.0[6], self.0[7]])
  }
  pub fn ack_no(&self) -> u32 {
    u32::from_be_bytes([self.0[7], self.0[8], self.0[9], self.0[10]])
  }
  pub fn payload(&self) -> &'a [u8] {
    return &self.0[self.data_offset()..];
  }
}
