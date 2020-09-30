use std::io;
use std::ptr;

use winapi::shared::minwindef::{BOOL, HINSTANCE, LPVOID};
use winapi::shared::winerror;
use winapi::um::dinput::{
    self, IDirectInput8W, IDirectInputDevice8W, IID_IDirectInput8W, DI8DEVCLASS_GAMECTRL,
    DIEDFL_ALLDEVICES, DIENUM_CONTINUE, DIRECTINPUT_VERSION, LPCDIDEVICEINSTANCEW,
};

use crate::device::Device;
use crate::device_info::DirectInputDeviceInfo;
use crate::error::DirectInputError;

#[derive(Debug)]
pub struct DirectInputManager {
    iface: *mut IDirectInput8W,
}

impl DirectInputManager {
    pub fn new(dll_instance: HINSTANCE) -> io::Result<Self> {
        let mut iface: *mut IDirectInput8W = ptr::null_mut();
        let hr = unsafe {
            dinput::DirectInput8Create(
                dll_instance,
                DIRECTINPUT_VERSION,
                &IID_IDirectInput8W,
                &mut iface as *mut *mut IDirectInput8W as *mut _,
                ptr::null_mut(),
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Self { iface })
        } else {
            Err(DirectInputError::from_hresult(hr))
        }
    }

    unsafe fn iface(&self) -> &IDirectInput8W {
        &*self.iface
    }

    pub fn enum_devices(&self) -> io::Result<Vec<DirectInputDeviceInfo>> {
        extern "system" fn enumeration_callback(
            device_instance: LPCDIDEVICEINSTANCEW,
            ctx: LPVOID,
        ) -> BOOL {
            let devices = unsafe { &mut *(ctx as *mut Vec<DirectInputDeviceInfo>) };

            if !device_instance.is_null() {
                devices.push(DirectInputDeviceInfo::from_instance(unsafe {
                    &*device_instance
                }));
            }

            DIENUM_CONTINUE
        }

        let mut devices = Vec::new();

        let hr = unsafe {
            self.iface().EnumDevices(
                DI8DEVCLASS_GAMECTRL,
                Some(enumeration_callback),
                &mut devices as *mut Vec<DirectInputDeviceInfo> as _,
                DIEDFL_ALLDEVICES,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(devices)
        } else {
            Err(DirectInputError::from_hresult(hr))
        }
    }

    pub fn create_device(&self, device_info: &DirectInputDeviceInfo) -> io::Result<Device> {
        let mut iface: *mut IDirectInputDevice8W = ptr::null_mut();
        let hr = unsafe {
            self.iface().CreateDevice(
                device_info.guid_instance(),
                &mut iface as *mut _ as _,
                ptr::null_mut(),
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(Device::from_instance(iface))
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }
}

impl Drop for DirectInputManager {
    fn drop(&mut self) {
        if !self.iface.is_null() {
            unsafe {
                (*self.iface).Release();
                self.iface = ptr::null_mut();
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use winapi::um::libloaderapi;

    use super::*;
    use crate::joy_state::JoyState;

    #[test]
    fn test_create_instance() {
        let dll_instance = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };

        DirectInputManager::new(dll_instance).expect("Failed to initialize manager");
    }

    #[test]
    fn test_enumeration() {
        let dll_instance = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };
        let manager = DirectInputManager::new(dll_instance).expect("Failed to initialize manager");

        manager.enum_devices().expect("Failed to enumerate devices");
    }

    #[test]
    fn test_create() {
        let dll_instance = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };
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
