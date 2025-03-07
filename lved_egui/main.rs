use eframe::egui::{self, Align2, Color32, FontId, Pos2, Stroke};
use std::{
    collections::HashMap,
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
};

const EDGE_THRESHOLD: f32 = 10.0;
const WINDOW_SIZE_X: f32 = 1280.0;
const WINDOW_SIZE_Y: f32 = 720.0;
const DROP_AREA_Y: f32 = 30.0;

fn main() {
    env_logger::init();
    let mut options = eframe::NativeOptions::default();
    options.viewport.resizable = Some(false);
    options.viewport.inner_size = Some(egui::Vec2::new(WINDOW_SIZE_X, WINDOW_SIZE_Y));
    let _ = eframe::run_native(
        "qpx level data editor",
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

struct EditCircle {
    circle_pos: egui::Pos2,
    radius: f32,
    is_editing: Option<EditOptionCircle>,
}

enum EditItem {
    Floor(EditRect),
    TriObstacle(EditTri),
    RectObstacle(EditRect),
    DoubleJump(EditCircle),
    RectFlyBegin(EditRect),
    RectFlyEnd(EditRect),
    Pass(EditRect),
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
                egui::Pos2::new(160.0, 180.0),
                egui::Pos2::new(190.0, 130.0),
            ],
            is_editing: None,
        }
    }
}

impl Default for EditCircle {
    fn default() -> Self {
        Self {
            circle_pos: egui::Pos2::new(100.0, 100.0),
            radius: 30.0,
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
    data: Vec<(u32, Vec<f32>)>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum EditOptionCircle {
    Pos(egui::Vec2),
    Radius,
}

impl LevelEditor {
    fn from_toml(path: impl AsRef<Path>) -> Self {
        let data_str_res = read_to_string(path);
        if let Ok(data_str) = data_str_res {
            let lv_data: LevelData = toml::from_str(&data_str).unwrap();
            let mut items = Vec::new();
            for (typ, i) in lv_data.data.iter() {
                if i.len() == 6 {
                    let tri = EditTri {
                        tri_points: [
                            egui::Pos2::new(i[0], i[1]),
                            egui::Pos2::new(i[2], i[3]),
                            egui::Pos2::new(i[4], i[5]),
                        ],
                        is_editing: None,
                    };
                    if *typ == 1 {
                        items.push(EditItem::TriObstacle(tri));
                    } else {
                        panic!();
                    }
                } else if i.len() == 4 {
                    let rect = EditRect {
                        rect_pos: egui::Pos2 { x: i[0], y: i[1] },
                        rect_size: egui::Vec2::new(i[2], i[3]),
                        is_editing: None,
                    };
                    if *typ == 0 {
                        items.push(EditItem::Floor(rect));
                    } else if *typ == 2 {
                        items.push(EditItem::RectObstacle(rect));
                    } else if *typ == 4 {
                        items.push(EditItem::RectFlyBegin(rect));
                    } else if *typ == 5 {
                        items.push(EditItem::RectFlyEnd(rect));
                    } else {
                        items.push(EditItem::Pass(rect));
                    }
                } else if i.len() == 3 {
                    let circle = EditCircle {
                        circle_pos: egui::Pos2 { x: i[0], y: i[1] },
                        radius: i[2],
                        is_editing: None,
                    };
                    if *typ == 3 {
                        items.push(EditItem::DoubleJump(circle));
                    } else {
                        panic!();
                    }
                }
            }
            Self { items }
        } else {
            return Self::default();
        }
    }
}

impl EditCircle {
    fn spawn_circle(&mut self, ui: &mut egui::Ui) -> bool {
        ui.painter().circle_stroke(
            self.circle_pos,
            self.radius,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );

        if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.circle_pos.x -= 10.0; // 按下左键，向左移动
        }
        if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.circle_pos.x += 10.0; // 按下右键，向右移动
        }

        let mouse_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(mouse_pos) = mouse_pos {
            let mut hold = None;
            if self.is_editing.is_none() {
                if (mouse_pos.distance(self.circle_pos) - self.radius).abs() < EDGE_THRESHOLD {
                    hold = Some(EditOptionCircle::Radius);
                } else if mouse_pos.distance(self.circle_pos) < EDGE_THRESHOLD {
                    hold = Some(EditOptionCircle::Pos(mouse_pos - self.circle_pos))
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

            if let Some(edge) = self.is_editing {
                match edge {
                    EditOptionCircle::Radius => {
                        self.radius = mouse_pos.distance(self.circle_pos);
                    }
                    EditOptionCircle::Pos(move_fix) => {
                        //if ui.input(|i| i.pointer.primary_down()) {
                        // 更新矩形位置为鼠标位置
                        self.circle_pos = mouse_pos - move_fix; // - self.rect_pos.to_vec2();
                                                                //}
                    }
                }
            }
        }

        if self.circle_pos.y < DROP_AREA_Y {
            return false;
        }
        return true;
    }
}

impl EditRect {
    fn spawn_rect(&mut self, ui: &mut egui::Ui, color: egui::Color32) -> bool {
        let rect = egui::Rect::from_min_size(self.rect_pos, self.rect_size);
        //let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        // 绘制矩形
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(0.0),
            egui::Stroke::new(2.0, color),
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
        if self.rect_pos.y < DROP_AREA_Y {
            return false;
        }
        return true;
    }
}

impl EditTri {
    fn spawn_tri(&mut self, ui: &mut egui::Ui) -> bool {
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
                        if mouse_pos.x > self.tri_points[0].x + 10.0 {
                            self.tri_points[2].x = mouse_pos.x;
                        }
                        self.tri_points[1].x = (self.tri_points[2].x - self.tri_points[0].x) / 2.0
                            + self.tri_points[0].x;
                    }
                }
            }
        }

        if self.tri_points[0].y < DROP_AREA_Y {
            return false;
        }
        return true;
    }
}

impl eframe::App for LevelEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().line_segment(
                [
                    Pos2::new(0.0, DROP_AREA_Y),
                    Pos2::new(WINDOW_SIZE_X, DROP_AREA_Y),
                ],
                Stroke::new(1.0, Color32::RED),
            );

            ui.painter().text(
                Pos2::new(WINDOW_SIZE_X / 2.0, DROP_AREA_Y / 2.0),
                Align2::CENTER_CENTER,
                "drop here to delete item",
                FontId::default(),
                Color32::RED,
            );

            if ui.button("spawn floor").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::Floor(rect));
            }

            if ui.button("spawn tri").clicked() {
                let tri = EditTri::default();
                self.items.push(EditItem::TriObstacle(tri));
            }

            if ui.button("spawn circle").clicked() {
                let circle = EditCircle::default();
                self.items.push(EditItem::DoubleJump(circle));
            }

            if ui.button("spawn rect obstacle").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::RectObstacle(rect));
            }

            if ui.button("spawn rect fly begin").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::RectFlyBegin(rect));
            }

            if ui.button("spawn rect fly end").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::RectFlyEnd(rect));
            }

            if ui.button("spawn level pass").clicked() {
                let rect = EditRect::default();
                self.items.push(EditItem::Pass(rect));
            }

            let mut lv_data_ori = LevelData { data: Vec::new() };
            if ui.button("save data").clicked() {
                for item in self.items.iter() {
                    let mut vt = Vec::new();
                    let mut typ: i32 = -1;
                    let mut rec = &EditRect::default();
                    match item {
                        EditItem::Floor(rect) => {
                            typ = 0;
                            rec = rect;
                        }
                        EditItem::RectObstacle(rect) => {
                            typ = 2;
                            rec = rect;
                        }
                        EditItem::RectFlyBegin(rect) => {
                            typ = 4;
                            rec = rect;
                        }
                        EditItem::RectFlyEnd(rect) => {
                            typ = 5;
                            rec = rect;
                        }
                        EditItem::Pass(rect) => {
                            typ = 6;
                            rec = rect;
                        }
                        EditItem::TriObstacle(tri) => {
                            typ = 1;
                            for i in tri.tri_points {
                                vt.push(i.x);
                                vt.push(i.y);
                            }
                        }
                        EditItem::DoubleJump(circle) => {
                            typ = 3;
                            vt.push(circle.circle_pos.x);
                            vt.push(circle.circle_pos.y);
                            vt.push(circle.radius);
                        }
                    }
                    if typ != 1 && typ != 3 {
                        vt.push(rec.rect_pos.x);
                        vt.push(rec.rect_pos.y);
                        vt.push(rec.rect_size.x);
                        vt.push(rec.rect_size.y);
                    }
                    lv_data_ori.data.push((typ as u32, vt));
                }

                lv_data_ori
                    .data
                    .sort_by(|a, b| a.1[0].partial_cmp(&b.1[0]).unwrap());

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
            let mut drop_idx = -1;
            for (i, item) in self.items.iter_mut().enumerate() {
                match item {
                    EditItem::Floor(rect) => {
                        if !rect.spawn_rect(ui, egui::Color32::WHITE) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::RectObstacle(rect) => {
                        if !rect.spawn_rect(ui, egui::Color32::RED) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::RectFlyBegin(rect) => {
                        if !rect.spawn_rect(ui, egui::Color32::GREEN) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::RectFlyEnd(rect) => {
                        if !rect.spawn_rect(ui, egui::Color32::BLUE) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::Pass(rect) => {
                        if !rect.spawn_rect(ui, egui::Color32::YELLOW) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::TriObstacle(tri) => {
                        if !tri.spawn_tri(ui) {
                            drop_idx = i as i32;
                        }
                    }
                    EditItem::DoubleJump(circle) => {
                        if !circle.spawn_circle(ui) {
                            drop_idx = i as i32;
                        }
                    }
                }
            }
            if drop_idx >= 0 {
                self.items.remove(drop_idx as usize);
            }
        });
    }
}

fn egui2bevy(ld: &mut LevelData) {
    for (_, i) in ld.data.iter_mut() {
        if i.len() == 4 {
            i[0] = i[0] + i[2] / 2.0;
            i[1] = i[1] + i[3] / 2.0;
            i[1] = 720.0 - i[1] - 360.0;
            i[2] /= 2.0;
            i[3] /= 2.0;
        } else if i.len() == 6 {
            i[1] = 720.0 - i[1] - 360.0;
            i[3] = 720.0 - i[3] - 360.0;
            i[5] = 720.0 - i[5] - 360.0;
        } else if i.len() == 3 {
            i[1] = 720.0 - i[1] - 360.0;
        } else {
            panic!();
        }
    }
}
