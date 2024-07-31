use super::*;

pub const SUPPORTED_FEATURES: u32 = !0;

pub fn init_device(mmio: StandardVirtIO) -> Option<VirtIODevicePtr> {
    let dev = GpuDevice {mmio};
    log::warn!("TODO Gpu device !");
    // dev.command(VIRTIO_GPU_CMD_GET_DISPLAY_INFO);
    Some(VirtIODevicePtr::Gpu(Box::new(dev)))}

pub struct GpuDevice {
    mmio: StandardVirtIO,
}

pub enum GpuFeatures {
    /// virgl 3D mode is supported
    VirGL = 1<<0,
    EDID = 1<<1,
}

#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum GpuCtrlType {
    /// 2d commands
    CMD_GET_DISPLAY_INFO = 0x0100, 
    CMD_RESOURCE_CREATE_2D, 
    CMD_RESOURCE_UNREF, 
    CMD_SET_SCANOUT, 
    CMD_RESOURCE_FLUSH, 
    CMD_TRANSFER_TO_HOST_2D, 
    CMD_RESOURCE_ATTACH_BACKING, 
    CMD_RESOURCE_DETACH_BACKING, 
    CMD_GET_CAPSET_INFO, 
    CMD_GET_CAPSET, 
    CMD_GET_EDID, 

    /// cursor commands
    CMD_UPDATE_CURSOR = 0x0300, 
    CMD_MOVE_CURSOR, 

    /// success responses
    RESP_OK_NODATA = 0x1100, 
    RESP_OK_DISPLAY_INFO, 
    RESP_OK_CAPSET_INFO, 
    RESP_OK_CAPSET, 
    RESP_OK_EDID, 

    /// error responses
    RESP_ERR_UNSPEC = 0x1200, 
    RESP_ERR_OUT_OF_MEMORY, 
    RESP_ERR_INVALID_SCANOUT_ID, 
    RESP_ERR_INVALID_RESOURCE_ID, 
    RESP_ERR_INVALID_CONTEXT_ID, 
    RESP_ERR_INVALID_PARAMETER, 
}
#[repr(C)]
/// All requests and responses on the virt queues have the fixed header struct virtio_gpu_ctrl_hdr.
pub struct GpuCtrlHeader {
    /// Specifies the type of the driver request (VIRTIO_GPU_CMD_*) or device response (VIRTIO_GPU_RESP_*).
    /// On success the device will return VIRTIO_GPU_RESP_OK_NODATA in case there is no payload. Otherwise the type field will indicate the kind of payload
    /// On error the device will return one of the VIRTIO_GPU_RESP_ERR_* error codes
    ty: GpuCtrlType,
    /// request / response flags
    flags: u32,
    /// If the driver sets the VIRTIO_GPU_FLAG_FENCE bit in the request flags field the device MUST:
    /// - set VIRTIO_GPU_FLAG_FENCE bit in the response,
    /// - copy the content of the fence_id field from the request to the response, and
    /// - send the response only after command processing is complete
    fence_id: u64,
    /// Rendering context (used in 3D mode only).
    ctx_id: u32,
    padding: u32,
}

/// Display configuration has changed. The driver SHOULD use the VIRTIO_GPU_CMD_GET_DISPLAY_INFO command to fetch the information from the device
const EVENT_DISPLAY: u32 = 1<<0;
const FLAG_FENCE: u32 = 1 << 0;
const MAX_SCANOUTS: u32 = 16;
#[repr(C)]
pub struct GpuConfig {
    /// Signals pending events to the driver. 
    /// Read-Only
    events_read: u32,
    /// Clears pending events in the device. 
    /// Writing a ’1’ into a bit will clear the corresponding bit in events_read, mimicking write-to-clear behavior
    events_clear: u32,
    /// Specifies the maximum number of scanouts supported by the device. Minimum value is 1, maximum value is 16
    num_scanouts: u32, 
    reserved: u32,
}

impl VirtIODevice for GpuDevice {
    fn handle_int(&mut self) {
        todo!()
    }
}