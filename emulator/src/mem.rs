use std::cell::RefCell;

use color_eyre::eyre::{ContextCompat, Error, Result};
use color_eyre::Report;
use crate::{iguest, uguest};

// Inspired by QEMU
pub enum MemMap {
    DEBUG,
    MROM,
    TEST,
    RTC,
    CLINT,
    AclintSswi,
    PciePio,
    PlatformBus,
    PLIC,
    AplicM,
    AplicS,
    UART0,
    VIRTIO,
    FwCfg,
    FLASH,
    ImsicM,
    ImsicS,
    PcieEcam,
    PcieMmio,
    DRAM,
}
impl MemoryMap for MemMap {
    fn base(&self) -> uguest {
        match self {
            Self::DEBUG => 0x0,
            Self::MROM => 0x1000,
            Self::TEST => 0x100000,
            Self::RTC => 0x101000,
            Self::CLINT => 0x2000000,
            // Self::AclintSswi => 0x2F00000,
            // Self::PciePio => 0x3000000,
            // Self::PlatformBus => 0x4000000,
            Self::PLIC => 0xc000000,
            // Self::AplicM => 0xc000000,
            // Self::AplicS => 0xd000000,
            Self::UART0 => 0x10000000,
            Self::VIRTIO => 0x10001000,
            // Self::FwCfg => 0x10100000,
            Self::FLASH => 0x20000000,
            // Self::ImsicM => 0x24000000,
            // Self::ImsicS => 0x28000000,
            // Self::PcieEcam => 0x30000000,
            // Self::PcieMmio => 0x40000000,
            Self::DRAM => 0x8000_0000,
            _ => {todo!()}
        }
    }
    fn len(&self) -> uguest {
        match self {
            Self::DEBUG => 0x100,
            Self::MROM => 0xf000,
            Self::TEST => 0x1000,
            Self::RTC => 0x1000,
            Self::CLINT => 0x10000,
            // Self::AclintSswi => 0x4000,
            // Self::PciePio => 0x10000,
            // Self::PlatformBus => 0x2000000,
            Self::PLIC => 0x0C20_0000-Self::PLIC.base(),
            // Self::AplicM => APLIC_SIZE(VIRT_CPUS_MAX),
            // Self::AplicS => APLIC_SIZE(VIRT_CPUS_MAX),
            Self::UART0 => 0x100,
            Self::VIRTIO => 0x1000,
            // Self::FwCfg => 0x18,
            // Self::FLASH => 0x4000000,
            // Self::ImsicM => VIRT_IMSIC_MAX_SIZE,
            // Self::ImsicS => VIRT_IMSIC_MAX_SIZE,
            // Self::PcieEcam => 0x10000000,
            // Self::PcieMmio => 0x40000000,
            Self::DRAM => uguest::MAX-Self::DRAM.base(), // Max size
            _ => todo!(),
        }
    }
}

pub trait MemoryMap {
    fn base(&self) -> uguest;
    fn len(&self) -> uguest;
    fn end(&self) -> uguest {self.base()+self.len()}
    fn in_bounds(&self, offset: uguest, len:uguest) -> bool {
        offset>=self.base() && offset<self.end()
    }
}

pub trait MemoryRegion: MemoryMap {
    // Bounds checks must be done by caller
    unsafe fn read(&self, offset: uguest) -> u8;
    unsafe fn read_bytes<'a>(&'a self, offset: uguest, len: uguest) -> Vec<u8> {
        let mut out = Vec::with_capacity(len as _);
        for i in offset..offset+len {
            out.push_within_capacity(unsafe {self.read(offset+i)}).unwrap()
        }
        out
    }
    unsafe fn write(&mut self, offset: uguest, val: u8);
    unsafe fn write_bytes(&mut self, offset: uguest, buffer: &mut [u8]) {
        for (i,byte) in buffer.into_iter().enumerate() {
            unsafe {self.write(offset+i as uguest, *byte)}
        }
    }
    
    // unsafe fn read<'a>    (&mut self, offset: uguest) -> &'a u8;
    // unsafe fn read_mut<'a>(&mut self, offset: uguest) -> &'a mut u8;
}

#[repr(transparent)]
pub struct DRAM {
    inner: RefCell<Vec<u8>>,
}
impl MemoryMap for DRAM {
    fn base(&self) -> uguest {MemMap::DRAM.base()}
    fn len(&self) -> uguest {MemMap::DRAM.len()}
}
impl MemoryRegion for DRAM {
    unsafe fn read(&self, offset: uguest) -> u8 {
        if offset>=self.len() {return 0}
        self.inner.borrow()[offset as usize]
    }
    unsafe fn read_bytes<'a>(&'a self, offset: uguest, len: uguest) -> Vec<u8> {
        let inner_len = self.inner.borrow().len();
        if (offset+len)>=inner_len as uguest {
            self.inner.borrow_mut().append(&mut vec![0; ((offset+len)-inner_len as uguest) as usize]);
        }
        self.inner.borrow()[offset as usize..(offset+len) as usize].to_vec()
    }
    
    unsafe fn write(&mut self, offset: uguest, val: u8) {
        self.inner.borrow_mut().append(&mut vec![0; offset as usize+10]);
        self.inner.borrow_mut()[offset as usize] = val;
    }
    
    unsafe fn write_bytes(&mut self, offset: uguest, buffer: &mut [u8]) {
        if offset+buffer.len() as uguest>=self.inner.borrow().len() as uguest {
            self.inner.borrow_mut().append(&mut vec![0u8; buffer.len()]);
            dbg!(self.inner.borrow().len());
        }
        self.inner.borrow_mut()[offset as usize..offset as usize+buffer.len()].copy_from_slice(buffer)
    }
}
#[derive(Debug, Default)]
pub struct UART {
    
}
impl MemoryMap for UART {
    fn base(&self) -> uguest {MemMap::UART0.base()}
    fn len(&self) -> uguest {MemMap::UART0.len()}
}
impl MemoryRegion for UART {
    unsafe fn read(&self, offset: uguest) -> u8 {
        todo!()
    }

    unsafe fn write(&mut self, offset: uguest, val: u8) {
        if offset == 0 {
            print!("{}", val as char);
            std::io::Write::flush(&mut std::io::stdout()).unwrap()
        }
    }
}

pub struct Memory {
    dram: DRAM,
    uart: UART,
}
impl Memory {
    pub fn with_program(program: Vec<u8>) -> Self {
        Self {
            dram: DRAM{inner:RefCell::new(program)},
            uart: UART::default(),
        }
    }
    pub fn get_region(&mut self, offset: uguest, len:uguest) -> Result<&mut dyn MemoryRegion> {
        Ok(
        if offset>=MemMap::DRAM.base() {
            &mut self.dram
        } 
        else if MemMap::UART0.in_bounds(offset, len) {
            &mut self.uart
        }
        else {
            return Err(Report::msg(format!("Can't find region: {}-{}",offset, offset+len)))
        })
    }
    pub fn get<T: Copy>(&mut self, offset: uguest) -> Result<T> {
        let region = self.get_region(offset, core::mem::size_of::<T>() as _)?;
        let bytes = unsafe { region.read_bytes(offset-region.base(), core::mem::size_of::<T>() as _) };
        Ok(unsafe { *(bytes.as_ptr() as *const T) })
    }
    pub fn set<T>(&mut self, offset: uguest, mut val: T) -> Result<()> {
        let mut vec_val = unsafe { core::slice::from_raw_parts_mut(&mut val as *mut T as *mut u8, core::mem::size_of::<T>()) };
        let region = self.get_region(offset, core::mem::size_of::<T>() as _)?;
        unsafe { region.write_bytes(offset-region.base(), &mut vec_val) }
        Ok(())
    }

    // For u16, more efficient than `(program[5] as u16) << 8|program[4] as u16` ?
    pub fn read(&mut self, offset: uguest, len: uguest) -> Result<Vec<u8>> {
        // Getting mut self because It's a pain
        let region = self.get_region(offset, len)?;
        Ok(unsafe { region.read_bytes(offset-region.base(), len) })
    }
    pub fn write(&mut self, offset: uguest, buffer: &mut [u8]) -> Result<()> {
        let region = self.get_region(offset, buffer.len() as _)?;
        unsafe { region.write_bytes(offset-region.base(), buffer) }
        Ok(())
    }
}


// pub struct MemRegionsIter<'a> {
//     regions: &'a Memory,
//     curr: usize,
// }
// impl<'a> Iterator for MemRegionsIter<'a> {
//     type Item = &'a mut dyn MemoryRegion;

//     fn next(&mut self) -> Option<Self::Item> {
//         let region = &mut match self.curr { // Sorted from most used to less used
//             0 => self.regions.dram,
//             // 1 => self.regions.uart,
//             // 2 => self.regions.plic,
//             // 3 => self.regions.dram,
//             // 4 => self.regions.dram,
//             // 5 => self.regions.dram,
//             // 6 => self.regions.dram,
//             // 7 => self.regions.dram,
//             // 8 => self.regions.dram,
//             // 9 => self.regions.dram,
//             // 10 => self.regions.dram,
//             // 11 => self.regions.dram,
//             // 12 => self.regions.dram,
//             // 13 => self.regions.dram,
//             // 14 => self.regions.dram,
//             // 15 => self.regions.dram,
//             // 16 => self.regions.dram,
//             // 17 => self.regions.dram,
//             // 18 => self.regions.dram,
//             // 19 => self.regions.dram,
//             _ => todo!(),
//         };
//         self.curr += 1;
//         Some(region)
//     }
// }

