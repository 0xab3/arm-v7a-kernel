use core::net::{IpAddr, Ipv4Addr};

use crate::{eth::MacAddress, kprintln, ser::BytesWriter};

//     let mut writer = ser::BytesWriter::new(to);
//     writer.write_u16(self.htype); // htype
//     writer.write_u16(self.ptype); // ptype
//     writer.write_u8(6); // hlen
//     writer.write_u8(4); // plen
//     writer.write_u16(2); // opcode
//     writer.write_bytes(&self.src_mac.addr);
//     // src haddr
//     writer.write_u32(self.src_ip.into()); // dst paddr

//     writer.write_bytes(&self.dst_mac.addr);
//     // dst haddr
//     writer.write_u32(self.dst_ip.into()); // dst paddr

//     &writer.buf[..writer.cursor]

// pub fn htype(&self) -> u16 {
//     u16::from_be_bytes(self.0[0..2].try_into().unwrap())
// }
// pub fn ptype(&self) -> u16 {
//     u16::from_be_bytes(self.0[2..4].try_into().unwrap())
// }
// pub fn opcode(&self) -> u16 {
//     u16::from_be_bytes(self.0[6..8].try_into().unwrap())
// }
// pub fn src_mac(&self) -> MacAddress {
//     MacAddress::new(self.0[8..14].try_into().unwrap())
// }
// pub fn src_ip(&self) -> Ipv4Addr {
//     Ipv4Addr::from(<[u8; 4]>::try_from(&self.0[14..18]).unwrap())
// }
// pub fn dst_mac(&self) -> MacAddress {
//     MacAddress::new(self.0[18..24].try_into().unwrap())
// }
// pub fn dst_ip(&self) -> Ipv4Addr {
//     Ipv4Addr::from(<[u8; 4]>::try_from(&self.0[24..28]).unwrap())
// }
pub struct ArpPacket {
  op: u16,
  src_ip: Ipv4Addr,
  dst_ip: Ipv4Addr,
}

pub struct UdpPacket<'a> {
  src_ip: Ipv4Addr,
  src_port: u16,
  dst_ip: Ipv4Addr,
  dst_port: u16,
  payload: &'a [u8],
}

pub enum EthernetPacket<'a> {
  Arp(ArpPacket),
  IpUdpPacket(UdpPacket<'a>),
  IpTcpPacket,
}
pub struct PacketBuilder<'a> {
  pub data: [u8; 1504],
  pub pkt: Option<EthernetPacket<'a>>,
  pub dst_mac: MacAddress,
  pub src_mac: MacAddress,
}

pub struct Packet {
  pub data: [u8; 1504],
  pub len: usize,
}

pub fn ipv4_chksm(data: &[u8]) -> u16 {
  let mut sum_of = 0u16;
  for it in data.chunks(2) {
    let hi = if it.len() > 1 { it[1] } else { 0 };
    let (res, of) = sum_of.overflowing_add(u16::from_be_bytes([it[0], hi]));
    sum_of = res;
    if of {
      sum_of += 1;
    }
  }

  !(sum_of as u16)
}
impl<'a> PacketBuilder<'a> {
  pub fn new(dst_mac: MacAddress, src_mac: MacAddress) -> Self {
    Self {
      data: [0; 1504],
      pkt: None,
      dst_mac,
      src_mac,
    }
  }

  pub fn arp(mut self, op: u16, src_ip: Ipv4Addr, dst_ip: Ipv4Addr) -> Self {
    let arp_pkt = ArpPacket {
      op,
      src_ip: src_ip,
      dst_ip: dst_ip,
    };
    self.pkt = Option::Some(EthernetPacket::Arp(arp_pkt));

    self
  }
  pub fn udp(
    mut self,
    src_ip: Ipv4Addr,
    src_port: u16,
    dst_ip: Ipv4Addr,
    dst_port: u16,
    payload: &'a [u8],
  ) -> Self {
    let udp_pkt = UdpPacket {
      src_ip,
      src_port,
      dst_ip,
      dst_port,
      payload,
    };
    self.pkt = Option::Some(EthernetPacket::IpUdpPacket(udp_pkt));

    self
  }

  pub fn build(mut self) -> Packet {
    let mut writer = BytesWriter::new(&mut self.data[..]);
    writer.write_bytes(&self.dst_mac.addr);
    writer.write_bytes(&self.src_mac.addr);
    let pkt = self
      .pkt
      .ok_or("oh neall naww")
      .expect("you can't build a packet without a type gng");
    match pkt {
      EthernetPacket::Arp(arp_packet) => {
        writer.write_u16(0x806);
        writer.write_u16(0x1);
        writer.write_u16(0x800);
        writer.write_u8(6);
        writer.write_u8(4);
        writer.write_u16(arp_packet.op);
        writer.write_bytes(&self.src_mac.addr);
        writer.write_u32(arp_packet.src_ip.into());
        writer.write_bytes(&self.dst_mac.addr);
        writer.write_u32(arp_packet.dst_ip.into());
        let cursor = writer.cursor;
        drop(writer);
        Packet {
          data: self.data,
          len: cursor,
        }
      }

      EthernetPacket::IpUdpPacket(udp_pkt) => {
        let header_start = writer.cursor;
        writer.write_u16(0x800);
        writer.write_u8(0x45); // v4, ihl 5
        writer.write_u8(0); // tos 0
        writer.write_u16(20 + 8 + udp_pkt.payload.len() as u16); // iphdr_len + udphdr_len
        // + payload_len
        writer.write_u16(0); // identification
        writer.write_u16(0); // flags / fragment offset
        writer.write_u8(64); // ttl
        writer.write_u8(17); // proto
        let chcksm_loc = writer.cursor;
        writer.write_u16(0); // checksum

        writer.write_bytes(&udp_pkt.src_ip.octets());
        writer.write_bytes(&udp_pkt.dst_ip.octets());
        let calculated_chcksm = ipv4_chksm(&writer.buf[header_start + 1..writer.cursor]);
        writer.buf[chcksm_loc] = calculated_chcksm as u8;
        writer.buf[chcksm_loc + 1] = (calculated_chcksm >> 8) as u8;

        // udp
        //

        writer.write_u16(udp_pkt.src_port);
        writer.write_u16(udp_pkt.dst_port);
        writer.write_u16(8 + udp_pkt.payload.len() as u16); // len 
        writer.write_u16(0); //chcksm

        writer.write_bytes(&udp_pkt.payload);

        let cursor = writer.cursor;
        drop(writer);
        Packet {
          data: self.data,
          len: cursor,
        }
      }
      EthernetPacket::IpTcpPacket => {
        writer.write_u16(0x800);
        todo!("what");
      }
    }
  }
}
