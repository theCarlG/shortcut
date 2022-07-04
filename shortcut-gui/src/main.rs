use eframe::egui;
use eframe::epaint::Color32;
use egui_toast::{Toast, Toasts};
use shortcut_core::tokio;
use std::sync::mpsc;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod audio;
mod ssh;
mod style;
mod widgets;
mod wifi;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
        let collector = tracing_subscriber::registry()
            .with(tracing_journald::layer().unwrap())
            .with(EnvFilter::from_default_env());
        tracing::subscriber::set_global_default(collector)
            .expect("Unable to set a global collector");
    } else {
        let collector = tracing_subscriber::registry()
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(EnvFilter::from_default_env());
        tracing::subscriber::set_global_default(collector)
            .expect("Unable to set a global collector");
    }

    tracing::info!("Logging initialized");

    let options = eframe::NativeOptions {
        decorated: false,
        transparent: false,
        min_window_size: Some(egui::vec2(1280.0, 800.0)),
        max_window_size: Some(egui::vec2(1280.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Shortcut",
        options,
        Box::new(|cc| Box::new(SteamDeckApp::new(cc))),
    );
}

pub(crate) trait Shortcut {
    fn draw(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, ui: &mut egui::Ui);
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}
    fn description(&mut self) -> Option<&str> {
        None
    }
    fn name(&mut self) -> Option<&str> {
        None
    }
}

struct SteamDeckApp {
    #[allow(dead_code)]
    rt: tokio::runtime::Runtime,
    notifications_rx: mpsc::Receiver<Toast>,

    shortcuts: Vec<Box<dyn Shortcut>>,
}

const PIXEL_PER_POINT: f32 = 1.5;

impl SteamDeckApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let pixels = cc.egui_ctx.pixels_per_point();
        cc.egui_ctx.set_pixels_per_point(pixels * PIXEL_PER_POINT);

        let (tx, notifications_rx) = mpsc::channel();

        style::setup_fonts(&cc.egui_ctx);
        style::setup_visuals(&cc.egui_ctx);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let shortcuts: Vec<Box<dyn Shortcut>> = vec![
            Box::new(ssh::Shortcut::new(rt.handle().clone(), tx.clone())),
            Box::new(wifi::Shortcut::new(rt.handle().clone(), cc, tx.clone())),
            Box::new(audio::Shortcut::new(rt.handle().clone(), cc, tx)),
        ];

        Self {
            rt,
            shortcuts,
            notifications_rx,
        }
    }
}

impl eframe::App for SteamDeckApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        tracing::debug!("Saving state");
        for shortcut in self.shortcuts.iter_mut() {
            shortcut.save(storage);
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);
            ui.heading("SteamDeck Shortcuts");

            for shortcut in self.shortcuts.iter_mut() {
                if let Some(name) = shortcut.name() {
                    ui.strong(name);
                }
                shortcut.draw(ctx, frame, ui);

                if let Some(description) = shortcut.description() {
                    ui.small(
                        egui::RichText::new(description).color(Color32::from_rgb(150, 150, 150)),
                    );
                }
                ui.separator();
            }
        });

        let mut toasts = Toasts::new(ctx)
            .anchor((1280.0 / PIXEL_PER_POINT, 800.0 / PIXEL_PER_POINT))
            .direction(egui::Direction::BottomUp)
            .align_to_end(true);

        if let Ok(msg) = self.notifications_rx.try_recv() {
            toasts.add(msg.text, msg.kind, msg.options);
        }

        toasts.show();
    }
}
