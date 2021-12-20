use std::error::Error as StdError;
use std::fmt;
use std::thread;
use std::time::Duration;

use directinput::{CooperativeLevel, Device, DirectInputError, DirectInputManager, JoyState};
use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[derive(Debug)]
struct Error {
    msg: &'static str,
    source: DirectInputError,
}

impl Error {
    fn new(source: DirectInputError, msg: &'static str) -> Self {
        Self { msg, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.msg)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

fn main() {
    let dll_instance = directinput::current_module();
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

    let event_loop = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();
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
        .spawn(move || {
            if let Err(e) = input_thread(device) {
                if let Err(e) = proxy.send_event(e) {
                    eprintln!("Failed to send error to message handler: {}", e);
                }
            }
        })
        .expect("Failed to spawn window handler thread");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(e) => {
                eprintln!("Error: {}", e);

                if let Some(e) = e.source() {
                    eprintln!();
                    eprintln!("Caused by:");
                    eprintln!("    {}", e);
                }

                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn input_thread(device: Device) -> Result<(), Error> {
    let mut previous_state: Option<JoyState> = None;

    loop {
        println!(
            "poll result: {:?}",
            device
                .poll()
                .map_err(|source| Error::new(source, "Failed to poll devicec"))
        );

        device
            .wait(Duration::from_secs(5))
            .map_err(|source| Error::new(source, "Failed to wait for event"))?;

        let state = device
            .get_state::<JoyState>()
            .map_err(|source| Error::new(source, "Failed to get device state"))?;
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
