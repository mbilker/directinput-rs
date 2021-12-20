use std::ffi::OsString;
use std::fmt::{self, Write};
use std::os::windows::ffi::OsStringExt;

use windows::core::GUID;
use windows::Win32::Devices::HumanInterfaceDevice::DIDEVICEINSTANCEW;

pub struct DirectInputDeviceInfo {
    guid_instance: GUID,
    guid_product: GUID,
    instance_name: OsString,
    product_name: OsString,
    force_feedback_driver: GUID,
    usage_page: u16,
    usage: u16,
}

impl DirectInputDeviceInfo {
    pub(crate) fn from_instance(device_instance: &DIDEVICEINSTANCEW) -> Self {
        let instance_name = {
            let end = device_instance
                .tszProductName
                .iter()
                .position(|&ch| ch == 0)
                .unwrap_or(device_instance.tszInstanceName.len());
            OsStringExt::from_wide(&device_instance.tszInstanceName[..end])
        };
        let product_name = {
            let end = device_instance
                .tszProductName
                .iter()
                .position(|&ch| ch == 0)
                .unwrap_or(device_instance.tszProductName.len());
            OsStringExt::from_wide(&device_instance.tszProductName[..end])
        };

        Self {
            guid_instance: device_instance.guidInstance,
            guid_product: device_instance.guidProduct,
            instance_name,
            product_name,
            force_feedback_driver: device_instance.guidFFDriver,
            usage_page: device_instance.wUsagePage,
            usage: device_instance.wUsage,
        }
    }

    pub(crate) fn guid_instance(&self) -> &GUID {
        &self.guid_instance
    }

    pub fn guid_instance_str(&self) -> String {
        GuidString(&self.guid_instance).to_string()
    }
}

struct GuidString<'a>(&'a GUID);

impl<'a> fmt::Display for GuidString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0.data1,
            self.0.data2,
            self.0.data3,
            self.0.data4[0],
            self.0.data4[1],
            self.0.data4[2],
            self.0.data4[3],
            self.0.data4[4],
            self.0.data4[5],
            self.0.data4[6],
            self.0.data4[7]
        )
    }
}

impl<'a> fmt::Debug for GuidString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('"')?;
        fmt::Display::fmt(self, f)?;
        f.write_char('"')
    }
}

impl fmt::Debug for DirectInputDeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DirectInputDevice")
            .field("guid_instance", &GuidString(&self.guid_instance))
            .field("guid_product", &GuidString(&self.guid_product))
            .field("instance_name", &self.instance_name)
            .field("product_name", &self.product_name)
            .field(
                "force_feedback_driver",
                &GuidString(&self.force_feedback_driver),
            )
            .field("usage_page", &self.usage_page)
            .field("usage", &self.usage)
            .finish()
    }
}
