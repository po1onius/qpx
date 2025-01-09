use eframe::egui;
use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
};

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.viewport.resizable = Some(false);
    options.viewport.inner_size = Some(egui::Vec2::new(1280.0, 720.0));
    let _ = eframe::run_native(
        "Resizable Rectangle with Border Drag",
        options,
        Box::new(|_| Ok(Box::new(LevelEditor::from_toml("level_data/new.toml")))),
    );
}

struct EditRect {
    rect_pos: egui::Pos2,           // 矩形左上角位置
    rect_size: egui::Vec2,          // 矩形大小
    is_editing: Option<EditOption>, // 当前正在调整的边框
}

impl Default for EditRect {
    fn default() -> Self {
        Self {
            rect_pos: egui::Pos2 { x: 30.0, y: 30.0 },
            rect_size: egui::Vec2 { x: 30.0, y: 30.0 },
            is_editing: None,
        }
    }
}

#[derive(Default)]
struct LevelEditor {
    rects: Vec<EditRect>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LevelData {
    data: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditOption {
    Left,
    Right,
    Top,
    Bottom,
    Pos(egui::Vec2),
}

const EDGE_THRESHOLD: f32 = 10.0;

impl LevelEditor {
    fn from_toml(path: impl AsRef<Path>) -> Self {
        let data_str_res = read_to_string(path);
        if let Ok(data_str) = data_str_res {
            let lv_data: LevelData = toml::from_str(&data_str).unwrap();
            let mut rects = Vec::new();
            for i in lv_data.data.iter() {
                let rect = EditRect {
                    rect_pos: egui::Pos2 { x: i[0], y: i[1] },
                    rect_size: egui::Vec2::new(i[2], i[3]),
                    is_editing: None,
                };
                rects.push(rect);
            }
            Self { rects }
        } else {
            return Self::default();
        }
    }
}

impl EditRect {
    fn spawn_rect(&mut self, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_size(self.rect_pos, self.rect_size);
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        // 绘制矩形
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(0.0),
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );

        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.rect_pos.x -= 10.0; // 按下左键，向左移动
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.rect_pos.x += 10.0; // 按下右键，向右移动
        }

        // 检测鼠标是否在边框附近
        let mouse_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(mouse_pos) = mouse_pos {
            let mut hold = None;

            // 检测左、右、上、下边框
            if (mouse_pos.x - rect.min.x).abs() < EDGE_THRESHOLD
                && rect.max.y > mouse_pos.y
                && rect.min.y < mouse_pos.y
            {
                hold = Some(EditOption::Left);
            } else if (mouse_pos.x - rect.max.x).abs() < EDGE_THRESHOLD
                && rect.max.y > mouse_pos.y
                && rect.min.y < mouse_pos.y
            {
                hold = Some(EditOption::Right);
            } else if (mouse_pos.y - rect.min.y).abs() < EDGE_THRESHOLD
                && rect.max.x > mouse_pos.x
                && rect.min.x < mouse_pos.x
            {
                hold = Some(EditOption::Top);
            } else if (mouse_pos.y - rect.max.y).abs() < EDGE_THRESHOLD
                && rect.max.x > mouse_pos.x
                && rect.min.x < mouse_pos.x
            {
                hold = Some(EditOption::Bottom);
            } else if rect.contains(mouse_pos) {
                hold = Some(EditOption::Pos(mouse_pos - self.rect_pos));
            }

            // 更新光标图标
            if let Some(edge) = hold {
                let cursor_icon = match edge {
                    EditOption::Left | EditOption::Right => egui::CursorIcon::ResizeHorizontal,
                    EditOption::Top | EditOption::Bottom => egui::CursorIcon::ResizeVertical,
                    EditOption::Pos(_) => egui::CursorIcon::Move,
                };
                ui.output_mut(|o| o.cursor_icon = cursor_icon);
            }

            // 开始调整大小
            if response.drag_started() {
                self.is_editing = hold;
            }

            // 调整矩形大小
            if let Some(edge) = self.is_editing {
                match edge {
                    EditOption::Left => {
                        let width = rect.max.x - mouse_pos.x;
                        if width > 0.0 {
                            self.rect_pos.x = mouse_pos.x;
                            self.rect_size.x = width;
                        }
                    }
                    EditOption::Right => {
                        let width = mouse_pos.x - rect.min.x;
                        if width > 0.0 {
                            self.rect_size.x = width;
                        }
                    }
                    EditOption::Top => {
                        let height = rect.max.y - mouse_pos.y;
                        if height > 0.0 {
                            self.rect_pos.y = mouse_pos.y;
                            self.rect_size.y = height;
                        }
                    }
                    EditOption::Bottom => {
                        let height = mouse_pos.y - rect.min.y;
                        if height > 0.0 {
                            self.rect_size.y = height;
                        }
                    }
                    EditOption::Pos(move_fix) => {
                        //if ui.input(|i| i.pointer.primary_down()) {
                        // 更新矩形位置为鼠标位置
                        self.rect_pos = mouse_pos - move_fix; // - self.rect_pos.to_vec2();
                                                              //}
                    }
                }
            }
        }

        // 结束调整大小
        if response.drag_stopped() {
            self.is_editing = None;
        }
    }
}

impl eframe::App for LevelEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("spawn rect").clicked() {
                let rect = EditRect::default();
                self.rects.push(rect);
            }

            let mut lv_data = LevelData { data: Vec::new() };
            if ui.button("save data").clicked() {
                for rect in self.rects.iter() {
                    let mut vt = Vec::new();
                    vt.push(rect.rect_pos.x);
                    vt.push(rect.rect_pos.y);
                    vt.push(rect.rect_size.x);
                    vt.push(rect.rect_size.y);
                    lv_data.data.push(vt);
                }

                lv_data
                    .data
                    .sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

                let mut file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("level_data/new.toml")
                    .unwrap();
                let s = toml::to_string(&lv_data).unwrap();
                let _ = file.write_all(s.as_bytes());
            }
            for rect in self.rects.iter_mut() {
                rect.spawn_rect(ui);
            }
        });
    }
}

fn egui2bevy(vec: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    vec.to_owned()
}
