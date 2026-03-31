use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, StreamConfig};
use wavecraft_protocol::{
    GetHardwareInputSelectionResult, HardwareInputChannelOption, HardwareInputDeviceOption,
};

const DEVICE_ID_PREFIX: &str = "input-device";
const MONO_CHANNEL_PREFIX: &str = "mono";
const STEREO_CHANNEL_PREFIX: &str = "stereo";
const NONE_CHANNEL_SENTINEL: usize = usize::MAX;
const ORDERING: Ordering = Ordering::SeqCst;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareInputRouting {
    pub left_channel: usize,
    pub right_channel: Option<usize>,
}

#[derive(Clone)]
pub struct SharedHardwareInputRoutingSelection {
    left_channel: Arc<AtomicUsize>,
    right_channel: Arc<AtomicUsize>,
}

impl Default for SharedHardwareInputRoutingSelection {
    fn default() -> Self {
        Self::new(HardwareInputRouting {
            left_channel: 0,
            right_channel: Some(1),
        })
    }
}

impl SharedHardwareInputRoutingSelection {
    pub fn new(initial: HardwareInputRouting) -> Self {
        Self {
            left_channel: Arc::new(AtomicUsize::new(initial.left_channel)),
            right_channel: Arc::new(AtomicUsize::new(
                initial.right_channel.unwrap_or(NONE_CHANNEL_SENTINEL),
            )),
        }
    }

    pub fn load(&self) -> HardwareInputRouting {
        let right_channel = self.right_channel.load(ORDERING);
        HardwareInputRouting {
            left_channel: self.left_channel.load(ORDERING),
            right_channel: (right_channel != NONE_CHANNEL_SENTINEL).then_some(right_channel),
        }
    }

    pub fn store(&self, routing: HardwareInputRouting) {
        self.left_channel.store(routing.left_channel, ORDERING);
        self.right_channel.store(
            routing.right_channel.unwrap_or(NONE_CHANNEL_SENTINEL),
            ORDERING,
        );
    }
}

pub struct ResolvedHardwareInputDevice {
    pub device: Device,
    pub config: StreamConfig,
    pub selected_device_id: String,
    pub channel_count: u16,
}

struct EnumeratedHardwareInputDevice {
    id: String,
    label: String,
    description: Option<String>,
    channel_count: u16,
    device: Device,
    config: StreamConfig,
}

pub fn build_hardware_input_selection(
    requested_device_id: Option<&str>,
    requested_channel_id: Option<&str>,
) -> Result<GetHardwareInputSelectionResult> {
    let devices = enumerate_input_devices()?;
    let available_devices = devices
        .iter()
        .map(|device| HardwareInputDeviceOption {
            id: device.id.clone(),
            label: device.label.clone(),
            channel_count: device.channel_count,
            description: device.description.clone(),
        })
        .collect::<Vec<_>>();

    let Some(selected_device) = resolve_selected_device(&devices, requested_device_id) else {
        return Ok(GetHardwareInputSelectionResult {
            selected_device_id: None,
            available_devices,
            selected_channel_id: None,
            available_channels: Vec::new(),
        });
    };

    let available_channels = build_channel_options(selected_device.channel_count);
    let selected_channel_id =
        resolve_selected_channel_id(requested_channel_id, &available_channels);

    Ok(GetHardwareInputSelectionResult {
        selected_device_id: Some(selected_device.id.clone()),
        available_devices,
        selected_channel_id,
        available_channels,
    })
}

pub fn resolve_selected_input_device(
    requested_device_id: Option<&str>,
) -> Result<ResolvedHardwareInputDevice> {
    let devices = enumerate_input_devices()?;
    let selected_device = resolve_selected_device(&devices, requested_device_id)
        .context("No input device available")?;

    Ok(ResolvedHardwareInputDevice {
        device: selected_device.device.clone(),
        config: selected_device.config.clone(),
        selected_device_id: selected_device.id.clone(),
        channel_count: selected_device.channel_count,
    })
}

pub fn resolve_selected_channel_id(
    requested_channel_id: Option<&str>,
    available_channels: &[HardwareInputChannelOption],
) -> Option<String> {
    requested_channel_id
        .filter(|requested| {
            available_channels
                .iter()
                .any(|channel| channel.id == *requested)
        })
        .map(ToOwned::to_owned)
        .or_else(|| available_channels.first().map(|channel| channel.id.clone()))
}

pub fn routing_from_channel_id(channel_id: &str) -> Option<HardwareInputRouting> {
    let mut parts = channel_id.split(':');
    let mode = parts.next()?;
    let first = parts.next()?.parse::<usize>().ok()?;

    match mode {
        MONO_CHANNEL_PREFIX => Some(HardwareInputRouting {
            left_channel: first,
            right_channel: None,
        }),
        STEREO_CHANNEL_PREFIX => Some(HardwareInputRouting {
            left_channel: first,
            right_channel: Some(parts.next()?.parse::<usize>().ok()?),
        }),
        _ => None,
    }
}

fn enumerate_input_devices() -> Result<Vec<EnumeratedHardwareInputDevice>> {
    let host = cpal::default_host();
    let default_device_name = host
        .default_input_device()
        .and_then(|device| device.name().ok());
    let input_devices = host
        .input_devices()
        .context("Failed to enumerate input devices")?;

    let mut devices = Vec::new();
    for (index, device) in input_devices.enumerate() {
        let Ok(supported_config) = device.default_input_config() else {
            continue;
        };
        let Ok(name) = device.name() else {
            continue;
        };

        let is_default = default_device_name
            .as_ref()
            .is_some_and(|default_name| default_name == &name);
        let channel_count = supported_config.channels();
        let label = if is_default {
            format!("{} (Default)", name)
        } else {
            name.clone()
        };
        let description = Some(format!(
            "{} input channel{}",
            channel_count,
            if channel_count == 1 { "" } else { "s" }
        ));

        devices.push(EnumeratedHardwareInputDevice {
            id: format!("{}:{}", DEVICE_ID_PREFIX, index),
            label,
            description,
            channel_count,
            device,
            config: supported_config.into(),
        });
    }

    Ok(devices)
}

fn resolve_selected_device<'a>(
    devices: &'a [EnumeratedHardwareInputDevice],
    requested_device_id: Option<&str>,
) -> Option<&'a EnumeratedHardwareInputDevice> {
    requested_device_id
        .and_then(|requested| devices.iter().find(|device| device.id == requested))
        .or_else(|| devices.first())
}

fn build_channel_options(channel_count: u16) -> Vec<HardwareInputChannelOption> {
    let mut options = Vec::new();

    if channel_count >= 2 {
        for left_channel in (0..channel_count).step_by(2) {
            let right_channel = left_channel + 1;
            if right_channel >= channel_count {
                break;
            }

            options.push(HardwareInputChannelOption {
                id: format!(
                    "{}:{}:{}",
                    STEREO_CHANNEL_PREFIX, left_channel, right_channel
                ),
                label: format!(
                    "Inputs {} + {} (stereo)",
                    left_channel + 1,
                    right_channel + 1
                ),
                description: Some(format!(
                    "Route input {} to left and input {} to right",
                    left_channel + 1,
                    right_channel + 1
                )),
            });
        }
    }

    for channel in 0..channel_count {
        options.push(HardwareInputChannelOption {
            id: format!("{}:{}", MONO_CHANNEL_PREFIX, channel),
            label: format!("Input {} (mono → dual mono)", channel + 1),
            description: Some(format!(
                "Duplicate hardware input {} to both left and right channels",
                channel + 1
            )),
        });
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_hardware_input_routing_roundtrips() {
        let selection = SharedHardwareInputRoutingSelection::default();
        assert_eq!(
            selection.load(),
            HardwareInputRouting {
                left_channel: 0,
                right_channel: Some(1),
            }
        );

        selection.store(HardwareInputRouting {
            left_channel: 3,
            right_channel: None,
        });
        assert_eq!(
            selection.load(),
            HardwareInputRouting {
                left_channel: 3,
                right_channel: None,
            }
        );
    }

    #[test]
    fn routing_from_channel_id_parses_mono_and_stereo() {
        assert_eq!(
            routing_from_channel_id("mono:2"),
            Some(HardwareInputRouting {
                left_channel: 2,
                right_channel: None,
            })
        );
        assert_eq!(
            routing_from_channel_id("stereo:0:1"),
            Some(HardwareInputRouting {
                left_channel: 0,
                right_channel: Some(1),
            })
        );
        assert_eq!(routing_from_channel_id("wat"), None);
    }

    #[test]
    fn stereo_pair_is_preferred_as_default_channel_option() {
        let options = build_channel_options(4);
        assert_eq!(
            options.first().map(|option| option.id.as_str()),
            Some("stereo:0:1")
        );
        assert_eq!(
            resolve_selected_channel_id(None, &options).as_deref(),
            Some("stereo:0:1")
        );
    }
}
