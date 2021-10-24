fn main() {
    // `dxguid` is needed for many of the included GUID definitions. MSVC
    // reports undefined external symbols while linking without this.
    println!("cargo:rustc-link-lib=dylib=dxguid");

    windows::build! {
        Windows::Win32::Devices::HumanInterfaceDevice::{
            DirectInput8Create, IDirectInput8W, IDirectInputDevice8W, DI8DEVCLASS_GAMECTRL,
            DIDATAFORMAT, DIDEVCAPS, DIDEVICEINSTANCEW, DIDEVICEOBJECTINSTANCEW, DIDFT_AXIS,
            DIEDFL_ALLDEVICES, DIENUM_CONTINUE, DIJOYSTATE2, DIPROPHEADER, DIPROPRANGE,
            DIRECTINPUT_VERSION, DISCL_BACKGROUND, DISCL_EXCLUSIVE, DISCL_FOREGROUND,
            DISCL_NONEXCLUSIVE, DISCL_NOWINKEY, DI_DOWNLOADSKIPPED, DI_EFFECTRESTARTED,
            DI_POLLEDDEVICE, DI_SETTINGSNOTSAVED, DI_TRUNCATED, DI_TRUNCATEDANDRESTARTED,
            DI_WRITEPROTECT,
        },
        Windows::Win32::Foundation::{
            CloseHandle, BOOL, E_FAIL, HANDLE, HINSTANCE, INVALID_HANDLE_VALUE, S_OK,
        },
        Windows::Win32::System::Com::E_PENDING,
        Windows::Win32::System::Diagnostics::Debug::GetLastError,
        Windows::Win32::System::LibraryLoader::GetModuleHandleW,
        Windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject},
        Windows::Win32::System::WindowsProgramming::INFINITE,
    }
}
