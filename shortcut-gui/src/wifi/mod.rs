use eframe::egui;
use egui_toast::{Toast, ToastOptions};
use poll_promise::Promise;
use shortcut_core::tokio::net::UnixStream;
use shortcut_core::tonic::transport::{Channel, Endpoint, Uri};
use shortcut_core::tower::service_fn;
use shortcut_core::wifi;
use shortcut_core::wifi::wifi_service_client;
use shortcut_core::{tokio, tonic};
use std::sync::mpsc;
use std::time::Duration;

use crate::widgets;

const SELECTED_DEVICE_KEY: &str = "selected_device";

#[derive()]
pub struct Shortcut {
    rt: tokio::runtime::Handle,
    power_save_enabled: bool,
    available_devices: Vec<String>,
    selected_device: Option<String>,

    devices_promise: Option<Promise<Result<Vec<String>, tonic::transport::Error>>>,
    power_save_promise: Option<Promise<Result<bool, tonic::transport::Error>>>,

    notifications_tx: mpsc::Sender<Toast>,
}

impl Shortcut {
    pub fn new(
        rt: tokio::runtime::Handle,
        cc: &eframe::CreationContext<'_>,
        notifications_tx: mpsc::Sender<Toast>,
    ) -> Self {
        let (selected_device, power_save_promise) = if let Some(storage) = cc.storage {
            let selected_device = storage.get_string(SELECTED_DEVICE_KEY);
            let promise = selected_device.clone().map(|dev| {
                rt.block_on(
                    async move { Promise::spawn_async(async { get_power_save(dev).await }) },
                )
            });

            (selected_device, promise)
        } else {
            (None, None)
        };

        let devices_promise = rt.block_on(async move {
            tracing::debug!("Creating new devices promise");
            Promise::spawn_async(async { list_devices().await })
        });

        Self {
            rt,
            power_save_enabled: false,
            available_devices: vec![],
            selected_device,

            devices_promise: Some(devices_promise),
            power_save_promise,

            notifications_tx,
        }
    }
}

impl crate::Shortcut for Shortcut {
    fn name(&mut self) -> Option<&str> {
        Some("WiFi")
    }

    fn description(&mut self) -> Option<&str> {
        Some("Update Wifi settings")
    }

    fn draw(&mut self, _ctx: &egui::Context, frame: &mut eframe::Frame, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Devices");
            egui::ComboBox::from_id_source("wifi_devices")
                .selected_text(self.selected_device.as_ref().unwrap_or(&"".to_string()))
                .show_ui(ui, |ui| {
                    for ele in &self.available_devices {
                        if ui
                            .selectable_value(&mut self.selected_device, Some(ele.clone()), ele)
                            .clicked()
                        {
                            if let Some(storage) = frame.storage_mut() {
                                storage.set_string(SELECTED_DEVICE_KEY, ele.clone());
                            }
                            tracing::debug!("Selected device: {ele}");
                        }
                    }
                });
            if let Some(promise) = &self.devices_promise {
                match promise.ready() {
                    None => {}
                    Some(Err(err)) => {
                        tracing::error!("unable to load devices: {err}");
                        self.devices_promise = None;
                    }
                    Some(Ok(devices)) => {
                        tracing::debug!("Promise ready with result: devices={devices:?}");
                        self.available_devices = devices.clone();
                        self.devices_promise = None;
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Power Save");
            ui.set_enabled(self.selected_device.is_some() == self.power_save_promise.is_none());

            if widgets::toggle(ui, &mut self.power_save_enabled).clicked() {
                if let Some(dev) = &self.selected_device {
                    let dev = dev.clone();
                    let enabled = self.power_save_enabled;

                    self.power_save_promise
                        .get_or_insert(self.rt.block_on(async move {
                            Promise::spawn_async(async move {
                                tracing::debug!("Creating new power_save promise");
                                set_power_save(dev.clone(), enabled).await
                            })
                        }));
                }
            }

            if let Some(promise) = &self.power_save_promise {
                match promise.ready() {
                    None => {}
                    Some(Err(err)) => {
                        self.notifications_tx
                            .send(Toast {
                                kind: egui_toast::ToastKind::Error,
                                text: format!("Unable to load power save setting: {err}").into(),
                                options: ToastOptions::with_duration(Duration::from_secs(5)),
                            })
                            .ok();
                        tracing::error!("unable to load power save setting: {err}");
                        self.power_save_promise = None;
                    }
                    Some(Ok(power_save)) => {
                        tracing::debug!("Promise ready with result: power_save={power_save}");
                        self.power_save_enabled = *power_save;
                        self.power_save_promise = None;
                    }
                }
            }
        });
    }
}

async fn get_client(
) -> Result<wifi_service_client::WifiServiceClient<Channel>, tonic::transport::Error> {
    let channel = Endpoint::try_from("http://127.0.0.1:50000")?
        .connect_with_connector(service_fn(|_: Uri| {
            UnixStream::connect(shortcut_core::SOCKET_PATH)
        }))
        .await?;

    Ok(wifi_service_client::WifiServiceClient::new(channel))
}

async fn list_devices() -> Result<Vec<String>, tonic::transport::Error> {
    let mut client = get_client().await?;

    let request = tonic::Request::new(wifi::ListDevicesRequest {});
    let response = client.list_devices(request).await.unwrap();

    let inner = response.into_inner();

    Ok(inner.devices)
}

async fn set_power_save(device: String, enabled: bool) -> Result<bool, tonic::transport::Error> {
    let mut client = get_client().await?;

    let request = tonic::Request::new(wifi::SetPowerSaveRequest { device, enabled });
    let response = client.set_power_save(request).await;

    if response.is_err() {
        return Ok(!enabled);
    }

    let inner = response.unwrap().into_inner();
    Ok(inner.enabled)
}

async fn get_power_save(device: String) -> Result<bool, tonic::transport::Error> {
    let mut client = get_client().await?;

    let request = tonic::Request::new(wifi::GetPowerSaveRequest { device });
    let response = client.get_power_save(request).await;

    if response.is_err() {
        return Ok(true);
    }

    let inner = response.unwrap().into_inner();
    Ok(inner.enabled)
}
