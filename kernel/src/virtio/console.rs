use super::*;

pub const SUPPORTED_FEATURES: u32 = 0;

pub fn init_device(_mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    Some(todo!())
}


pub struct ConsoleDevice {
    
}
impl VirtIODevice for ConsoleDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}