use bitflags::bitflags;
use windows::Win32::Devices::HumanInterfaceDevice::{
    DISCL_BACKGROUND, DISCL_EXCLUSIVE, DISCL_FOREGROUND, DISCL_NONEXCLUSIVE, DISCL_NOWINKEY,
};

bitflags! {
    pub struct CooperativeLevel: u32 {
        /// The application requires background access. If background access is granted, the device
        /// can be acquired at any time, even when the associated window is not the active window.
        const BACKGROUND = DISCL_BACKGROUND;

        /// The application requires exclusive access. If exclusive access is granted, no other
        /// instance of the device can obtain exclusive access to the device while it is acquired.
        /// However, nonexclusive access to the device is always permitted, even if another
        /// application has obtained exclusive access. An application that acquires the mouse or
        /// keyboard device in exclusive mode should always unacquire the devices when it receives
        /// `WM_ENTERSIZEMOVE` and `WM_ENTERMENULOOP` messages. Otherwise, the user cannot
        /// manipulate the menu or move and resize the window.
        const EXCLUSIVE = DISCL_EXCLUSIVE;

        /// The application requires foreground access. If foreground access is granted, the device
        /// is automatically unacquired when the associated window moves to the background.
        const FOREGROUND = DISCL_FOREGROUND;

        /// The application requires nonexclusive access. Access to the device does not interfere
        /// with other applications that are accessing the same device.
        const NON_EXCLUSIVE = DISCL_NONEXCLUSIVE;

        /// Disable the Windows logo key. Setting this flag ensures that the user cannot
        /// inadvertently break out of the application. Note, however, that `DISCL_NOWINKEY` has no
        /// effect when the default action mapping user interface (UI) is displayed, and the Windows
        /// logo key will operate normally as long as that UI is present.
        const NO_WIN_KEY = DISCL_NOWINKEY;
    }
}
