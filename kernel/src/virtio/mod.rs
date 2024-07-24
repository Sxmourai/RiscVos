pub mod network;
pub mod block;
pub mod console;
pub mod entropy;
pub mod gpu;
pub mod input;
pub mod memory;

use alloc::{boxed::Box, string::ToString, vec::{self, Vec}};
use log::{info, warn};

use crate::{*, paging::{PAGE_SIZE, PAGE_SIZE64}};

/// QEMU SPECIFIC
pub const VIRTIO_START: usize = 0x1000_1000;
pub const VIRTIO_END: usize =   0x1000_8000;
pub const VIRTIO_STRIDE: usize = 0x1000;
pub const VIRTIO_MAGIC: u32 = 0x74_72_69_76; // "virt" in little endian
pub const VIRTIO_COUNT: usize = (VIRTIO_END-VIRTIO_START)/VIRTIO_STRIDE+1;
pub static mut VIRTIO_DEVICES: [Option<VirtIODevicePtr>; VIRTIO_COUNT] = [const{None}; VIRTIO_COUNT];


pub const VIRTIO_DESC_F_NEXT: u16 = 1;
pub const VIRTIO_DESC_F_WRITE: u16 = 2;
pub const VIRTIO_DESC_F_INDIRECT: u16 = 4;

pub fn init() {
    crate::info!("Probing Virtio devices...");
    for addr in (VIRTIO_START..=VIRTIO_END).step_by(VIRTIO_STRIDE) {
        map!(addr);
        crate::trace!("Virtio probing 0x{:08x}...", addr);
        let magicvalue;
        let deviceid;
        let ptr = addr as *mut u32;
        unsafe {
          magicvalue = ptr.read_volatile();
          deviceid = ptr.add(2).read_volatile();
        }
        if magicvalue != VIRTIO_MAGIC {
          // Not Virt IO device
        }
        // If we are a virtio device, we now need to see if anything
        // is actually attached to it. The DeviceID register will
        // contain what type of device this is. If this value is 0,
        // then it is not connected.
        else if deviceid == 0 {
          // Not connected
        }
        else {
            let idx = (addr - VIRTIO_START) / VIRTIO_STRIDE;
            let mut mmio = StandardVirtIO { idx, read_only: false };
            if init_device(&mut mmio).is_none() {warn!("Failed initialising device {}", deviceid);}
            // https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html#x1-1930005
    
            if let Some(device) = match deviceid {
                2 => {block_device(mmio)},
                4 => {todo!()},
                _ => {todo!("support this device: {}", deviceid)},
            } {
                #[allow(unused_results)]
                unsafe { VIRTIO_DEVICES[idx].replace(device); };
                // for i in 0..1000000 {}
                // let mut buf = Vec::with_capacity(512);
                // unsafe { buf.set_len(buf.capacity()) }
                // // For debugging only donowori
                // let mut dev = unsafe {&mut *((VIRTIO_DEVICES[idx].as_mut()).unwrap().as_mut() as *mut dyn VirtIODevice as *mut BlockDevice)};
                // dev.read(0, &mut buf).unwrap();
                // for i in 0..1000000 {}
                // dbg!(buf);
                // let mut buf = alloc::vec![1u8; 512];
                // for i in 0..1000000 {}
                // dev.write(0, &mut buf).unwrap();
                // for i in 0..1000000 {}
                // let mut buf2 = Vec::with_capacity(512);
                // unsafe { buf2.set_len(buf2.capacity()) }
                // dev.read(0, &mut buf2).unwrap();
                // for i in 0..1000000 {}
                // dbg!(buf2);
            }
        }
    }
}

pub fn block_device(mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    let blk = block::BlockDevice::new(mmio)?;
    Some(VirtIODevicePtr::Block(Box::new(blk)))
}

#[derive(Debug)]
pub struct StandardVirtIO {
    pub idx: usize,
    pub read_only: bool, // ! Costs an entire usize because of alignment, we will have to change that ! 
}
// According to the documentation, this must be a power
// of 2 for the new style. So, I'm changing this to use
// 1 << instead because that will enforce this standard.
pub const VIRTIO_RING_SIZE: usize = 1 << 7;

// VirtIO structures

// The descriptor holds the data that we need to send to 
// the device. The address is a physical address and NOT
// a virtual address. The len is in bytes and the flags are
// specified above. Any descriptor can be chained, hence the
// next field, but only if the F_NEXT flag is specified.
#[repr(C)]
#[derive(Debug)]
pub struct Descriptor {
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}

#[repr(C)]
pub struct Available {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [u16; VIRTIO_RING_SIZE],
	pub event: u16,
}
impl core::fmt::Debug for Available {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Available")
        .field("flags", &self.flags)
        .field("idx", &self.idx)
        .field("ring", &self.ring.iter().map(|x| x.to_string()).collect::<alloc::string::String>())
        .field("event", &self.event).finish()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct UsedElem {
	pub id:  u32,
	pub len: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Used {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [UsedElem; VIRTIO_RING_SIZE],
	pub event: u16,
}

#[repr(C)]
pub struct Queue {
	pub desc:  [Descriptor; VIRTIO_RING_SIZE],
	pub avail: Available,
	// Calculating padding, we need the used ring to start on a page boundary. We take the page size, subtract the
	// amount the descriptor ring takes then subtract the available structure and ring.
	pub padding0: [u8; PAGE_SIZE - (size_of::<Descriptor>() * VIRTIO_RING_SIZE + size_of::<Available>())],
	pub used:     Used,
}
impl core::fmt::Debug for Queue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Queue")
        .field("desc", &format_args!("{:?}", self.desc.iter().filter(|x| x.addr != 0).collect::<Vec<&Descriptor>>()))
        .field("avail", &self.avail)
        // .field("padding0", &self.padding0)
        .field("used", &self.used)
        .finish()
    }
}



/// TODO Return result
fn init_device(dev: &mut StandardVirtIO) -> Option<()> {
    unsafe {
        let mut status_bits = 0;
        dev.write(MmioOffset::Status, status_bits);
        status_bits |= StatusField::Acknowledge;
        dev.write(MmioOffset::Status, status_bits);
        status_bits |= StatusField::DriverOk;
        dev.write(MmioOffset::Status, status_bits);
        // Read device feature bits, write subset of feature
        // we support and device support.
        let host_features = dev.read(MmioOffset::HostFeatures);
        let guest_features = host_features & !(1 << block::VIRTIO_BLK_F_RO);
        let ro = host_features & (1 << block::VIRTIO_BLK_F_RO) != 0;
        dev.write(MmioOffset::GuestFeatures, guest_features);
        status_bits |= StatusField::FeaturesOk;
        dev.write(MmioOffset::Status, status_bits);
        
        // Re-read status to ensure FEATURES_OK is still set.
        // Otherwise, it doesn't support our features.
        if dev.read(MmioOffset::Status) & (StatusField::FeaturesOk as u32) == 0 {
            log::warn!("Device {} doesn't support our features :c ", dev.idx);
            dev.write(MmioOffset::Status, StatusField::Failed as _);
            return None;
        }
        Some(())
    }
}
impl StandardVirtIO {
    fn base(&self) -> usize {
        VIRTIO_START+self.idx*VIRTIO_STRIDE
    }
    unsafe fn write(&mut self, reg: MmioOffset, value: u32) {
        unsafe{*(reg.ptr()).byte_add(self.base()) = value}
    }
    fn read(&self, reg: MmioOffset) -> u32 {
        unsafe{*(reg.ptr()).byte_add(self.base())}
    }
}

pub trait VirtIODevice {
    fn handle_int(&mut self);
}

pub enum MmioOffset {
    MagicValue = 0x000,
    Version = 0x004,
    DeviceId = 0x008,
    VendorId = 0x00c,
    HostFeatures = 0x010,
    HostFeaturesSel = 0x014,
    GuestFeatures = 0x020,
    GuestFeaturesSel = 0x024,
    GuestPageSize = 0x028,
    QueueSel = 0x030,
    QueueNumMax = 0x034,
    QueueNum = 0x038,
    QueueAlign = 0x03c,
    QueuePfn = 0x040,
    QueueNotify = 0x050,
    InterruptStatus = 0x060,
    InterruptAck = 0x064,
    Status = 0x070,
    Config = 0x100,
}
impl MmioOffset {
    pub fn ptr(self) -> *mut u32 {
        (self as u64) as *mut u32
    }
}


#[repr(usize)]
pub enum VirtIODevicePtr {
    Network(Box<network::NetworkDevice>),
    Block(Box<block::BlockDevice>),
    Console(Box<console::ConsoleDevice>),
    Entropy(Box<entropy::EntropyDevice>),
    Gpu(Box<gpu::GpuDevice>),
    Input(Box<input::InputDevice>),
    Memory(Box<memory::MemoryDevice>),
}
impl VirtIODevicePtr {
    pub fn category(self) -> u8 {
        match self {
            Self::Network(_) => 1,
            Self::Block(_)   => 2,
            Self::Console(_) => 3,
            Self::Entropy(_) => 4,
            Self::Gpu(_)     => 16,
            Self::Input(_)   => 18,
            Self::Memory(_)  => 24,
        }
    }
    pub fn handle_int(&mut self) {
        match self {
            VirtIODevicePtr::Network(dev) => dev.handle_int(),
            VirtIODevicePtr::Block(dev)     => dev.handle_int(),
            VirtIODevicePtr::Console(dev) => dev.handle_int(),
            VirtIODevicePtr::Entropy(dev) => dev.handle_int(),
            VirtIODevicePtr::Gpu(dev)         => dev.handle_int(),
            VirtIODevicePtr::Input(dev)     => dev.handle_int(),
            VirtIODevicePtr::Memory(dev)   => dev.handle_int(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusField {
    Acknowledge      = 1<<0,
    Driver           = 1<<1,
    DriverOk         = 1<<2,
    FeaturesOk       = 1<<3,
    DeviceNeedsReset = 1<<6,
    Failed           = 1<<7,
}
impl core::ops::BitOr<Self> for StatusField {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as Self::Output | rhs as Self::Output
    }
}
impl core::ops::BitOr<u32> for StatusField {
    type Output = u32;
    fn bitor(self, rhs: u32) -> Self::Output {
        self as Self::Output | rhs as Self::Output
    }
}
impl core::ops::BitOrAssign<StatusField> for u32 {
    fn bitor_assign(&mut self, rhs: StatusField) {
        *self |= rhs as Self
    }
}

pub fn handle_int(int: u32) {
    let idx = int-1;
    unsafe {
        let boxed_dev = VIRTIO_DEVICES[idx as usize].as_mut().expect("Should be set if we get an interrupt from there ?");
        boxed_dev.handle_int();
    }
}