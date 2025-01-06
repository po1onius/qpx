use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("eee");
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for i in 0..100 {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());

                        // 绘制背景
                        let color = if response.hovered() {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            egui::Color32::LIGHT_GRAY
                        };
                        ui.painter().rect_filled(rect, 5.0, color);

                        // 在矩形中自定义绘制 Label 的位置
                        let label_text = format!("Label {}", i);
                        let label_pos = rect.min + egui::vec2(20.0, 40.0); // 自定义位置
                        ui.painter().text(
                            label_pos,
                            egui::Align2::LEFT_CENTER,
                            label_text,
                            egui::TextStyle::Body.resolve(ui.style()),
                            egui::Color32::BLACK,
                        );
                    }
                });
            });
        });
    }
}
