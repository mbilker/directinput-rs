use std::ptr;
use std::time::Duration;

use directinput::{DirectInputManager, JoyState};
use winapi::um::libloaderapi;

fn main() {
    let dll_instance = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };
    let manager = DirectInputManager::new(dll_instance).expect("Failed to initialize manager");
    let devices = manager.enum_devices().expect("Failed to enumerate devices");

    let device_info = devices.first().expect("No device found");

    let device = manager
        .create_device(device_info)
        .expect("Failed to create device instance");

    let caps = device
        .capabilities()
        .expect("Failed to get device capabilities");
    println!("{:#?}", caps);

    device
        .set_axes_range(i16::min_value() as i32, i16::max_value() as i32)
        .expect("Failed to set axes range");

    //device.init_event().expect("Failed to initialize event");
    device.init().expect("Failed to initialize device");
    device.acquire().expect("Failed to acquire device access");

    let mut previous_state: Option<JoyState> = None;

    loop {
        /*
        device
            .wait(Duration::from_secs(5))
            .expect("Failed to wait for event");
         */

        let state = device
            .get_state::<JoyState>()
            .expect("Failed to get device state");
        let last_state = previous_state.as_ref().unwrap_or(&state);

        // Detect negative-to-positive range rollover
        let change = state.x as i32 - last_state.x as i32;
        let rollover_detect = if change.abs() > i16::max_value() as i32 {
            ", roll-over detected!"
        } else {
            ""
        };

        println!(
            "Axes: X: {} (change: {}), Y: {}, Z: {}{}",
            state.x, change, state.y, state.z, rollover_detect
        );

        previous_state = Some(state);

        std::thread::sleep(Duration::from_millis(25));
    }
}
