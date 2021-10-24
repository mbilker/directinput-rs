mod bindings;
mod cooperative_level;
mod device;
mod device_capabilities;
mod device_info;
mod error;
mod joy_state;
mod manager;

pub use crate::cooperative_level::CooperativeLevel;
pub use crate::device::Device;
pub use crate::device_capabilities::DeviceCapabilities;
pub use crate::device_info::DirectInputDeviceInfo;
pub use crate::joy_state::JoyState;
pub use crate::manager::DirectInputManager;

#[doc(hidden)]
#[inline]
pub fn current_module() -> crate::bindings::Windows::Win32::Foundation::HINSTANCE {
    unsafe { crate::bindings::Windows::Win32::System::LibraryLoader::GetModuleHandleW(None) }
}
