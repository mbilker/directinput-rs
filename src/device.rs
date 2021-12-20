use std::convert::TryInto;
//use std::ffi::OsString;
use std::ffi::c_void;
use std::mem::{self, MaybeUninit};
//use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::time::Duration;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winapi::um::dinput::{
    c_dfDIJoystick2, DIERR_OTHERAPPHASPRIO, DIPH_BYID, DIPROP_RANGE, DI_NOEFFECT, DI_OK,
    DI_POLLEDDEVICE,
};
use windows::{Interface, HRESULT};

use crate::bindings::Windows::Win32::Devices::HumanInterfaceDevice::{
    IDirectInputDevice8W, DIDATAFORMAT, DIDEVCAPS, DIDEVICEOBJECTINSTANCEW, DIDFT_AXIS,
    DIENUM_CONTINUE, DIPROPHEADER, DIPROPRANGE,
};
use crate::bindings::Windows::Win32::Foundation::{
    CloseHandle, BOOL, HANDLE, HWND, INVALID_HANDLE_VALUE,
};
use crate::bindings::Windows::Win32::System::Diagnostics::Debug::GetLastError;
use crate::bindings::Windows::Win32::System::Threading::{
    CreateEventW, WaitForSingleObject, WAIT_OBJECT_0,
};
use crate::bindings::Windows::Win32::System::WindowsProgramming::INFINITE;
use crate::cooperative_level::CooperativeLevel;
use crate::device_capabilities::DeviceCapabilities;
use crate::error::{DirectInputError, DirectInputStatus, Result};

pub struct Device {
    iface: IDirectInputDevice8W,
    event: Option<HANDLE>,
}

pub trait FromDeviceState {
    type RawState;

    fn from_instance(state: Self::RawState) -> Self;
}

unsafe impl Send for Device {}

impl Device {
    pub(crate) fn new(iface: IDirectInputDevice8W) -> Self {
        Self { iface, event: None }
    }

    pub fn capabilities(&self) -> Result<DeviceCapabilities> {
        let mut caps = DIDEVCAPS::default();
        caps.dwSize = mem::size_of::<DIDEVCAPS>() as _;

        unsafe { self.iface.GetCapabilities(&mut caps)? };

        Ok(DeviceCapabilities::from_instance(caps))
    }

    pub fn init(&mut self) -> Result<()> {
        // Clone the `c_dfDIJoystick2` data format with the axis value set to relative
        let mut data_format: DIDATAFORMAT = unsafe { mem::transmute(c_dfDIJoystick2) };
        eprintln!("dwSize: {}", data_format.dwSize);
        eprintln!("dwObjSize: {}", data_format.dwObjSize);
        eprintln!("dwFlags: {}", data_format.dwFlags);
        eprintln!("dwDataSize: {}", data_format.dwDataSize);
        eprintln!("dwNumObjs: {}", data_format.dwNumObjs);

        //data_format.dwFlags = DIDF_RELAXIS;

        self.set_data_format(&mut data_format)?;

        Ok(())
    }

    pub fn init_event(&mut self) -> Result<DirectInputStatus> {
        self.event
            .take()
            .map(|event| unsafe {
                let res = self.iface.SetEventNotification(None);

                CloseHandle(event);

                res
            })
            .unwrap_or(Ok(()))?;

        let event = unsafe { CreateEventW(ptr::null(), false, false, None) };

        if event == INVALID_HANDLE_VALUE {
            return Err(DirectInputError::from_last_error());
        }

        // TODO: Use `SetEventNotification` directly when status `HRESULT` return values are
        // exposed
        let hr = unsafe { (self.iface.vtable().12)(mem::transmute_copy(&self.iface), event) };

        if let Err(e) = hr.ok() {
            unsafe { CloseHandle(event) };

            return Err(e.into());
        }

        self.event = Some(event);

        Ok(match hr.0 as i32 {
            // If the method succeeds, the return value is DI_OK
            DI_OK => DirectInputStatus::Ok,
            // or DI_POLLEDDEVICE
            DI_POLLEDDEVICE => DirectInputStatus::PolledDevice,
            _ => DirectInputStatus::from_hresult_or_ok(hr),
        })
    }

    pub fn set_axes_range(&mut self, min: i32, max: i32) -> Result<()> {
        struct SetAxesRangeContext<'parent> {
            device: &'parent Device,
            min: i32,
            max: i32,
        }

        extern "system" fn enumerate_callback(
            device_object_instance: *mut DIDEVICEOBJECTINSTANCEW,
            ctx: *mut c_void,
        ) -> BOOL {
            let ctx = unsafe { &*(ctx as *const SetAxesRangeContext) };

            if !device_object_instance.is_null() {
                let device_object_instance = unsafe { &*device_object_instance };

                /*
                let end = device_object_instance
                    .tszName
                    .iter()
                    .position(|&ch| ch == 0)
                    .unwrap_or(device_object_instance.tszName.len());
                let name = OsString::from_wide(&device_object_instance.tszName[..end]);
                println!("name: {:?}", name);
                */

                let prop_range = DIPROPRANGE {
                    diph: DIPROPHEADER {
                        dwSize: mem::size_of::<DIPROPRANGE>() as _,
                        dwHeaderSize: mem::size_of::<DIPROPHEADER>() as _,
                        dwHow: DIPH_BYID,
                        dwObj: device_object_instance.dwType,
                    },
                    lMin: ctx.min,
                    lMax: ctx.max,
                };
                unsafe {
                    if let Err(e) = ctx
                        .device
                        .iface
                        .SetProperty(DIPROP_RANGE.cast(), &prop_range as *const _ as _)
                    {
                        eprintln!("Failed to set device range: {:?}", e);
                    }
                };
            }

            BOOL(DIENUM_CONTINUE as _)
        }

        let ctx = SetAxesRangeContext {
            device: self,
            min,
            max,
        };
        unsafe {
            self.iface.EnumObjects(
                Some(enumerate_callback),
                &ctx as *const SetAxesRangeContext as *mut _,
                DIDFT_AXIS,
            )?
        };

        Ok(())
    }

    pub fn acquire(&self) -> Result<()> {
        match unsafe { self.iface.Acquire() } {
            Ok(()) => Ok(()),
            Err(e) => match e.code().0 as i32 {
                DIERR_OTHERAPPHASPRIO => Err(DirectInputError::OtherAppHasPrio),
                _ => Err(e.into()),
            },
        }
    }

    pub fn set_cooperative_level<H: HasRawWindowHandle>(
        &mut self,
        window_handle: &H,
        flags: CooperativeLevel,
    ) -> Result<()> {
        let hwnd = match window_handle.raw_window_handle() {
            RawWindowHandle::Win32(handle) => handle.hwnd,
            _ => return Err(DirectInputError::Handle),
        };

        Ok(unsafe {
            self.iface
                .SetCooperativeLevel(HWND(hwnd as _), flags.bits())?
        })
    }

    pub fn set_data_format(&mut self, format: &mut DIDATAFORMAT) -> Result<()> {
        Ok(unsafe { self.iface.SetDataFormat(format)? })
    }

    /// From MSDN:
    /// > Retrieves data from polled objects on a DirectInput device. If the device does not require
    /// > polling, calling this method has no effect. If a device that requires polling is not
    /// > polled periodically, no new data is received from the device. Calling this method causes
    /// > DirectInput to update the device state, generate input events (if buffered data is
    /// > enabled), and set notification events (if notification is enabled).
    /// >
    /// > If the method succeeds, the return value is DI_OK, or DI_NOEFFECT if the device does not
    /// > require polling. If the method fails, the return value can be one of the following error
    /// > values: DIERR_INPUTLOST, DIERR_NOTACQUIRED, DIERR_NOTINITIALIZED.
    pub fn poll(&self) -> Result<DirectInputStatus> {
        // TODO(felix): replace with direct `Poll` call when the non-error part of the `HRESULT` is
        // made available to API consumers
        let hr = unsafe { (self.iface.vtable().25)(mem::transmute_copy(&self.iface)) };

        if hr.is_ok() {
            Ok(match hr.0 as i32 {
                // If the method succeeds, the return value is DI_OK
                DI_OK => DirectInputStatus::Ok,
                // or `DI_NOEFFECT` if the device does not require polling.
                DI_NOEFFECT => DirectInputStatus::NoEffect,
                // Check for other status codes or fallback to `Ok`
                _ => DirectInputStatus::from_hresult_or_ok(hr),
            })
        } else {
            Err(DirectInputError::from_hresult(hr))
        }
    }

    pub fn get_state<T: FromDeviceState>(&self) -> Result<T> {
        let mut data: MaybeUninit<T::RawState> = MaybeUninit::zeroed();

        unsafe {
            self.iface
                .GetDeviceState(mem::size_of::<T::RawState>() as _, data.as_mut_ptr().cast())?;

            let state = data.assume_init();

            Ok(T::from_instance(state))
        }
    }

    /// If event polling is enabled using `init_event`, this will wait for up to the duration
    /// specified for an event update to arrive.
    ///
    /// Return value is `true` if an event arrived or `false` if the timeout expired. If no event
    /// handle is configured, this method returns `true`.
    pub fn wait(&self, timeout: Duration) -> Result<bool> {
        if let Some(event) = self.event {
            let millis: u32 = timeout.as_millis().try_into().unwrap_or(INFINITE);

            let res = unsafe { WaitForSingleObject(event, millis) };

            if res == WAIT_OBJECT_0 {
                Ok(true)
            } else {
                let err = unsafe { GetLastError() };

                if err.0 == 0 {
                    Ok(false)
                } else {
                    Err(DirectInputError::from_hresult(HRESULT::from(err)))
                }
            }
        } else {
            Ok(true)
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            let _ = self.iface.Unacquire();
        };
    }
}
