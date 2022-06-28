use core::convert::TryFrom;
use qemu_fw_cfg::FwCfg;
use qemu_fw_cfg::FwCfgFile;
use qemu_fw_cfg::FwCfgWriteError;

// https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/ui/qemu-pixman.c#L96-98
// https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/include/standard-headers/drm/drm_fourcc.h#L152-161
#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    /// 3 bytes per pixel: blue, green, red
    #[allow(unused)]
    B8G8R8,
    /// 4 bytes per pixel: blue, green, red, (unused)
    #[allow(unused)]
    B8G8R8X8,
    /// 4 bytes per pixel: blue, green, red, alpha (maybe unused?)
    #[allow(unused)]
    B8G8R8A8,
}

pub struct RamFbConfig {
    pub pixel_format: PixelFormat,
    pub width_pixels: usize,
    pub height_pixels: usize,
    pub stride_bytes: usize,
}

#[derive(Debug)]
pub enum RamFbError {
    DeviceNotFound,
    FwCfgWriteError(FwCfgWriteError),
}

pub unsafe fn configure<Pixel>(
    fw_cfg: &mut FwCfg,
    config: &RamFbConfig,
    buffer: *const Pixel,
) -> Result<(), RamFbError> {
    if let Some(device) = fw_cfg.find_file("etc/ramfb") {
        configure_device(fw_cfg, &device, config, buffer).map_err(RamFbError::FwCfgWriteError)
    } else {
        Err(RamFbError::DeviceNotFound)
    }
}

pub unsafe fn configure_device<Pixel>(
    fw_cfg: &mut FwCfg,
    device: &FwCfgFile,
    config: &RamFbConfig,
    buffer: *const Pixel,
) -> Result<(), FwCfgWriteError> {
    let fourcc = *match config.pixel_format {
        PixelFormat::B8G8R8 => b"RG24",
        PixelFormat::B8G8R8X8 => b"AR24",
        PixelFormat::B8G8R8A8 => b"XR24",
    };
    let ramfb_cfg = RAMFBCfg {
        address: (buffer as u64).to_be(),
        fourcc: u32::from_le_bytes(fourcc).to_be(),
        flags: 0,
        width: u32::try_from(config.width_pixels).unwrap().to_be(),
        height: u32::try_from(config.height_pixels).unwrap().to_be(),
        stride: u32::try_from(config.stride_bytes).unwrap().to_be(),
    };
    let as_ptr = &ramfb_cfg as *const RAMFBCfg as *const u8;
    let len = 28; // without padding
    let as_bytes = core::slice::from_raw_parts(as_ptr, len);
    fw_cfg.write_to_file(&device, as_bytes)
}

// Memory layout must match this exactly:
// https://gitlab.com/qemu-project/qemu/-/blob/v7.0.0/hw/display/ramfb.c#L22-29
#[repr(C)]
struct RAMFBCfg {
    address: u64,
    fourcc: u32,
    flags: u32,
    width: u32,
    height: u32,
    stride: u32,
}
