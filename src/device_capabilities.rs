use windows::Win32::Devices::HumanInterfaceDevice::DIDEVCAPS;

#[derive(Debug)]
pub struct DeviceCapabilities {
    pub flags: u32,
    pub dev_type: u32,
    pub axes: u32,
    pub buttons: u32,
    pub povs: u32,
    pub ff_sample_period: u32,
    pub ff_min_time_resolution: u32,
    pub firmware_revision: u32,
    pub hardware_revision: u32,
    pub ff_driver_version: u32,
}

impl DeviceCapabilities {
    pub(crate) fn from_instance(caps: DIDEVCAPS) -> Self {
        Self {
            flags: caps.dwFlags,
            dev_type: caps.dwDevType,
            axes: caps.dwAxes,
            buttons: caps.dwButtons,
            povs: caps.dwPOVs,
            ff_sample_period: caps.dwFFSamplePeriod,
            ff_min_time_resolution: caps.dwFFMinTimeResolution,
            firmware_revision: caps.dwFirmwareRevision,
            hardware_revision: caps.dwHardwareRevision,
            ff_driver_version: caps.dwFFDriverVersion,
        }
    }
}
