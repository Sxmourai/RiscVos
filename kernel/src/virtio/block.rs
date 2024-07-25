use super::*;

#[derive(Debug)]
pub struct BlockDevice {
    mmio: StandardVirtIO,
    queue: Option<&'static mut Queue>,
    queue_idx: usize,
    // pendings: Vec<&'static [u8]>, // Not static, but gonna change it, but lifetime is not the right move for now
}
impl BlockDevice {
    pub fn new(mmio: StandardVirtIO) -> Option<Self> {
        let mut _s = Self {
            mmio,
            queue: None,
            queue_idx: 0,
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
        unsafe { self.mmio.write(MmioOffset::GuestPageSize, PAGE_SIZE.try_into().unwrap()) };
        // QueuePFN is a physical page number, however it
        // appears for QEMU we have to write the entire memory
        // address.
        let mut addr = kalloc(1).ok()?;
        unsafe {
            let _ = self.queue.replace(&mut *(addr as *mut Queue));
            self.mmio.write(MmioOffset::QueuePfn, (addr as u64 / PAGE_SIZE64) as u32);
            self.mmio.write(MmioOffset::Status, StatusField::DriverOk | self.mmio.read(MmioOffset::Status));
        }
        Some(())
    }
    fn op(&mut self, sector_offset: u64, buffer: &[u8], writing: bool) -> Option<()> {
        assert_eq!(buffer.len()%512, 0);
        if self.mmio.read_only && writing {return None;}
        let request = unsafe{&mut *(kmalloc(core::mem::size_of::<Request>()).ok()? as *mut Request)};
        let desc = Descriptor {
            addr: unsafe { &request.header } as *const Header as u64, // Isn't it zero ?
            len:  core::mem::size_of::<Header>() as _,
            flags:VIRTIO_DESC_F_NEXT,
            next: 0,
        };
        unsafe {
            self.fill_next_descriptor(desc)?;
        }
        request.header.sector = sector_offset;
        request.header.blktype = if writing {VIRTIO_BLK_T_OUT} else {VIRTIO_BLK_T_IN};
        request.header.reserved = 0;
        
        request.data = buffer[0] as *mut _;
        // We put 111 in the status. Whenever the device finishes, it will write into
        // status. If we read status and it is 111, we know that it wasn't written to by
        // the device.
        request.status = 111; // Not 0b111 but One hundred and one !
        let head_idx = self.queue_idx;

        let mut flags = VIRTIO_DESC_F_NEXT;
        if writing {
            flags |= VIRTIO_DESC_F_WRITE
        }
        let data_desc = Descriptor { 
            addr: (buffer[0] as *mut u8) as u64,
            len:(buffer.len()/512) as u32,
            flags,
            next: 0, 
        };
        unsafe {self.fill_next_descriptor(data_desc)}?;
        let data_idx = self.queue_idx;
        let status_desc = Descriptor {
            addr:  unsafe { &request.status } as *const u8 as u64,
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
        dbg!(self);
        for i in 0..100000 {}
        unsafe {self.mmio.write(MmioOffset::QueueNotify, 0);}
        for i in 0..100000 {}
        dbg!(self);
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
        dbg!(self.queue);
        Some(())
    }
    
    pub fn write(&mut self, sector_offset: u64, buffer: &[u8]) -> Option<()> {
        self.op(sector_offset, buffer, true)?;
        Some(())
    }
}
impl VirtIODevice for BlockDevice {
    fn handle_int(&mut self) {
        dbg!(self.queue_idx, self.queue);
        dbg!(self);
        let mut queue = self.queue.as_mut().unwrap();
        dbg!(queue);
    }
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
