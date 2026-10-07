#![no_std]
#![no_main]
#![feature(core_io)]
#![feature(io_const_error)]
#![allow(unsafe_op_in_unsafe_fn)]

mod allocator;
mod arp;
mod bitset;
mod eth;
mod interrupts;
mod ip;
mod kprint;
mod net;
mod paging;
mod ser;
mod tcp;
mod udp;

use core::{
  arch::{asm, global_asm},
  mem,
};

use crate::{ip::ipv4, net::PacketBuilder};

global_asm!(include_str!("./_start.arm"));

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  kprintln!("{}", info);
  loop {}
}

unsafe extern "C" {
  static KERNEL_START: u8;
  static KERNEL_END: u8;
}
const fn kernel_start_addr() -> *const u8 {
  &raw const KERNEL_START
}

const fn kernel_end_addr() -> *const u8 {
  &raw const KERNEL_START
}

fn available_mem() -> usize {
  0x40000000 - (kernel_end_addr() as usize - kernel_start_addr() as usize)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn soft_inter() {
  panic!("we got software interrupt");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
  interrupts::enable();
  unsafe { asm!("svc #0x1") };

  let l9eth = eth::L9118Eth::new();
  let mac_addr = eth::L9118Eth::mac_addr(&l9eth);

  loop {
    let mut out_buf: [u8; 1500] = mem::zeroed();
    let frame = eth::L9118Eth::read_frame(&mut out_buf[..]);

    if let Err(e) = frame {
      if !matches!(e, eth::FrameError::EAgain) {
        kprintln!("error! {}", e);
      }
      continue;
    }
    if let Ok(f) = frame {
      match f.typ() {
        // arp packet
        0x806 => {
          let arp = arp::Packet::new(f.payload());
          if arp.htype() != 0x01 || arp.ptype() != 0x800 {
            panic!("{}", arp::Error::Unsupported);
          }

          let arp_res = PacketBuilder::new(arp.src_mac(), mac_addr)
            .arp(02, arp.dst_ip(), arp.src_ip())
            .build();

          eth::L9118Eth::write_data_osns(&arp_res.data[..arp_res.len]);
        }
        // ipv4 packet
        0x800 => {
          kprintln!("ipv4");
          let ipv4hdr = ipv4::Header::new(f.payload());
          let proto = ipv4hdr.proto();
          let ip_payload = ipv4hdr.payload();
          if proto == 17 {
            let udp_pkt = udp::Header::new(ip_payload);
            let udp_payload = udp_pkt.payload();

            kprintln!(
              "src_ip: {}, src_port: {}, dst_ip: {}, dst_port: {}, len: {} ",
              ipv4hdr.src_addr(),
              udp_pkt.src_port(),
              ipv4hdr.dst_addr(),
              udp_pkt.dst_port(),
              udp_pkt.len(),
            );
            let as_str = str::from_utf8(udp_payload).unwrap();
            kprint!("got message: {}", as_str);

            kprintln!("sending reply");
            let reply = PacketBuilder::new(f.src_addr(), mac_addr)
              .udp(
                ipv4hdr.dst_addr(),
                udp_pkt.dst_port(),
                ipv4hdr.src_addr(),
                udp_pkt.dst_port(),
                "we sent a reply osns".as_bytes(),
              )
              .build();
            eth::L9118Eth::write_data_osns(&reply.data[..reply.len]);
          }
        }
        _ => {
          kprintln!("unknown type! {:02X}", f.typ());
        }
      }
    }
  }
}
