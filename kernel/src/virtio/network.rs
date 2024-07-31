use super::*;

pub const SUPPORTED_FEATURES: u32 = 0;

pub fn init_device(mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    let dev = NetworkDevice {mmio};
    log::warn!("TODO Network device");
    Some(VirtIODevicePtr::Network(Box::new(dev)))
}

pub struct NetworkDevice {
    mmio: StandardVirtIO,
}
impl VirtIODevice for NetworkDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}