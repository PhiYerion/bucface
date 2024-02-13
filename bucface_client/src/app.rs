#[derive(Default)]
pub struct App {
    log_buf: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            body(ui, &mut self.log_buf);
        });
    }
}

fn header(ui: &mut egui::Ui) {
    ui.heading("BucFace Client v0.1");
}

fn log_entry(ui: &mut egui::Ui, buf: &mut String) {
    ui.vertical(|ui| {
        ui.label("Log");
        ui.text_edit_multiline(buf);
    });
}

fn log_panel(ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label("Log Panel - WIP");
    });
}

fn body(ui: &mut egui::Ui, log_buf: &mut String) {
    ui.vertical(|ui| {
        header(ui);
        ui.horizontal(|ui| {
            log_entry(ui, log_buf);
            log_panel(ui);
        })
    });
}

fn send_log_entry(log_buf: &str) {
    println!("{}", log_buf);
}
