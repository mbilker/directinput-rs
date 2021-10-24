use std::ptr;
use std::thread;
use std::time::Duration;

use directinput::{CooperativeLevel, Device, DirectInputManager, JoyState};
use winapi::um::libloaderapi;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let dll_instance = unsafe { libloaderapi::GetModuleHandleW(ptr::null_mut()) };
    let manager = DirectInputManager::new(dll_instance).expect("Failed to initialize manager");
    let devices = manager.enum_devices().expect("Failed to enumerate devices");

    let device_info = devices.first().expect("No device found");

    let mut device = manager
        .create_device(device_info)
        .expect("Failed to create device instance");

    let caps = device
        .capabilities()
        .expect("Failed to get device capabilities");
    println!("{:#?}", caps);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("directinput-rs")
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to create window");

    device
        .set_axes_range(i16::MIN as i32, i16::MAX as i32)
        .expect("Failed to set axes range");

    device.init_event().expect("Failed to initialize event");
    device.init().expect("Failed to initialize device");
    device
        .set_cooperative_level(
            &window,
            CooperativeLevel::BACKGROUND | CooperativeLevel::EXCLUSIVE,
        )
        .expect("Failed to set cooperative level");
    device.acquire().expect("Failed to acquire device access");

    thread::Builder::new()
        .name(String::from("directinput-rs input processing"))
        .spawn(move || input_thread(device))
        .expect("Failed to spawn window handler thread");

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
    });
}

fn input_thread(device: Device) {
    let mut previous_state: Option<JoyState> = None;

    loop {
        println!(
            "poll result: {:?}",
            device.poll().expect("Failed to poll device")
        );

        device
            .wait(Duration::from_secs(5))
            .expect("Failed to wait for event");

        let state = device
            .get_state::<JoyState>()
            .expect("Failed to get device state");
        let last_state = previous_state.as_ref().unwrap_or(&state);

        // Detect negative-to-positive range rollover
        let change = state.x as i32 - last_state.x as i32;
        let rollover_detect = if change.abs() > i16::MAX as i32 {
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
