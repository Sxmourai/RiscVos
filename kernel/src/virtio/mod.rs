pub mod network;
pub mod block;
pub mod console;
pub mod entropy;
pub mod gpu;
pub mod input;
pub mod memory;

use alloc::{boxed::Box, format, string::String, vec::{Vec}};
use log::warn;

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
        unsafe{map!(addr)};
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
            init_virt_device(addr, deviceid);
        }
    }
}
/// Inits a V1 virtio device
fn init_virt_device(addr: usize, deviceid: u32) -> Option<()> {
    let idx = (addr - VIRTIO_START) / VIRTIO_STRIDE;
    let mut mmio = StandardVirtIO { idx, read_only: false };
    if mmio.read(MmioOffset::Version) != 1 {
        todo!()
    }
    // https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html#x1-1930005
    let supported_feats = VirtIODevicePtr::supported_features(deviceid);
    init_device(&mut mmio, supported_feats)?;
    let dev = match deviceid {
        1 =>  {network::init_device(mmio)},
        2 =>  {block::init_device(mmio)},
        3 =>  {console::init_device(mmio)},
        4 =>  {entropy::init_device(mmio)},
        16 => {gpu::init_device(mmio)},
        18 => {input::init_device(mmio)},
        24 => {memory::init_device(mmio)},
        _ =>  {todo!("Not supported {}", deviceid)},
    }?;

    unsafe { VIRTIO_DEVICES[idx].replace(dev); };
    Some(())
}
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
        .field("ring", &self.ring.iter().fold(String::new(), |mut s,x| {s.push_str(&format!("{x} "));s}))
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
        .field("addr", &(self as *const Queue))
        .field("desc", &format_args!("{:?}", self.desc.iter().filter(|x| x.addr != 0).collect::<Vec<&Descriptor>>()))
        .field("avail", &self.avail)
        // .field("padding0", &self.padding0)
        .field("used", &self.used)
        .finish()
    }
}



/// TODO Return result
fn init_device(dev: &mut StandardVirtIO, supported_drv_features: u32) -> Option<()> {
    unsafe {
        dev.write(MmioOffset::Status, 0); // Reset device
        dev.set_driver_status_bit(StatusField::Acknowledge);
        dev.set_driver_status_bit(StatusField::DriverOk);
        dev.set_features(supported_drv_features);
        // let ro = host_features & (1 << block::VIRTIO_BLK_F_RO) != 0;
        
        // Re-read status to ensure FEATURES_OK is still set.
        // Otherwise, it doesn't support our features.
        if dev.read(MmioOffset::Status) & (StatusField::FeaturesOk as u32) == 0 {
            log::warn!("Device {} doesn't support our features :c ", dev.idx);
            dev.set_driver_status_bit(StatusField::Failed as _);
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
        unsafe{(reg.ptr()).byte_add(self.base()).write_volatile(value)}
    }
    fn read(&self, reg: MmioOffset) -> u32 {
        unsafe{(reg.ptr()).byte_add(self.base()).read_volatile()}
    }
    /// The driver MUST NOT clear a device status bit. If the driver sets the FAILED bit, the driver MUST later reset the device before attempting to re-initialize
    pub fn set_driver_status_bit(&mut self, field: StatusField) {
        unsafe{self.write(MmioOffset::Status, self.read(MmioOffset::Status) | field as u32)}
    }
    /// Sets subset of the features supported by device and driver. 
    /// returns the intersection of both.
    /// **You must check the FeaturesOk to see if it is still set, if no the features are not supported**
    pub fn set_features(&mut self, supported_drv_features: u32) -> u32 {
        let dev_features = self.read(MmioOffset::DeviceFeatures);
        let drv_features = dev_features & supported_drv_features;
        unsafe { self.write(MmioOffset::DriverFeatures, drv_features) };
        self.set_driver_status_bit(StatusField::FeaturesOk);
        drv_features
    }
}

pub trait VirtIODevice {
    fn handle_int(&mut self);
}

/// Guest is Driver and Host is device
pub enum MmioOffset {
    /// Read-only
    MagicValue = 0x000,
    /// 0x2. Note: Legacy devices (see 4.2.4 Legacy interface) used 0x1.
    /// Read-only
    Version = 0x004,
    /// Read-only
    DeviceId = 0x008,
    /// Read-only
    VendorId = 0x00c,
    /// Reading from this register returns 32 consecutive flag bits, the least significant bit depending on the last value written to DeviceFeaturesSel. 
    /// Access to this register returns bits DeviceFeaturesSel ∗ 32 to (DeviceFeaturesSel ∗ 32) + 31, eg. feature bits 0 to 31 if DeviceFeaturesSel is 
    /// set to 0 and features bits 32 to 63 if DeviceFeaturesSel is set to 1. Also see 2.2 Feature Bits.
    /// Read-only
    DeviceFeatures = 0x010,
    /// Writing to this register selects a set of 32 device feature bits accessible by reading from DeviceFeatures.
    /// Write-only
    DeviceFeaturesSel = 0x014,
    /// Writing to this register sets 32 consecutive flag bits, the least significant bit depending on the last value written to DriverFeaturesSel. 
    /// Access to this register sets bits DriverFeaturesSel ∗ 32 to (DriverFeaturesSel ∗ 32) + 31, eg. feature bits 0 to 31 if DriverFeaturesSel is *
    /// set to 0 and features bits 32 to 63 if DriverFeaturesSel is set to 1. Also see 2.2 Feature Bits.
    /// Write-only
    DriverFeatures = 0x020,
    /// Writing to this register selects a set of 32 activated feature bits accessible by writing to DriverFeatures.
    /// Write-only
    DriverFeaturesSel = 0x024,
    DriverPageSize = 0x028,
    /// Writing to this register selects the virtual queue that the following operations on 
    /// QueueNumMax, QueueNum, QueueReady, QueueDescLow, QueueDescHigh, QueueAvailLow, QueueAvailHigh, QueueUsedLow and QueueUsedHigh apply to. 
    /// The index number of the first queue is zero (0x0).
    /// Write-only
    QueueSel = 0x030,
    /// Reading from the register returns the maximum size (number of elements) of the queue the device is ready to process or zero (0x0) if the queue is not available. 
    /// This applies to the queue selected by writing to QueueSel.
    /// Read-only
    QueueNumMax = 0x034,
    QueueNum = 0x038,
    QueueAlign = 0x03c,
    QueuePfn = 0x040,
    QueueNotify = 0x050,
    /// Reading from this register returns a bit mask of events that caused the device interrupt to be asserted. The following events are possible:
    /// Used Buffer Notification
    /// - bit 0 - the interrupt was asserted because the device has used a buffer in at least one of the active virtual queues.
    ///   Configuration Change Notification
    /// - bit 1 - the interrupt was asserted because the configuration of the device has changed
    ///   Read-only
    InterruptStatus = 0x060,
    /// Writing a value with bits set as defined in InterruptStatus to this register notifies the device that events causing the interrupt have been handled
    /// Write-only
    InterruptAck = 0x064,
    /// Reading from this register returns the current device status flags. Writing non-zero values to this register sets the status flags, indicating the driver progress. 
    /// Writing zero (0x0) to this register triggers a device reset. See also p. 4.2.3.1 Device Initialization.
    /// Read-Write
    Status = 0x070,
    /// Device-specific configuration space starts at the offset 0x100 and is accessed with byte alignment. 
    /// Its meaning and size depend on the device and the driver.
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
    pub fn supported_features(deviceid: u32) -> u32 {
        match deviceid {
            1 =>  {network::SUPPORTED_FEATURES},
            2 =>  {block::SUPPORTED_FEATURES},
            3 =>  {console::SUPPORTED_FEATURES},
            4 =>  {entropy::SUPPORTED_FEATURES},
            16 => {gpu::SUPPORTED_FEATURES},
            18 => {input::SUPPORTED_FEATURES},
            24 => {memory::SUPPORTED_FEATURES},
            _ => {todo!("Not supported {}", deviceid)},
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusField {
    /// Indicates that the guest OS has found the device and recognized it as a valid virtio device.
    Acknowledge      = 1<<0,
    /// Indicates that the guest OS knows how to drive the device. Note: There could be a significant (or infinite) delay before setting this bit. For example, under Linux, drivers can be loadable modules
    Driver           = 1<<1,
    /// Indicates that the driver is set up and ready to drive the device
    DriverOk         = 1<<2,
    /// Indicates that the driver has acknowledged all the features it understands, and feature negotiation is complete
    FeaturesOk       = 1<<3,
    /// Indicates that the device has experienced an error from which it can’t recover
    DeviceNeedsReset = 1<<6,
    /// Indicates that something went wrong in the guest, and it has given up on the device. This could be an internal error, or the driver didn’t like the device for some reason, or even a fatal error during device operation
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