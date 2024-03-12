use bucface_utils::EventDB;
use egui::{Align, Layout, Rgba};

use crate::app::App;

pub fn body(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        header(ui);
        ui.horizontal(|ui| {
            log_entry(ui, app);
            log_panel(ui, app);
        })
    });
}

fn header(ui: &mut egui::Ui) {
    ui.heading("BucFace Client v0.1");
}

fn log_entry(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        ui.label("Log");
        ui.text_edit_multiline(&mut app.log_buf);
        if ui.button("Send Log").clicked() {
            let _ = app.send_log();
        }
    });
}

fn log_panel(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        // create vertical collumn of all logs from App::logs
        ui.label("Logs");
        if ui.button("Refresh").clicked() {
            app.logs.clear();
            app.log_ids.clear();
            let _ = app.ws_client.get_logs_since(0);
        }

        let print_event = |ui: &mut egui::Ui, event: &EventDB| {
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(Rgba::from_rgb(0.0, 1.0, 0.5), &event.author);
                    ui.colored_label(Rgba::from_rgb(0.1, 0.1, 0.1), "@");
                    ui.colored_label(Rgba::from_rgb(0.5, 1.0, 0.5), &event.machine);
                });
                ui.colored_label(Rgba::from_rgb(0.1, 0.7, 0.9), &event.event);
            });
        };

        /* let print_error = |ui: &mut egui::Ui, error: &EventDBErrorSerde| {
            ui.colored_label(Rgba::from_rgb(1., 0., 0.), format!("Error: {:?}", error));
        }; */

        for log in &app.logs {
            let text = |ui: &mut egui::Ui| print_event(ui, log);

            let time = |ui: &mut egui::Ui| {
                ui.colored_label(Rgba::from_rgb(0.5, 0.7, 0.9), log.time.to_string())
            };

            ui.horizontal_wrapped(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    time(ui);
                    ui.with_layout(
                        Layout::left_to_right(Align::Min).with_main_wrap(true),
                        |ui| {
                            text(ui);
                        },
                    );
                });
            });
            ui.separator();
        }
    });
}
