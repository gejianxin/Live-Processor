pub mod app;
pub mod live_processor;
pub mod matrix;
pub mod rack;
pub mod snapshots;
pub mod supervisor;

use anyhow::Result;

pub fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 720.0])
            .with_min_inner_size([980.0, 580.0])
            .with_title("Dialog Audio Live-Processor Pro"),
        ..Default::default()
    };

    eframe::run_native(
        "Live-Processor Pro",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            configure_audio_theme(&cc.egui_ctx);
            Ok(Box::new(app::LiveProcessorApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to launch GUI: {}", e))
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(windows)]
    let candidates = [
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
    ];

    #[cfg(not(windows))]
    let candidates = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/System/Library/Fonts/PingFang.ttc",
    ];

    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "system_cjk".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );

            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.insert(0, "system_cjk".to_owned());
            }

            if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                mono.push("system_cjk".to_owned());
            }

            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn configure_audio_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Authentic Dialog Audio Studio Broadcast Palette
    let bg_canvas = egui::Color32::from_rgb(14, 21, 30);      // #0E151E Deep Navy Broadcast
    let bg_surface = egui::Color32::from_rgb(20, 28, 40);     // #141C28 Studio Surface
    let bg_widget = egui::Color32::from_rgb(26, 36, 50);      // #1A2432 Control Plate
    let border_subtle = egui::Color32::from_rgb(38, 52, 72);  // #263448 Precise Divider
    let text_primary = egui::Color32::from_rgb(240, 246, 252);
    let accent_cyan = egui::Color32::from_rgb(56, 189, 248);   // #38BDF8 Glowing Cyan

    visuals.override_text_color = Some(text_primary);
    visuals.panel_fill = bg_canvas;
    visuals.window_fill = bg_surface;

    visuals.widgets.noninteractive.bg_fill = bg_surface;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_subtle);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(5);

    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_subtle);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(5);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(32, 46, 64);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent_cyan);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(5);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(24, 48, 72);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent_cyan);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(5);

    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(28, 40, 56);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, border_subtle);

    ctx.set_visuals(visuals);
}
