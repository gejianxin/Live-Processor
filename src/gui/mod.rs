pub mod app;
pub mod matrix;
pub mod rack;
pub mod snapshots;
pub mod supervisor;

use anyhow::Result;

pub fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1220.0, 780.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("Live-Processor Pro"),
        ..Default::default()
    };

    eframe::run_native(
        "Live-Processor Pro",
        options,
        Box::new(|cc| {
            configure_audio_theme(&cc.egui_ctx);
            Ok(Box::new(app::LiveProcessorApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to launch GUI: {}", e))
}

fn configure_audio_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Minimalist Studio Palette
    let bg_canvas = egui::Color32::from_rgb(18, 20, 24);
    let bg_surface = egui::Color32::from_rgb(25, 29, 35);
    let bg_widget = egui::Color32::from_rgb(32, 37, 46);
    let border_subtle = egui::Color32::from_rgb(44, 52, 64);
    let text_primary = egui::Color32::from_rgb(230, 236, 244);
    let accent_teal = egui::Color32::from_rgb(56, 189, 248);

    visuals.override_text_color = Some(text_primary);
    visuals.panel_fill = bg_canvas;
    visuals.window_fill = bg_surface;

    visuals.widgets.noninteractive.bg_fill = bg_surface;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_subtle);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_subtle);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(40, 48, 60);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent_teal);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(28, 54, 78);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent_teal);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(34, 40, 50);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, border_subtle);

    ctx.set_visuals(visuals);
}
