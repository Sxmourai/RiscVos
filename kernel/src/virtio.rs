use log::{info, warn};

use crate::{dbg, kalloc, paging::{PAGE_SIZE, PAGE_SIZE64}};

/// QEMU SPECIFIC
pub const VIRTIO_START: usize = 0x1000_1000;
pub const VIRTIO_END: usize =   0x1000_8000;
pub const VIRTIO_STRIDE: usize = 0x1000;
pub const VIRTIO_MAGIC: u32 = 0x74_72_69_76; // "virt" in little endian
pub static mut VIRTIO_DEVICES: [Option<alloc::boxed::Box<VirtIODevice>>; (VIRTIO_END-VIRTIO_START)/VIRTIO_STRIDE] = [None; _];


pub fn init() {
    crate::info!("Probing Virtio devices...");
    for addr in (VIRTIO_START..=VIRTIO_END).step_by(VIRTIO_STRIDE) {
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
            // https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html#x1-1930005
            if let Some(device) = match deviceid {
                2 => {block_device(idx)},
                4 => {todo!()},
                _ => {todo!("support this device: {}", deviceid)},
            } {
                unsafe { VIRTIO_DEVICES[idx].replace(alloc::boxed::Box::new(device)); }
            }

        }
    }
}

pub fn block_device(idx: usize) -> Option<BlockDevice> {
    info!("Found block device at {:#x}", idx);
    
    None
}

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

#[repr(C)]
pub struct UsedElem {
	pub id:  u32,
	pub len: u32,
}

#[repr(C)]
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
	pub padding0: [u8; PAGE_SIZE - size_of::<Descriptor>() * VIRTIO_RING_SIZE - size_of::<Available>()],
	pub used:     Used,
}
pub struct BlockDevice {
    mmio: StandardVirtIO,
    queue: Option<*mut Queue>,
}
impl BlockDevice {
    pub fn new(mmio: StandardVirtIO) -> Option<Self> {
        let mut _s = Self {
            mmio,
            queue: None,
        };
        _s.specific_init()?;
        Some(_s)
    }
    fn specific_init(&mut self) -> Option<()> {
        // Set the queue num. We have to make sure that the
        // queue size is valid because the device can only take
        // a certain size.
        let qnmax = self.mmio.read(MmioOffset::QueueNumMax);
        if VIRTIO_RING_SIZE as u32 > qnmax {
            warn!("Setting queue size failed, max: {}", qnmax);
            return None;
        }
        unsafe {
            self.mmio.write(MmioOffset::QueueNum, VIRTIO_RING_SIZE as u32);
        }
        // We add 4095 to round this up and then do an integer
        // divide to truncate the decimal. We don't add 4096,
        // because if it is exactly 4096 bytes, we would get two
        // pages, not one.
        let num_pages = (size_of::<Queue>() + PAGE_SIZE - 1) / PAGE_SIZE;
        // Allocate 1 page for every device, but we will still be using the MMIO registers
        // ! Don't forget to use memory fences, cuz we need to have finished writing to memory before issuing a notify 
        dbg!(num_pages);
        unsafe { self.mmio.write(MmioOffset::QueueSel, 0) };

        //TODO Alignment
        let self.queue = kalloc(1) as *mut Queue;
        unsafe { self.mmio.write(MmioOffset::GuestPageSize, PAGE_SIZE.try_into().unwrap()) };
        // QueuePFN is a physical page number, however it
        // appears for QEMU we have to write the entire memory
        // address.
        unsafe {
            self.mmio.write(MmioOffset::QueuePfn, (queue as u64 / PAGE_SIZE64) as u32);
            self.mmio.write(MmioOffset::Status, self.mmio.read(MmioOffset::Status) | StatusField::DriverOk);
        }
        Some(())
    }
    fn op<T: Sized>(&mut self, sector_offset: usize, write: bool) -> Option<alloc::vec::Vec<T>> {
        if self.mmio.read_only && write {return None;}
        
        Some(buffer)
    }
}
impl VirtIODevice for BlockDevice {

}
#[repr(C)]
pub struct Geometry {
	cylinders: u16,
	heads:     u8,
	sectors:   u8,
}

#[repr(C)]
pub struct Topology {
	physical_block_exp: u8,
	alignment_offset:   u8,
	min_io_size:        u16,
	opt_io_size:        u32,
}

// There is a configuration space for VirtIO that begins
// at offset 0x100 and continues to the size of the configuration.
// The structure below represents the configuration for a
// block device. Really, all that this OS cares about is the
// capacity.
#[repr(C)]
pub struct Config {
	capacity:                 u64,
	size_max:                 u32,
	seg_max:                  u32,
	geometry:                 Geometry,
	blk_size:                 u32,
	topology:                 Topology,
	writeback:                u8,
	unused0:                  [u8; 3],
	max_discard_sector:       u32,
	max_discard_seg:          u32,
	discard_sector_alignment: u32,
	max_write_zeroes_sectors: u32,
	max_write_zeroes_seg:     u32,
	write_zeroes_may_unmap:   u8,
	unused1:                  [u8; 3],
}

// The header/data/status is a block request
// packet. We send the header to tell the direction
// (blktype: IN/OUT) and then the starting sector
// we want to read. Then, we put the data buffer
// as the Data structure and finally an 8-bit
// status. The device will write one of three values
// in here: 0 = success, 1 = io error, 2 = unsupported
// operation.
#[repr(C)]
pub struct Header {
	blktype:  u32,
	reserved: u32,
	sector:   u64,
}
#[repr(C)]
pub struct Request {
	header: Header,
	data:   *mut u8,
	status: u8,
	head:   u16,
}
// Type values
pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;
pub const VIRTIO_BLK_T_FLUSH: u32 = 4;
pub const VIRTIO_BLK_T_DISCARD: u32 = 11;
pub const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13;

// Status values
pub const VIRTIO_BLK_S_OK: u8 = 0;
pub const VIRTIO_BLK_S_IOERR: u8 = 1;
pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;

// Feature bits
pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1;
pub const VIRTIO_BLK_F_SEG_MAX: u32 = 2;
pub const VIRTIO_BLK_F_GEOMETRY: u32 = 4;
pub const VIRTIO_BLK_F_RO: u32 = 5;
pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 6;
pub const VIRTIO_BLK_F_FLUSH: u32 = 9;
pub const VIRTIO_BLK_F_TOPOLOGY: u32 = 10;
pub const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11;
pub const VIRTIO_BLK_F_DISCARD: u32 = 13;
pub const VIRTIO_BLK_F_WRITE_ZEROES: u32 = 14;




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
        let guest_features = host_features & !(1 << VIRTIO_BLK_F_RO);
        let ro = host_features & (1 << VIRTIO_BLK_F_RO) != 0;
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
pub enum DeviceType {
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4,
    Gpu = 16,
    Input = 18,
    Memory = 24,
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
impl core::ops::BitOr for StatusField {
    type Output = u32;
    fn bitor(self, rhs: Self) -> Self::Output {
        self as Self::Output | rhs as Self::Output
    }
}
impl core::ops::BitOrAssign<StatusField> for u32 {
    fn bitor_assign(&mut self, rhs: StatusField) {
        *self |= rhs as Self
    }
}