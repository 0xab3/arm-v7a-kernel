#![allow(dead_code)]

use core::{
  array::TryFromSliceError,
  fmt::Display,
  io::Write,
  mem,
  ptr::{self},
};

use crate::{
  bitset::{BitSet, IntoInner},
  kprintln,
};

const ETH_BASE_ADDR: u32 = 0x1A000000;

const MAC_CSR_CMD_ADDR: *mut u32 = (ETH_BASE_ADDR + 0xA4) as *mut u32; // MAC_CSR_CMD_OFFSET
const MAC_CSR_DATA_ADDR: *mut u32 = (ETH_BASE_ADDR + 0xA8) as *mut u32; // MAC_CSR_DATA_OFFSET

const ETH_RX_DATA_PORT: *mut u32 = (ETH_BASE_ADDR + 0x0) as *mut u32;
const ETH_TX_DATA_PORT: *mut u32 = (ETH_BASE_ADDR + 0x20) as *mut u32;

const ETH_RX_STATUS_FIFO_PORT: *mut u32 = (ETH_BASE_ADDR + 0x40) as *mut u32;
const ETH_RX_STATUS_FIFO_PEEK: *mut u32 = (ETH_BASE_ADDR + 0x44) as *mut u32;
const ETH_TX_STATUS_FIFO_PORT: *mut u32 = (ETH_BASE_ADDR + 0x48) as *mut u32;
const ETH_TX_STATUS_FIFO_PEEK: *mut u32 = (ETH_BASE_ADDR + 0x4C) as *mut u32;

const ETH_RX_FIFO_INF: *mut u32 = (ETH_BASE_ADDR + 0x7c) as *mut u32;
const ETH_TX_FIFO_INF: *mut u32 = (ETH_BASE_ADDR + 0x80) as *mut u32;
const ETH_RX_CFG: *mut u32 = (ETH_BASE_ADDR + 0x6C) as *mut u32;

#[derive(Debug, Clone)]
pub struct TxBufferWriter {
  write_addr: *mut u32,
}

#[derive(Debug)]
pub enum FrameError {
  BufferTooShort,
  EAgain, // remove this and use interrupts
  SliceError(TryFromSliceError),
}

impl Display for FrameError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      FrameError::BufferTooShort => return write!(f, "buffer too short"),
      FrameError::SliceError(e) => return write!(f, "slice error {}", e),
      FrameError::EAgain => return write!(f, "no data to read or smth"),
    }
  }
}
impl core::error::Error for FrameError {}
impl From<TryFromSliceError> for FrameError {
  fn from(e: TryFromSliceError) -> Self {
    FrameError::SliceError(e)
  }
}

pub struct Frame<'a>(&'a [u8]);

impl<'a> Frame<'a> {
  pub fn new(_0: &'a [u8]) -> Self {
    Self(_0)
  }
  pub fn dst_addr(&self) -> MacAddress {
    MacAddress::new(self.0[0..6].try_into().unwrap())
  }
  pub fn src_addr(&self) -> MacAddress {
    MacAddress::new(self.0[6..12].try_into().unwrap())
  }
  pub fn typ(&self) -> u16 {
    u16::from_be_bytes(self.0[12..14].try_into().unwrap())
  }
  pub fn hdr_len(&self) -> usize {
    14
  }
  pub fn payload(&self) -> &'a [u8] {
    &self.0[14..]
  }
}

impl<'a> core::io::Write for TxBufferWriter {
  fn write(&mut self, buf: &[u8]) -> core::io::Result<usize> {
    // should be 1500 but we dc
    if buf.len() > 1400 {
      return Err(core::io::const_error!(
        core::io::ErrorKind::StorageFull,
        "bro we have not implemented fragmentation"
      ));
    }
    let cmd_a = self.command_a(buf.len() as u32);
    let cmd_b = self.command_b(buf.len() as u32);

    self.send_u32(cmd_a);
    self.send_u32(cmd_b);

    let padding = buf.len() % 4;

    for chunk in buf.chunks_exact(4) {
      let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
      self.send_u32(value);
    }

    let mut final_shit: [u8; 4] = unsafe { mem::zeroed() };
    let off = buf.len() - padding;
    for i in 0..padding {
      final_shit[i] = buf[off + i];
    }
    if padding != 0 {
      let value = u32::from_le_bytes([final_shit[0], final_shit[1], final_shit[2], final_shit[3]]);
      self.send_u32(value);
    }

    return Ok(buf.len());
  }

  fn flush(&mut self) -> core::io::Result<()> {
    todo!()
  }
}
impl TxBufferWriter {
  pub fn new() -> Self {
    Self {
      write_addr: ETH_TX_DATA_PORT,
    }
  }

  fn command_a(&self, buffer_size: u32) -> u32 {
    if buffer_size > 1024 {
      panic!("lowk cooked this should be comptime!");
    }
    BitSet::default()
      .set(13)
      .set(14)
      .mask(buffer_size)
      .into_inner()
  }

  fn command_b(&self, packet_size: u32) -> u32 {
    if packet_size > 1024 {
      panic!("lowk cooked this should be comptime!");
    }
    BitSet::default()
      .set(14) // disable CRC
      .set(13) // disable padding
      .mask(packet_size)
      .into_inner()
  }
  fn send_u32(&self, data: u32) {
    unsafe {
      ptr::write_volatile(self.write_addr, data);
    }
    // must wait after this
  }
}

#[derive(Debug, Clone)]
pub struct TxBufferReader<'a> {
  data: &'a [u8],
  read_addr: *mut u32,
}

impl<'a> TxBufferReader<'a> {
  pub fn new(data: &'a [u8], read_addr: *mut u32) -> Self {
    Self { data, read_addr }
  }
  // todo doesn't actually read :sob:
  pub fn read(buf: &mut [u8]) -> usize {
    // let fifo_inf = unsafe { ptr::read_volatile(ETH_RX_FIFO_INF) };

    let rx_status = unsafe { ptr::read_volatile(ETH_RX_STATUS_FIFO_PORT) };
    let len = (rx_status >> 16) & 0x3fff;

    // let payload_len = unsafe { ptr::read_volatile(ETH_RX_DATA_PORT) }.to_le();
    let payload_len = len;

    // let payload_len = min(payload_len, len);
    let mut cursor = 0;
    for _ in 0..payload_len.div_ceil(4) {
      let data = unsafe { ptr::read_volatile(ETH_RX_DATA_PORT) };
      let as_bytes = &data.to_le_bytes();
      buf[cursor..cursor + 4].copy_from_slice(as_bytes);
      cursor += 4;
    }
    return cursor;
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MacAddress {
  pub addr: [u8; 6],
}

impl MacAddress {
  pub fn new(addr: [u8; 6]) -> Self {
    Self { addr }
  }

  pub fn from_uints(hi: u32, lo: u32) -> Self {
    let mut as_bytes: [u8; 6] = [0; 6];
    as_bytes[..4].copy_from_slice(&lo.to_be_bytes());
    as_bytes[4..].copy_from_slice(&hi.to_be_bytes()[..2]);
    MacAddress::new(as_bytes)
  }
}

impl Display for MacAddress {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    for (it_index, it) in self.addr.iter().enumerate() {
      write!(f, "{:02x}", it)?;
      if it_index != 5 {
        f.write_str(":")?;
      }
    }
    Ok(())
  }
}

struct L9118CommandBuilder;

impl L9118CommandBuilder {
  fn new() -> L9118CSRCommand {
    // polling bit or smth
    return L9118CSRCommand::new();
  }
  fn new_mi2() -> L9118Mi2Command {
    // polling bit or smth
    return L9118Mi2Command::new();
  }
}

#[derive(Clone)]
struct L9118Mi2Command(BitSet<u32>);

impl L9118Mi2Command {
  fn new() -> Self {
    Self(BitSet::default().set(1)).phy_addr()
  }

  fn phy_addr(mut self) -> Self {
    // must be set to 0b00001
    self.0 = self.0.unset(16).unset(15).unset(14).unset(13).set(12);
    self
  }

  fn read(mut self) -> L9118Mi2Command {
    self.0 = self.0.unset(2);
    self
  }

  fn write(mut self) -> Self {
    self.0 = self.0.set(2);
    self
  }

  #[allow(unused_mut)]
  fn mi_index(mut self, index: u8) -> Self {
    if index != 0 {
      panic!("unimplemented! for bitset!")
    }
    self
  }
}

#[derive(Clone)]
struct L9118CSRCommand(BitSet<u32>);

impl L9118CSRCommand {
  fn new() -> Self {
    Self(BitSet::default().set(32))
  }
  fn read(mut self) -> L9118CSRCommand {
    self.0 = self.0.set(31);
    self
  }
  fn write(mut self) -> L9118CSRCommand {
    self.0 = self.0.unset(31);
    self
  }
  fn mac_cr(mut self) -> L9118CSRCommand {
    // index is 1
    self.0 = self.0.set(1);
    self
  }
  fn mac_addr_hi(mut self) -> L9118CSRCommand {
    // index is 2
    self.0 = self.0.set(2);
    self
  }
  fn mac_addr_lo(mut self) -> L9118CSRCommand {
    // index is 3
    self.0 = self.0.mask(3);
    self
  }

  fn mac_mi2_acc(mut self) -> L9118CSRCommand {
    // index is 1
    self.0 = self.0.mask(6);
    self
  }
  fn mac_mi2_data(mut self) -> L9118CSRCommand {
    // index is 1
    self.0 = self.0.mask(7);
    self
  }
}

impl Into<u32> for L9118CSRCommand {
  fn into(self) -> u32 {
    self.0.into_inner()
  }
}

impl Into<u32> for L9118Mi2Command {
  fn into(self) -> u32 {
    self.0.into_inner()
  }
}

pub struct L9118Eth {
  addr_base: *mut u32,
  csr_cmd_addr: *mut u32,
  csr_data_addr: *mut u32,
}

impl L9118Eth {
  pub fn new() -> Self {
    let s = Self {
      addr_base: ETH_BASE_ADDR as *mut u32,
      csr_cmd_addr: MAC_CSR_CMD_ADDR,
      csr_data_addr: MAC_CSR_DATA_ADDR,
    };
    s.set_trx();
    return s;
  }
  pub fn write_data_osns(data: &[u8]) {
    let mut writer = TxBufferWriter::new();
    writer.write(data).unwrap();
  }
  pub fn read_data_osns(buf: &mut [u8]) -> usize {
    // let buf: [u8; 20] = unsafe { mem::zeroed() };
    // let reader = TxBufferReader::new(&buf, ptr::null_mut::<u32>());

    TxBufferReader::read(buf)
  }
  pub fn write_frame<'a>(data: [u8; 1504]) {
    let mut writer = TxBufferWriter::new();
    writer.write(&data[..]).unwrap();
  }

  pub fn read_frame<'a>(buf: &'a mut [u8]) -> Result<Frame<'a>, FrameError> {
    let bytes_read = TxBufferReader::read(buf);
    if bytes_read < 22 {
      return Err(FrameError::EAgain);
    }
    Ok(Frame::new(&buf[..bytes_read]))
  }

  pub fn mac_addr(&self) -> MacAddress {
    let cmd = L9118CommandBuilder::new().read().mac_addr_lo();
    self.send_command(cmd);

    let mac_lo = self.read_data().to_be();

    let cmd = L9118CommandBuilder::new().read().mac_addr_hi();

    self.send_command(cmd);
    let mac_hi = self.read_data().to_be();

    MacAddress::from_uints(mac_hi, mac_lo)
  }

  pub fn set_trx(&self) {
    // get the flags
    self.send_command(L9118CommandBuilder::new().read().mac_cr().into());

    const TX: u32 = 1 << 3;
    const RX: u32 = 1 << 2;
    let have = self.read_data() | TX | RX;
    self.set_command_data(have);

    self.send_command(L9118CommandBuilder::new().write().mac_cr().into());

    // make sure
    self.send_command(L9118CommandBuilder::new().read().mac_cr().into());
    let read_back = self.read_data();
    assert!(
      ((read_back & (1 << 3 | 1 << 2)) >> 2) == 0b11,
      "this probably shouldn't happen"
    );
  }

  pub fn set_loopback(&self) {
    // set the loopback bit and send to mi2_data register
    self.mi2_write(0, BitSet::with(self.mi2_read(0)).set(15).into_inner());
    let read_back = self.mi2_read(0);
    assert!((read_back >> 14) & 1 == 1, "we can't set the loopback bru");
  }

  fn mi2_write(&self, idx: u8, data: u32) {
    self.set_command_data(data);
    self.send_command(L9118CommandBuilder::new().write().mac_mi2_data());

    // set phy destination for the data that we set
    self.set_command_data(L9118CommandBuilder::new_mi2().write().mi_index(idx).into());
    self.send_command(L9118CommandBuilder::new().write().mac_mi2_acc());
    self.mi2_wait_cmd();
  }

  fn mi2_read(&self, idx: u8) -> u32 {
    // read the shit to mi2 register
    self.set_command_data(L9118CommandBuilder::new_mi2().read().mi_index(idx).into());
    self.send_command(L9118CommandBuilder::new().write().mac_mi2_acc().into());
    self.mi2_wait_cmd();

    // copy from mi2 register
    self.send_command(L9118CommandBuilder::new().read().mac_mi2_data());
    self.read_data()
  }

  fn mi2_wait_cmd(&self) {
    self.send_command(L9118CommandBuilder::new().read().mac_mi2_acc());
    loop {
      let data = self.read_data_volatile();
      if data & 1 == 0 {
        break;
      }
    }
  }

  fn send_command(&self, command: L9118CSRCommand) {
    unsafe { *self.csr_cmd_addr = command.into() }
    while !self.is_csr_ready() {}
  }
  fn read_data(&self) -> u32 {
    unsafe { *self.csr_data_addr }
  }
  fn read_data_volatile(&self) -> u32 {
    unsafe { return core::ptr::read_volatile(self.csr_data_addr) }
  }
  fn set_command_data(&self, data: u32) {
    unsafe { *self.csr_data_addr = data }
  }
  fn is_csr_ready(&self) -> bool {
    unsafe {
      if (core::ptr::read_volatile(self.csr_cmd_addr) >> 31) == 0 {
        return true;
      }
    }
    return false;
  }

  fn read_csr_data() -> u32 {
    unsafe { return (*MAC_CSR_DATA_ADDR).to_be() }
  }

  // pub fn init() {
  //     mac_set_rx();
  // }

  // pub fn read_frame() {
  //     unsafe {
  //         loop {
  //             let data = *ETH_RX_DATA_PORT;
  //             kprintln!("data {}", data);
  //             if data != 0 {
  //                 break;
  //             }
  //         }
  //     }
  // }
}
