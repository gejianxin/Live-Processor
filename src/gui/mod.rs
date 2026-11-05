pub mod app;
pub mod matrix;
pub mod rack;
pub mod snapshots;
pub mod supervisor;

use anyhow::Result;

pub fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1140.0, 740.0])
            .with_min_inner_size([960.0, 620.0])
            .with_title("Live-Processor Pro - Audio Processing & Supervisor"),
        ..Default::default()
    };

    eframe::run_native(
        "Live-Processor Pro",
        options,
        Box::new(|cc| {
            // Apply professional dark audio console theme
            configure_audio_theme(&cc.egui_ctx);
            Ok(Box::new(app::LiveProcessorApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to launch GUI: {}", e))
}

fn configure_audio_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(220, 225, 235));
    visuals.panel_fill = egui::Color32::from_rgb(18, 21, 26);
    visuals.window_fill = egui::Color32::from_rgb(22, 26, 32);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 28, 36);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 46, 58));

    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 36, 46);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 58, 72));

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(42, 50, 65);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 210, 255));

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 160, 210);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 240, 255));

    ctx.set_visuals(visuals);
}
