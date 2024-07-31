pub fn test_read() {
    let mut blk = match unsafe { virtio::VIRTIO_DEVICES[7].as_mut().unwrap() } {
        virtio::VirtIODevicePtr::Block(blk) => blk,
        _ => todo!(),
    };
    let mut buffer = alloc::vec![0u8; 1024];
    blk.read(0, &mut buffer);
    let read = alloc::string::String::from_utf8(buffer).unwrap();
    
    let raw = include_str!("../disk.hdd");
    let included = &raw[0..1024];
    assert_eq!(read, included);
}
pub fn test_write() {
    let mut blk = match unsafe { virtio::VIRTIO_DEVICES[7].as_mut().unwrap() } {
        virtio::VirtIODevicePtr::Block(blk) => blk,
        _ => todo!(),
    };
    let mut write = alloc::vec![55u8; 1024];
    blk.write(0, &write);
    let mut read = alloc::vec![1u8; 1024];
    blk.read(0, &mut read);
    // Revert disk contents
    let raw = &include_bytes!("../disk.hdd")[0..1024];
    blk.write(0, raw);
    assert_eq!(write, read);
}
pub fn test_size() {
    let mut blk = match unsafe { virtio::VIRTIO_DEVICES[7].as_mut().unwrap() } {
        virtio::VirtIODevicePtr::Block(blk) => blk,
        _ => todo!(),
    };
    let sectors = include_bytes!("../disk.hdd").len().div_ceil(512);
    assert_eq!(blk.config.capacity, sectors as _);
}