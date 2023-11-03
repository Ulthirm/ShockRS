use rosc::OscType;
use serde::Deserialize;
use std::fs;
use std::fmt;
use once_cell::sync::Lazy;

use crate::{openshock_legacy,openshock,pishock};

#[derive(Deserialize)]
pub struct Touchpoints {
    pub touchpoints: Vec<Device>,
}

#[derive(Deserialize)]
pub struct Device {
    pub address: String,
    pub firmware: String,
    pub method: u8,
    pub intensity: f32,
    pub duration: u32,
    pub ids: Vec<u32>,
}

pub static TOUCHPOINTS: Lazy<Touchpoints> = Lazy::new(|| {
    let touchpoints_path = "touchpoints.toml";
    let touchpoints_str = fs::read_to_string(touchpoints_path)
        .expect("Failed to read touchpoints.toml");
    toml::from_str(&touchpoints_str)
        .expect("Failed to parse touchpoints")
});

pub fn get_config() -> &'static Touchpoints {
    &TOUCHPOINTS
}

pub async fn display_touchpoints() -> Result<(), Box<dyn std::error::Error>> {
    for device in TOUCHPOINTS.touchpoints.iter() {
        log::debug!("\nTouchpoint: {}\n Firmware: {}\n IDs: {}", device.address, device.firmware, device.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

//takes in a given touchpoint and routes it to the appropriate function for it's firmware
pub async fn touchpoint_router(touchpoint: String, touchpoint_args: Vec<OscType>) {
    log::debug!("Touchpoint Router: {}", touchpoint);

    if let Some(device) = TOUCHPOINTS.touchpoints.iter().find(|device| device.address == touchpoint) {
        log::debug!("Touchpoint Firmware: {}", device.firmware);
        match device.firmware.to_ascii_lowercase().as_str() {
            "legacy" => {
                log::debug!("Legacy Touchpoint");
                openshock_legacy::handler::handler(device,touchpoint,touchpoint_args.clone()).await;
            },
            "openshock" => {
                log::debug!("OpenShock Touchpoint");
                openshock::handler::handler(touchpoint,touchpoint_args.clone()).await;
            },
            "pishock" => {
                log::debug!("PiShock Touchpoint");
                pishock::handler::handler(touchpoint,touchpoint_args.clone()).await;
            }
            _ => log::error!("Unknown touchpoint firmware: {}", device.firmware),
        }
    } else {
        log::error!("Unknown touchpoint: {}", touchpoint);
    }

    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }
}
