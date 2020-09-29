use std::convert::TryInto;
use std::ffi::OsString;
use std::io;
use std::mem::{self, MaybeUninit};
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::time::Duration;

use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPVOID};
use winapi::shared::ntdef::HANDLE;
use winapi::shared::winerror::{self, HRESULT, S_OK};
use winapi::um::dinput::{
    c_dfDIJoystick2, IDirectInputDevice8W, DIDATAFORMAT, DIDEVCAPS, DIDFT_AXIS, DIENUM_CONTINUE,
    DIPH_BYID, DIPROPHEADER, DIPROPRANGE, DIPROP_RANGE, LPCDIDEVICEOBJECTINSTANCEW,
};
use winapi::um::errhandlingapi;
use winapi::um::handleapi::{self, INVALID_HANDLE_VALUE};
use winapi::um::synchapi;
use winapi::um::winbase::{INFINITE, WAIT_OBJECT_0};

use super::device_capabilities::DeviceCapabilities;

pub struct Device {
    iface: *mut IDirectInputDevice8W,
    #[allow(dead_code)]
    event: Option<HANDLE>,
}

pub trait FromDeviceState {
    type RawState;

    fn from_instance(state: Self::RawState) -> Self;
}

impl Device {
    pub(crate) fn from_instance(iface: *mut IDirectInputDevice8W) -> Self {
        Self { iface, event: None }
    }

    unsafe fn iface(&self) -> &IDirectInputDevice8W {
        &*self.iface
    }

    pub fn capabilities(&self) -> io::Result<DeviceCapabilities> {
        let mut caps: DIDEVCAPS = unsafe { MaybeUninit::zeroed().assume_init() };
        caps.dwSize = mem::size_of::<DIDEVCAPS>() as DWORD;

        let hr = unsafe { self.iface().GetCapabilities(&mut caps) };

        if winerror::SUCCEEDED(hr) {
            Ok(DeviceCapabilities::from_instance(caps))
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    pub fn init(&self) -> io::Result<()> {
        // Clone the `c_dfDIJoystick2` data format with the axis value set to relative
        let data_format: DIDATAFORMAT = unsafe { c_dfDIJoystick2 };
        eprintln!("dwSize: {}", data_format.dwSize);
        eprintln!("dwObjSize: {}", data_format.dwObjSize);
        eprintln!("dwFlags: {}", data_format.dwFlags);
        eprintln!("dwDataSize: {}", data_format.dwDataSize);
        eprintln!("dwNumObjs: {}", data_format.dwNumObjs);

        //data_format.dwFlags = DIDF_RELAXIS;

        self.set_data_format(&data_format)?;
        self.acquire()?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn init_event(&mut self) -> io::Result<()> {
        let hr = self
            .event
            .take()
            .map(|event| unsafe {
                let hr = self.iface().SetEventNotification(ptr::null_mut());

                handleapi::CloseHandle(event);

                hr
            })
            .unwrap_or(S_OK);

        if !winerror::SUCCEEDED(hr) {
            return Err(io::Error::from_raw_os_error(hr));
        }

        let event =
            unsafe { synchapi::CreateEventW(ptr::null_mut(), FALSE, FALSE, ptr::null_mut()) };
        if event == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        let hr = unsafe { self.iface().SetEventNotification(event) };

        if winerror::SUCCEEDED(hr) {
            self.event = Some(event);

            Ok(())
        } else {
            unsafe { handleapi::CloseHandle(event) };

            Err(io::Error::from_raw_os_error(hr))
        }
    }

    pub fn set_axes_range(&self, min: i32, max: i32) -> io::Result<()> {
        struct SetAxesRangeContext<'parent> {
            device: &'parent Device,
            min: i32,
            max: i32,
        }

        extern "system" fn enumerate_callback(
            device_object_instance: LPCDIDEVICEOBJECTINSTANCEW,
            ctx: LPVOID,
        ) -> BOOL {
            let ctx = unsafe { &*(ctx as *const SetAxesRangeContext) };

            if !device_object_instance.is_null() {
                let device_object_instance = unsafe { &*device_object_instance };

                let end = device_object_instance
                    .tszName
                    .iter()
                    .position(|&ch| ch == 0)
                    .unwrap_or(device_object_instance.tszName.len());
                let name = OsString::from_wide(&device_object_instance.tszName[..end]);
                println!("name: {:?}", name);

                let prop_range = DIPROPRANGE {
                    diph: DIPROPHEADER {
                        dwSize: mem::size_of::<DIPROPRANGE>() as DWORD,
                        dwHeaderSize: mem::size_of::<DIPROPHEADER>() as DWORD,
                        dwHow: DIPH_BYID,
                        dwObj: device_object_instance.dwType,
                    },
                    lMin: ctx.min,
                    lMax: ctx.max,
                };
                let hr = unsafe {
                    ctx.device
                        .iface()
                        .SetProperty(DIPROP_RANGE, &prop_range as *const _ as _)
                };

                if !winerror::SUCCEEDED(hr) {
                    eprintln!(
                        "Failed to set device range: {:?}",
                        io::Error::from_raw_os_error(hr)
                    );
                }
            }

            DIENUM_CONTINUE
        }

        let ctx = SetAxesRangeContext {
            device: self,
            min,
            max,
        };
        let hr = unsafe {
            self.iface().EnumObjects(
                Some(enumerate_callback),
                &ctx as *const SetAxesRangeContext as *mut _,
                DIDFT_AXIS,
            )
        };

        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    pub fn acquire(&self) -> io::Result<()> {
        let hr = unsafe { (*self.iface).Acquire() };

        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    pub fn set_data_format(&self, format: &DIDATAFORMAT) -> io::Result<()> {
        let hr = unsafe { (*self.iface).SetDataFormat(format) };

        if winerror::SUCCEEDED(hr) {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    #[allow(dead_code)]
    pub fn poll(&self) -> io::Result<HRESULT> {
        let hr = unsafe { (*self.iface).Poll() };

        if winerror::SUCCEEDED(hr) {
            Ok(hr)
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    pub fn get_state<T: FromDeviceState>(&self) -> io::Result<T> {
        let mut data: MaybeUninit<T::RawState> = MaybeUninit::uninit();
        let hr = unsafe {
            (*self.iface).GetDeviceState(
                mem::size_of::<T::RawState>() as DWORD,
                data.as_mut_ptr() as *mut _,
            )
        };

        if winerror::SUCCEEDED(hr) {
            let state = unsafe { data.assume_init() };

            Ok(T::from_instance(state))
        } else {
            Err(io::Error::from_raw_os_error(hr))
        }
    }

    #[allow(dead_code)]
    pub fn wait(&self, timeout: Duration) -> io::Result<()> {
        if let Some(event) = self.event {
            let millis: DWORD = timeout.as_millis().try_into().unwrap_or(INFINITE);

            let res = unsafe { synchapi::WaitForSingleObject(event, millis) };
            if res != WAIT_OBJECT_0 {
                let err = unsafe { errhandlingapi::GetLastError() };

                if err == 0 {
                    return Ok(());
                } else {
                    return Err(io::Error::last_os_error());
                }
            }
        }

        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if !self.iface.is_null() {
            unsafe {
                (*self.iface).Unacquire();
                (*self.iface).Release();
                self.iface = ptr::null_mut();
            }
        }
    }
}
