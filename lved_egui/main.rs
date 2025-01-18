use eframe::egui;
use std::{
    collections::HashMap,
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
        Box::new(|_| Ok(Box::new(LevelEditor::from_toml("level_data/egui.toml")))),
    );
}

struct EditRect {
    rect_pos: egui::Pos2,           // 矩形左上角位置
    rect_size: egui::Vec2,          // 矩形大小
    is_editing: Option<EditOption>, // 当前正在调整的边框
}

struct EditTri {
    tri_points: [egui::Pos2; 3],
    is_editing: Option<EditOptionTri>,
}

enum EditItem {
    Rect(EditRect),
    Tri(EditTri),
}

impl Default for EditRect {
    fn default() -> Self {
        Self {
            rect_pos: egui::Pos2 { x: 100.0, y: 100.0 },
            rect_size: egui::Vec2 { x: 100.0, y: 100.0 },
            is_editing: None,
        }
    }
}

impl Default for EditTri {
    fn default() -> Self {
        Self {
            tri_points: [
                egui::Pos2::new(130.0, 130.0),
                egui::Pos2::new(145.0, 180.0),
                egui::Pos2::new(190.0, 130.0),
            ],
            is_editing: None,
        }
    }
}

#[derive(Default)]
struct LevelEditor {
    items: Vec<EditItem>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditOptionTri {
    Left(egui::Vec2),
    Mid,
    Right,
}

const EDGE_THRESHOLD: f32 = 10.0;

impl LevelEditor {
    fn from_toml(path: impl AsRef<Path>) -> Self {
        let data_str_res = read_to_string(path);
        if let Ok(data_str) = data_str_res {
            let lv_data: LevelData = toml::from_str(&data_str).unwrap();
            let mut items = Vec::new();
            for i in lv_data.data.iter() {
                if i.len() == 6 {
                    let tri = EditTri {
                        tri_points: [
                            egui::Pos2::new(i[0], i[1]),
                            egui::Pos2::new(i[2], i[3]),
                            egui::Pos2::new(i[4], i[5]),
                        ],
                        is_editing: None,
                    };
                    items.push(EditItem::Tri(tri));
                } else if i.len() == 4 {
                    let rect = EditRect {
                        rect_pos: egui::Pos2 { x: i[0], y: i[1] },
                        rect_size: egui::Vec2::new(i[2], i[3]),
                        is_editing: None,
                    };
                    items.push(EditItem::Rect(rect));
                } else {
                    panic!();
                }
            }
            Self { items }
        } else {
            return Self::default();
        }
    }
}

impl EditRect {
    fn spawn_rect(&mut self, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_size(self.rect_pos, self.rect_size);
        //let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

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

        let mouse_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(mouse_pos) = mouse_pos {
            let mut hold = None;
            if self.is_editing.is_none() {
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
            }
            ui.input(|i| {
                if i.pointer.button_pressed(egui::PointerButton::Primary) {
                    self.is_editing = hold;
                }
                if i.pointer.button_released(egui::PointerButton::Primary) {
                    self.is_editing = None;
                }
            });

            // 更新光标图标
            if let Some(edge) = self.is_editing {
                let cursor_icon = match edge {
                    EditOption::Left | EditOption::Right => egui::CursorIcon::ResizeHorizontal,
                    EditOption::Top | EditOption::Bottom => egui::CursorIcon::ResizeVertical,
                    EditOption::Pos(_) => egui::CursorIcon::Move,
                };
                ui.output_mut(|o| o.cursor_icon = cursor_icon);
            }

            // 开始调整大小

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
                        println!("drag right");
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
    }
}

impl EditTri {
    fn spawn_tri(&mut self, ui: &mut egui::Ui) {
        ui.painter().add(egui::Shape::convex_polygon(
            self.tri_points.to_vec(),
            egui::Color32::LIGHT_BLUE,
            egui::Stroke::new(2.0, egui::Color32::BLACK),
        ));

        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            for i in self.tri_points.iter_mut() {
                i.x -= 10.0;
            }
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            for i in self.tri_points.iter_mut() {
                i.x += 10.0;
            }
        }

        let mouse_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(mouse_pos) = mouse_pos {
            let mut hold = None;
            let m = HashMap::from([
                (0, EditOptionTri::Left(egui::Vec2::default())),
                (1, EditOptionTri::Mid),
                (2, EditOptionTri::Right),
            ]);
            if self.is_editing.is_none() {
                for (j, i) in self.tri_points.iter().enumerate() {
                    let x = (i.x - mouse_pos.x).abs();
                    let y = (i.y - mouse_pos.y).abs();
                    if (x * x + y * y).sqrt() < EDGE_THRESHOLD {
                        hold = Some(m[&j]);
                        break;
                    }
                }
            }
            ui.input(|i| {
                if i.pointer.button_pressed(egui::PointerButton::Primary) {
                    if let Some(EditOptionTri::Left(_)) = hold {
                        self.is_editing = Some(EditOptionTri::Left(mouse_pos - self.tri_points[0]))
                    } else {
                        self.is_editing = hold;
                    }
                }
                if i.pointer.button_released(egui::PointerButton::Primary) {
                    self.is_editing = None;
                }
            });

            // 更新光标图标
            //if let Some(edge) = self.is_editing {
            //    let cursor_icon = match edge {
            //        EditOption::Left | EditOption::Right => egui::CursorIcon::ResizeHorizontal,
            //        EditOption::Top | EditOption::Bottom => egui::CursorIcon::ResizeVertical,
            //        EditOption::Pos(_) => egui::CursorIcon::Move,
            //    };
            //    ui.output_mut(|o| o.cursor_icon = cursor_icon);
            //}

            // 开始调整大小

            // 调整矩形大小
            if let Some(edge) = &self.is_editing {
                match edge {
                    EditOptionTri::Left(fix_pos) => {
                        let mid_fix_pos = self.tri_points[1] - self.tri_points[0];
                        let right_fix_pos = self.tri_points[2] - self.tri_points[0];
                        self.tri_points[0] = mouse_pos - *fix_pos;
                        self.tri_points[1] = self.tri_points[0] + mid_fix_pos;
                        self.tri_points[2] = self.tri_points[0] + right_fix_pos;
                    }
                    EditOptionTri::Mid => {
                        self.tri_points[1].y = mouse_pos.y;
                    }
                    EditOptionTri::Right => {
                        self.tri_points[2].x = mouse_pos.x;
                    }
                }
            }
        }
    }
}

impl eframe::App for LevelEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("spawn rect").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::Rect(rect));
            }

            if ui.button("spawn tri").clicked() {
                let tri = EditTri::default();
                self.items.push(EditItem::Tri(tri));
            }

            let mut lv_data_ori = LevelData { data: Vec::new() };
            if ui.button("save data").clicked() {
                for item in self.items.iter() {
                    let mut vt = Vec::new();
                    match item {
                        EditItem::Rect(rect) => {
                            vt.push(rect.rect_pos.x);
                            vt.push(rect.rect_pos.y);
                            vt.push(rect.rect_size.x);
                            vt.push(rect.rect_size.y);
                        }
                        EditItem::Tri(tri) => {
                            for i in tri.tri_points {
                                vt.push(i.x);
                                vt.push(i.y);
                            }
                        }
                    }
                    lv_data_ori.data.push(vt);
                }

                lv_data_ori
                    .data
                    .sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

                let mut file_ori = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("level_data/egui.toml")
                    .unwrap();
                let s = toml::to_string(&lv_data_ori).unwrap();
                let _ = file_ori.write_all(s.as_bytes());
                let mut file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open("level_data/new.toml")
                    .unwrap();

                egui2bevy(&mut lv_data_ori);
                let s = toml::to_string(&lv_data_ori).unwrap();
                let _ = file.write_all(s.as_bytes());
            }
            for item in self.items.iter_mut() {
                match item {
                    EditItem::Rect(rect) => {
                        rect.spawn_rect(ui);
                    }
                    EditItem::Tri(tri) => {
                        tri.spawn_tri(ui);
                    }
                }
            }
        });
    }
}

fn egui2bevy(ld: &mut LevelData) {
    for i in ld.data.iter_mut() {
        i[0] = i[0] + i[2] / 2.0;
        i[1] = i[1] + i[3] / 2.0;
        i[1] = 720.0 - i[1] - 360.0;
        i[2] /= 2.0;
        i[3] /= 2.0;
    }
}
