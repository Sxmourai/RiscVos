use super::*;

pub const SUPPORTED_FEATURES: u32 = !(BlockFeatures::ReadOnly as u32);
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockFeatures {
    /// Legacy:
    /// Device supports request barriers.
    VIRTIO_BLK_F_BARRIER = 1<<0,
    /// Maximum size of any single segment is in size_max.
    SIZE_MAX = 1<<1,
    /// Maximum number of segments in a request is in seg_max.
    SEG_MAX = 1<<2,
    /// Disk-style geometry specified in geometry.
    GEOMETRY = 1<<4,
    /// Device is read-only.
    ReadOnly = 1<<5,
    /// Block size of disk is in blk_size.
    BLK_SIZE = 1<<6,
    /// Legacy:
    /// Device supports scsi packet commands.
    VIRTIO_BLK_F_SCSI = 1<<7,
    /// Cache flush command support.
    FLUSH = 1<<9,
    /// Device exports information on optimal I/O alignment.
    TOPOLOGY = 1<<10,
    /// Device can toggle its cache between writeback and writethrough modes.
    CONFIG_WCE = 1<<11,
    /// Device can support discard command, maximum discard sectors size in max_discard_sectors and maximum discard segment number in max_discard_seg.
    DISCARD = 1<<13,
    /// Device can support write zeroes command, maximum write zeroes sectors size in max_write_zeroes_sectors and maximum write zeroes segment number in max_write_zeroes_seg.
    WRITE_ZEROES = 1<<14,
}
pub fn dbg_features(f: u32) {
    let variants = [
        BlockFeatures::VIRTIO_BLK_F_BARRIER,
        BlockFeatures::SIZE_MAX,
        BlockFeatures::SEG_MAX,
        BlockFeatures::GEOMETRY,
        BlockFeatures::ReadOnly,
        BlockFeatures::BLK_SIZE,
        BlockFeatures::VIRTIO_BLK_F_SCSI,
        BlockFeatures::FLUSH,
        BlockFeatures::TOPOLOGY,
        BlockFeatures::CONFIG_WCE,
        BlockFeatures::DISCARD,
        BlockFeatures::WRITE_ZEROES,
    ];
    for v in variants {
        if f & v as u32 != 0 {
            print!("{:?} ", v);
        }
    }
}

pub fn init_device(mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    let blk = block::BlockDevice::new(mmio)?;
    log::info!("Found disk with size {}kb", blk.config.capacity*blk.config.blk_size as u64/1024);
    Some(VirtIODevicePtr::Block(Box::new(blk)))
}


#[derive(Debug)]
pub struct BlockDevice {
    mmio: StandardVirtIO,
    pub config: &'static mut Config,
    queue: Option<&'static mut Queue>,
    queue_idx: usize,
    // pendings: Vec<&'static [u8]>, // Not static, but gonna change it, but lifetime is not the right move for now
}
impl BlockDevice {
    pub fn new(mmio: StandardVirtIO) -> Option<Self> {
        let config = unsafe{&mut *((mmio.base()+MmioOffset::Config as usize) as *mut Config)};
        let mut _s = Self {
            mmio,
            queue: None,
            queue_idx: 0,
            config,
            // pendings: Vec::new(),
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
        let num_pages = size_of::<Queue>().div_ceil(PAGE_SIZE);
        // Allocate 1 page for every device, but we will still be using the MMIO registers
        // ! Don't forget to use memory fences, cuz we need to have finished writing to memory before issuing a notify 
        unsafe { self.mmio.write(MmioOffset::QueueSel, 0) };

        //TODO Alignment
        unsafe { self.mmio.write(MmioOffset::DriverPageSize, PAGE_SIZE.try_into().unwrap()) };
        // QueuePFN is a physical page number, however it
        // appears for QEMU we have to write the entire memory
        // address.
        let mut addr = kalloc(2).ok()?;
        unsafe {
            let _ = self.queue.replace(&mut *(addr as *mut Queue));
            self.mmio.write(MmioOffset::QueuePfn, (addr as u64 / PAGE_SIZE64) as u32);
            self.mmio.set_driver_status_bit(StatusField::DriverOk);
        }
        Some(())
    }
    fn op(&mut self, sector_offset: u64, buffer: &[u8], writing: bool) -> Option<()> {
        assert_eq!(buffer.len()%512, 0);
        if self.mmio.read_only && writing {return None;}
        let request = unsafe{&mut *(kmalloc(core::mem::size_of::<Request>()).ok()? as *mut Request)};
        request.header.sector = sector_offset;
        request.header.blktype = if writing {VIRTIO_BLK_T_OUT} else {VIRTIO_BLK_T_IN};
        request.header.reserved = 0;
        
        request.data = core::ptr::addr_of!(buffer[0]) as _;
        // We put 111 in the status. Whenever the device finishes, it will write into
        // status. If we read status and it is 111, we know that it wasn't written to by
        // the device.
        request.status = 111;
        let desc = Descriptor {
            addr: core::ptr::addr_of!(request.header) as u64,
            len:  core::mem::size_of::<Header>() as _,
            flags:VIRTIO_DESC_F_NEXT,
            next: 0,
        };
        unsafe {
            self.fill_next_descriptor(desc)?;
        }

        let head_idx = self.queue_idx.clone();

        let mut flags = VIRTIO_DESC_F_NEXT;
        if !writing {
            flags |= VIRTIO_DESC_F_WRITE
        }
        let data_desc = Descriptor { 
            addr: core::ptr::addr_of!(buffer[0]) as u64,
            len: buffer.len() as u32,
            flags,
            next: 0,
        };
        unsafe {self.fill_next_descriptor(data_desc)}?;
        let data_idx = self.queue_idx;
        let status_desc = Descriptor {
            addr:  core::ptr::addr_of!(request.status) as u64,
            len:   1,
            flags: VIRTIO_DESC_F_WRITE,
            next:  0, 
        };
        unsafe {self.fill_next_descriptor(status_desc)}?;
        let status_idx = self.queue_idx;
        let mut queue = self.queue.as_mut()?;
        queue.avail.ring[queue.avail.idx as usize] = head_idx as u16;
        queue.avail.idx = (((queue.avail.idx as usize) + 1) % VIRTIO_RING_SIZE) as _;
        // The only queue a block device has is 0, which is the request
        // queue.
        unsafe {self.mmio.write(MmioOffset::QueueNotify, 0);}
        Some(())
    }
    unsafe fn fill_next_descriptor(&mut self, desc: Descriptor) -> Option<()> {
		// The ring structure increments here first. This allows us to skip
		// index 0, which then in the used ring will show that .id > 0. This
		// is one way to error check. We will eventually get back to 0 as
		// this index is cyclical. However, it shows if the first read/write
		// actually works.
        let mut queue = self.queue.as_mut()?;
        self.queue_idx = (self.queue_idx + 1) % VIRTIO_RING_SIZE;
        queue.desc[self.queue_idx] = desc;
        if queue.desc[self.queue_idx].flags & VIRTIO_DESC_F_NEXT != 0 {
            // If the next flag is set, we need another descriptor.
            queue.desc[self.queue_idx].next = ((self.queue_idx + 1) % VIRTIO_RING_SIZE) as _;
        }
        Some(())
    }
    /// Reads size in bytes and returns a vector of the values as T
    pub fn read(&mut self, sector_offset: u64, buffer: &mut [u8]) -> Option<()> {
        self.op(sector_offset, buffer, false)?;
        Some(())
    }
    
    pub fn write(&mut self, sector_offset: u64, buffer: &[u8]) -> Option<()> {
        self.op(sector_offset, buffer, true)?;
        Some(())
    }
}
impl VirtIODevice for BlockDevice {
    fn handle_int(&mut self) {
        let queue = self.queue.as_mut().unwrap();
        let head_idx = queue.avail.ring[queue.avail.idx as usize-1];
        let rq_addr = queue.desc[head_idx as usize].addr;
        let rq = unsafe {&mut *(rq_addr as *mut Request)};
        match rq.status {
            0 => {}, // Success
            1 => {todo!()}, // IO Error
            2 => {todo!()}, // Unsupported op
            111 => {log::warn!("Not ");}
            status => todo!("Invalid status: {}", status)
        };
        // todo!()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Geometry {
	cylinders: u16,
	heads:     u8,
	sectors:   u8,
}

#[repr(C)]
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Config {
	pub capacity:                 u64,
	pub size_max:                 u32,
	pub seg_max:                  u32,
	pub geometry:                 Geometry,
	pub blk_size:                 u32,
	pub topology:                 Topology,
	pub writeback:                u8,
	unused0:                  [u8; 3],
	pub max_discard_sector:       u32,
	pub max_discard_seg:          u32,
	pub discard_sector_alignment: u32,
	pub max_write_zeroes_sectors: u32,
	pub max_write_zeroes_seg:     u32,
	pub write_zeroes_may_unmap:   u8,
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
#[derive(Debug)]
pub struct Header {
	blktype:  u32,
	reserved: u32,
	sector:   u64,
}
#[repr(C)]
#[derive(Debug)]
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
