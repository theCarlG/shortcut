use std::time::Duration;

use eframe::egui;
use egui_toast::{Toast, ToastOptions};
use poll_promise::Promise;
use shortcut_core::ssh::ssh_service_client;
use shortcut_core::tokio::net::UnixStream;
use shortcut_core::tonic;
use shortcut_core::tonic::transport::{Channel, Endpoint, Uri};
use shortcut_core::tower::service_fn;
use shortcut_core::{ssh, tokio};
use std::sync::mpsc;

use crate::widgets;

#[derive()]
pub struct Shortcut {
    rt: tokio::runtime::Handle,
    enabled: bool,
    promise: Option<Promise<Result<bool, tonic::transport::Error>>>,
    notifications_tx: mpsc::Sender<Toast>,
}

impl Shortcut {
    pub fn new(rt: tokio::runtime::Handle, notifications_tx: mpsc::Sender<Toast>) -> Self {
        let promise = Some(rt.block_on(async move {
            tracing::debug!("Creating new promise");
            Promise::spawn_async(async { get_enabled().await })
        }));

        Self {
            rt,
            enabled: false,
            promise,
            notifications_tx,
        }
    }
}

impl crate::Shortcut for Shortcut {
    fn name(&mut self) -> Option<&str> {
        Some("Remote Access")
    }

    fn description(&mut self) -> Option<&str> {
        Some("Enable remote access via SSH")
    }

    fn draw(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Enable");
            if widgets::toggle(ui, &mut self.enabled).clicked() {
                let enabled = self.enabled;

                self.promise.get_or_insert(self.rt.block_on(async move {
                    tracing::debug!("Creating new promise");
                    Promise::spawn_async(async move { set_enabled(enabled).await })
                }));
            }
        });

        if let Some(promise) = &self.promise {
            match promise.ready() {
                None => {}
                Some(Err(err)) => {
                    self.notifications_tx
                        .send(Toast {
                            kind: egui_toast::ToastKind::Error,
                            text: format!("Unable to load remote access setting: {err}").into(),
                            options: ToastOptions::with_duration(Duration::from_secs(5)),
                        })
                        .ok();
                    tracing::error!("unable to load remote access setting: {err}");
                    self.promise = None;
                }
                Some(Ok(enabled)) => {
                    tracing::debug!("Promise ready with result: {enabled}");
                    self.enabled = *enabled;
                    self.promise = None;
                }
            }
        }
    }
}

async fn get_client(
) -> Result<ssh_service_client::SshServiceClient<Channel>, tonic::transport::Error> {
    let channel = Endpoint::try_from("http://127.0.0.1:50000")?
        .connect_with_connector(service_fn(|_: Uri| {
            UnixStream::connect(shortcut_core::SOCKET_PATH)
        }))
        .await?;

    Ok(ssh_service_client::SshServiceClient::new(channel))
}

async fn set_enabled(enabled: bool) -> Result<bool, tonic::transport::Error> {
    let mut client = get_client().await?;

    let request = tonic::Request::new(ssh::SetEnabledRequest { enabled });
    let response = client.set_enabled(request).await;

    if response.is_err() {
        return Ok(!enabled);
    }

    let inner = response.unwrap().into_inner();
    Ok(inner.enabled)
}

async fn get_enabled() -> Result<bool, tonic::transport::Error> {
    let mut client = get_client().await?;

    let request = tonic::Request::new(ssh::GetEnabledRequest {});
    let response = client.get_enabled(request).await;

    if response.is_err() {
        return Ok(true);
    }

    let inner = response.unwrap().into_inner();
    Ok(inner.enabled)
}
