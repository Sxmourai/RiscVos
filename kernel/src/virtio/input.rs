use super::*;

pub const SUPPORTED_FEATURES: u32 = 0;

pub fn init_device(mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    Some(todo!())
}

pub struct InputDevice {}
impl VirtIODevice for InputDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}
