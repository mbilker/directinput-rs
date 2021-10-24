use std::ffi::c_void;

use windows::Interface;

use crate::bindings::Windows::Win32::Devices::HumanInterfaceDevice::{
    DirectInput8Create, IDirectInput8W, IDirectInputDevice8W, DI8DEVCLASS_GAMECTRL,
    DIDEVICEINSTANCEW, DIEDFL_ALLDEVICES, DIENUM_CONTINUE, DIRECTINPUT_VERSION,
};
use crate::bindings::Windows::Win32::Foundation::{BOOL, HINSTANCE};
use crate::device::Device;
use crate::device_info::DirectInputDeviceInfo;
use crate::error::{DirectInputError, Result};

#[derive(Debug)]
pub struct DirectInputManager {
    iface: IDirectInput8W,
}

pub trait IntoModuleInstance {
    fn into_instance(self) -> HINSTANCE;
}

impl IntoModuleInstance for *mut c_void {
    fn into_instance(self) -> HINSTANCE {
        HINSTANCE(self as isize)
    }
}

impl IntoModuleInstance for HINSTANCE {
    fn into_instance(self) -> HINSTANCE {
        self
    }
}

impl DirectInputManager {
    pub fn new(instance: impl IntoModuleInstance) -> Result<Self> {
        let mut iface: Option<IDirectInput8W> = None;

        unsafe {
            DirectInput8Create(
                instance.into_instance(),
                DIRECTINPUT_VERSION,
                &IDirectInput8W::IID,
                &mut iface as *mut _ as _,
                None,
            )?;
        };

        match iface {
            Some(iface) => Ok(Self { iface }),
            None => Err(DirectInputError::BadDriverVersion),
        }
    }

    pub fn enum_devices(&self) -> Result<Vec<DirectInputDeviceInfo>> {
        extern "system" fn enumeration_callback(
            device_instance: *mut DIDEVICEINSTANCEW,
            ctx: *mut c_void,
        ) -> BOOL {
            let devices = unsafe { &mut *(ctx as *mut Vec<DirectInputDeviceInfo>) };

            if !device_instance.is_null() {
                devices.push(DirectInputDeviceInfo::from_instance(unsafe {
                    &*device_instance
                }));
            }

            BOOL(DIENUM_CONTINUE as _)
        }

        let mut devices = Vec::new();

        unsafe {
            self.iface.EnumDevices(
                DI8DEVCLASS_GAMECTRL,
                Some(enumeration_callback),
                &mut devices as *mut Vec<DirectInputDeviceInfo> as _,
                DIEDFL_ALLDEVICES,
            )?;
        };

        Ok(devices)
    }

    pub fn create_device(
        &self,
        device_info: &DirectInputDeviceInfo,
    ) -> Result<Device, DirectInputError> {
        let mut iface: Option<IDirectInputDevice8W> = None;

        unsafe {
            self.iface
                .CreateDevice(device_info.guid_instance(), &mut iface, None)?;
        };

        match iface {
            Some(device) => Ok(Device::new(device)),
            None => Err(DirectInputError::InputLost),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::bindings::Windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use crate::joy_state::JoyState;

    #[test]
    fn test_create_instance() {
        let dll_instance = unsafe { GetModuleHandleW(None) };

        DirectInputManager::new(dll_instance).expect("Failed to initialize manager");
    }

    #[test]
    fn test_enumeration() {
        let dll_instance = unsafe { GetModuleHandleW(None) };
        let manager = DirectInputManager::new(dll_instance).expect("Failed to initialize manager");

        manager.enum_devices().expect("Failed to enumerate devices");
    }

    #[test]
    fn test_create() {
        let dll_instance = unsafe { GetModuleHandleW(None) };
        let manager = DirectInputManager::new(dll_instance).expect("Failed to initialize manager");
        let devices = manager.enum_devices().expect("Failed to enumerate devices");

        if let Some(device) = devices.first() {
            let mut device = manager
                .create_device(device)
                .expect("Failed to create device instance");

            device.init_event().expect("Failed to initialize event");
            device.init().expect("Failed to initialize device");
            device
                .wait(Duration::from_secs(5))
                .expect("Failed to wait for event");

            let state = device
                .get_state::<JoyState>()
                .expect("Failed to get device state");
            eprintln!("state: {:#?}", state);
        }
    }
}
