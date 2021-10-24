use super::device::FromDeviceState;
use crate::bindings::Windows::Win32::Devices::HumanInterfaceDevice::DIJOYSTATE2;

#[derive(Debug)]
pub struct JoyState {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub slider: [i32; 2],
    pub pov: [u32; 4],
    pub buttons: [u8; 128],
    pub v_x: i32,
    pub v_y: i32,
    pub v_z: i32,
    pub v_rx: i32,
    pub v_ry: i32,
    pub v_rz: i32,
    pub v_slider: [i32; 2],
    pub a_x: i32,
    pub a_y: i32,
    pub a_z: i32,
    pub a_rx: i32,
    pub a_ry: i32,
    pub a_rz: i32,
    pub a_slider: [i32; 2],
    pub f_x: i32,
    pub f_y: i32,
    pub f_z: i32,
    pub f_rx: i32,
    pub f_ry: i32,
    pub f_rz: i32,
    pub f_slider: [i32; 2],
}

impl FromDeviceState for JoyState {
    type RawState = DIJOYSTATE2;

    fn from_instance(state: Self::RawState) -> Self {
        Self {
            x: state.lX,
            y: state.lY,
            z: state.lZ,
            rx: state.lRx,
            ry: state.lRy,
            rz: state.lRz,
            slider: state.rglSlider,
            pov: state.rgdwPOV,
            buttons: state.rgbButtons,
            v_x: state.lVX,
            v_y: state.lVY,
            v_z: state.lVZ,
            v_rx: state.lVRx,
            v_ry: state.lVRy,
            v_rz: state.lVRz,
            v_slider: state.rglVSlider,
            a_x: state.lAX,
            a_y: state.lAY,
            a_z: state.lAZ,
            a_rx: state.lARx,
            a_ry: state.lARy,
            a_rz: state.lARz,
            a_slider: state.rglASlider,
            f_x: state.lFX,
            f_y: state.lFY,
            f_z: state.lFZ,
            f_rx: state.lFRx,
            f_ry: state.lFRy,
            f_rz: state.lFRz,
            f_slider: state.rglFSlider,
        }
    }
}
