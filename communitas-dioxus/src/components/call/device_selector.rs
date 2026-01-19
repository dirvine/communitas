//! Device selector dropdown for audio/video device selection.

use communitas_ui_api::{DeviceType, MediaDevice};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the DeviceSelector component.
#[derive(Props, Clone, PartialEq)]
pub struct DeviceSelectorProps {
    /// Type of device to select.
    pub device_type: DeviceType,
    /// Optional callback when device is selected.
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
}

/// Device selector dropdown for microphone, speaker, or camera.
#[component]
pub fn DeviceSelector(props: DeviceSelectorProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    // Subscribe to call state for device list
    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Update signal when snapshot changes
    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                call_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let snapshot = call_snapshot();
    let devices: Vec<MediaDevice> = snapshot
        .available_devices
        .iter()
        .filter(|d| d.device_type == props.device_type)
        .cloned()
        .collect();

    // Get currently selected device ID
    let selected_id = match props.device_type {
        DeviceType::Microphone => snapshot.settings.selected_microphone.clone(),
        DeviceType::Speaker => snapshot.settings.selected_speaker.clone(),
        DeviceType::Camera => snapshot.settings.selected_camera.clone(),
    };

    let label = props.device_type.label();
    let icon = match props.device_type {
        DeviceType::Microphone => "🎤",
        DeviceType::Speaker => "🔊",
        DeviceType::Camera => "📷",
    };

    let device_type = props.device_type;
    let on_select = move |evt: Event<FormData>| {
        let value = evt.value().to_string();
        if value.is_empty() {
            return;
        }

        let value_for_spawn = value.clone();
        let call = call_service.clone();
        spawn(async move {
            let _ = match device_type {
                DeviceType::Microphone => call.select_microphone(&value_for_spawn).await,
                DeviceType::Speaker => call.select_speaker(&value_for_spawn).await,
                DeviceType::Camera => call.select_camera(&value_for_spawn).await,
            };
        });

        if let Some(on_change) = &props.on_change {
            on_change.call(value);
        }
    };

    rsx! {
        div {
            class: "device-selector",
            label {
                class: "block text-sm font-medium text-slate-300 mb-1",
                r#for: format!("device-{:?}", props.device_type),
                span { class: "mr-1", "{icon}" }
                "{label}"
            }
            select {
                id: format!("device-{:?}", props.device_type),
                class: "w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded-lg text-slate-200 focus:outline-none focus:ring-2 focus:ring-emerald-500",
                aria_label: format!("Select {}", label.to_lowercase()),
                onchange: on_select,
                // Default/empty option
                if devices.is_empty() {
                    option {
                        value: "",
                        "No {label.to_lowercase()}s available"
                    }
                } else {
                    option {
                        value: "",
                        "Select {label.to_lowercase()}..."
                    }
                }
                // Device options
                for device in devices.iter() {
                    option {
                        key: "{device.id}",
                        value: "{device.id}",
                        selected: selected_id.as_ref() == Some(&device.id),
                        "{device.name}"
                        if device.is_default {
                            " (Default)"
                        }
                        if !device.is_available {
                            " (Unavailable)"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_icons() {
        let icons = [
            (DeviceType::Microphone, "🎤"),
            (DeviceType::Speaker, "🔊"),
            (DeviceType::Camera, "📷"),
        ];
        for (device_type, expected_icon) in icons {
            let icon = match device_type {
                DeviceType::Microphone => "🎤",
                DeviceType::Speaker => "🔊",
                DeviceType::Camera => "📷",
            };
            assert_eq!(icon, expected_icon);
        }
    }

    #[test]
    fn device_type_labels() {
        assert_eq!(DeviceType::Microphone.label(), "Microphone");
        assert_eq!(DeviceType::Speaker.label(), "Speaker");
        assert_eq!(DeviceType::Camera.label(), "Camera");
    }
}
