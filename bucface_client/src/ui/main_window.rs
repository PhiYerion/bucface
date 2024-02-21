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
            app.send_logs();
        }
    });
}

fn log_panel(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical(|ui| {
        // create vertical collumn of all logs from App::logs
        ui.label("Logs");
        if ui.button("Refresh").clicked() {
            app.get_new_logs();
        }
        for log in &app.logs {
            ui.label(&*log.event.clone());
        }
    });
}

