use eframe::egui;
use egui_toast::{Toast, ToastOptions};
use pulse::context::subscribe::{self, Facility, Operation};
use pulsectl::controllers::types::DeviceInfo;
use pulsectl::controllers::{DeviceControl, SinkController, SourceController};
use shortcut_core::tokio;
use std::sync::mpsc;
use std::time::Duration;

struct Controller {
    title: String,
    inner: Box<dyn DeviceControl<DeviceInfo>>,
    default: Option<(String, String)>,
    devices: Vec<(String, String)>,
}

impl Controller {
    fn new(title: String, controller: Box<dyn DeviceControl<DeviceInfo>>) -> Self {
        Self {
            title,
            inner: controller,
            default: None,
            devices: vec![],
        }
    }
}

#[derive()]
pub struct Shortcut {
    notifications_tx: mpsc::Sender<Toast>,
    device_changes: mpsc::Receiver<()>,

    controllers: Vec<Controller>,
}

impl Shortcut {
    pub fn new(
        _rt: tokio::runtime::Handle,
        cc: &eframe::CreationContext<'_>,
        notifications_tx: mpsc::Sender<Toast>,
    ) -> Self {
        let mut controllers = Vec::new();

        let controller = SourceController::create().expect("unable to create sink controller");
        let rx = Self::setup_subscriber(&controller, cc.egui_ctx.clone());

        controllers.push(Controller::new("Input".to_string(), Box::new(controller)));
        controllers.push(Controller::new(
            "Output".to_string(),
            Box::new(SinkController::create().expect("unable to create sink controller")),
        ));

        Self {
            notifications_tx,
            device_changes: rx,

            controllers,
        }
    }

    fn setup_subscriber(controller: &SourceController, ctx: egui::Context) -> mpsc::Receiver<()> {
        controller
            .handler
            .context
            .borrow_mut()
            .subscribe(subscribe::InterestMaskSet::ALL, |_| {});

        let (tx, rx) = mpsc::channel();
        tx.send(()).ok();
        controller
            .handler
            .context
            .borrow_mut()
            .set_subscribe_callback(Some(Box::new(move |facility, operation, index| {
                if Some(Facility::Card) == facility {
                    match operation {
                        Some(Operation::New) | Some(Operation::Removed) => {
                            tracing::debug!(
                                "Card update: facility: {:?}, operation: {:?}, index: {index}",
                                facility.unwrap(),
                                operation.unwrap()
                            );
                            tx.send(()).ok();
                            ctx.request_repaint();
                        }
                        _ => (),
                    }
                }
            })));

        rx
    }

    fn update_devices(&mut self) {
        self.controllers.iter_mut().for_each(|controller| {
            controller.default = match controller.inner.get_default_device() {
                Ok(dev) => {
                    let name = dev.name.as_ref().unwrap().to_string();
                    let description = dev.description.as_ref().unwrap().to_string();
                    Some((name, description))
                }
                Err(err) => {
                    tracing::error!(
                        "Error when getting default device for {}: {err}",
                        controller.title
                    );
                    None
                }
            };

            controller.devices = match controller.inner.list_devices() {
                Ok(devices) => devices
                    .iter()
                    .filter_map(|dev| {
                        let name = dev.name.as_ref().unwrap().to_string();
                        let description = dev.description.as_ref().unwrap().to_string();
                        if !name.ends_with(".monitor") {
                            Some((name, description))
                        } else {
                            None
                        }
                    })
                    .collect(),
                Err(err) => {
                    tracing::error!("Error when getting devices for {}: {err}", controller.title);
                    vec![]
                }
            }
        });
    }
}

impl crate::Shortcut for Shortcut {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // Not the prettiest solution but wth
        self.update_devices()
    }

    fn name(&mut self) -> Option<&str> {
        Some("Audio")
    }

    fn description(&mut self) -> Option<&str> {
        Some("Update default audio devices")
    }

    fn draw(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame, ui: &mut egui::Ui) {
        if let Ok(()) = self.device_changes.try_recv() {
            tracing::debug!("Device update");
            self.update_devices();
        }

        self.controllers.iter_mut().for_each(|controller| {
            ui.horizontal(|ui| {
                ui.label(format!("{} Device", controller.title));
                egui::ComboBox::from_id_source(format!("{}_devices", controller.title))
                    .width(300.0)
                    .selected_text(
                        controller
                            .default
                            .clone()
                            .unwrap_or((String::new(), String::new()))
                            .1,
                    )
                    .show_ui(ui, |ui| {
                        ui.set_max_width(300.0);
                        controller.devices.iter().for_each(|dev| {
                            if ui
                                .selectable_value(
                                    &mut controller.default,
                                    Some(dev.clone()),
                                    dev.1.clone(),
                                )
                                .clicked()
                            {
                                match controller.inner.set_default_device(&dev.0) {
                                    Ok(_done) => {}
                                    Err(err) => {
                                        tracing::error!("unable to set default source: {err}");
                                        self.notifications_tx
                                            .send(Toast {
                                                kind: egui_toast::ToastKind::Error,
                                                text: format!("Unable to set default input: {err}")
                                                    .into(),
                                                options: ToastOptions::with_duration(
                                                    Duration::from_secs(5),
                                                ),
                                            })
                                            .ok();
                                    }
                                }
                            };
                        });
                    });
            });
        });
    }
}
