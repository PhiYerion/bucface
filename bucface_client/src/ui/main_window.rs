use crate::app::App;

use super::log_ui::{log_entry, log_panel};

pub fn body(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App) {
    ui.vertical(|ui| {
        header(ui);
        server_entry(ui, ctx, app);
        ui.horizontal(|ui| {
            log_entry(ui, app);
            log_panel(ui, app);
        })
    });
}

fn header(ui: &mut egui::Ui) {
    ui.heading("BucFace Client v0.1");
}

fn server_entry(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App<'_>) {
    ui.horizontal(|ui| {
        ui.label("Server");
        ui.text_edit_singleline(&mut app.bufs.server);
        ui.label("Port");
        ui.text_edit_singleline(&mut app.bufs.port);
        if ui.button("Connect").clicked() {
            app.set_endpoint(ctx);
        }
        ui.label("Status");
        ui.label(app.ws_client.to_string());
    });
}
