use super::*;

pub const SUPPORTED_FEATURES: u32 = 0;

pub fn init_device(_mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    todo!()
}

pub struct EntropyDevice {}
impl VirtIODevice for EntropyDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}
