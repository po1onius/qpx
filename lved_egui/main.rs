use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Resizable Rectangle with Border Drag",
        options,
        Box::new(|_cc| Ok(Box::<ResizableRectangleApp>::default())),
    );
}

struct ResizableRectangleApp {
    rect_pos: egui::Pos2,            // 矩形左上角位置
    rect_size: egui::Vec2,           // 矩形大小
    is_resizing: Option<ResizeEdge>, // 当前正在调整的边框
}

impl Default for ResizableRectangleApp {
    fn default() -> Self {
        Self {
            rect_pos: egui::Pos2 { x: 30.0, y: 30.0 },
            rect_size: egui::vec2(30.0, 30.0),
            is_resizing: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl eframe::App for ResizableRectangleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = egui::Rect::from_min_size(self.rect_pos, self.rect_size);
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

            // 绘制矩形
            ui.painter().rect_stroke(
                rect,
                egui::Rounding::same(0.0),
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            // 检测鼠标是否在边框附近
            let mouse_pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(mouse_pos) = mouse_pos {
                let edge_threshold = 5.0; // 边框检测的阈值
                let mut hovered_edge = None;

                // 检测左、右、上、下边框
                if (mouse_pos.x - rect.min.x).abs() < edge_threshold {
                    hovered_edge = Some(ResizeEdge::Left);
                } else if (mouse_pos.x - rect.max.x).abs() < edge_threshold {
                    hovered_edge = Some(ResizeEdge::Right);
                } else if (mouse_pos.y - rect.min.y).abs() < edge_threshold {
                    hovered_edge = Some(ResizeEdge::Top);
                } else if (mouse_pos.y - rect.max.y).abs() < edge_threshold {
                    hovered_edge = Some(ResizeEdge::Bottom);
                }

                // 更新光标图标
                if let Some(edge) = hovered_edge {
                    let cursor_icon = match edge {
                        ResizeEdge::Left | ResizeEdge::Right => egui::CursorIcon::ResizeHorizontal,
                        ResizeEdge::Top | ResizeEdge::Bottom => egui::CursorIcon::ResizeVertical,
                    };
                    ui.output_mut(|o| o.cursor_icon = cursor_icon);
                }

                // 开始调整大小
                if response.drag_started() {
                    self.is_resizing = hovered_edge;
                }

                // 调整矩形大小
                if let Some(edge) = self.is_resizing {
                    match edge {
                        ResizeEdge::Left => {
                            let width = rect.max.x - mouse_pos.x;
                            if width > 0.0 {
                                self.rect_pos.x = mouse_pos.x;
                                self.rect_size.x = width;
                            }
                        }
                        ResizeEdge::Right => {
                            let width = mouse_pos.x - rect.min.x;
                            if width > 0.0 {
                                self.rect_size.x = width;
                            }
                        }
                        ResizeEdge::Top => {
                            let height = rect.max.y - mouse_pos.y;
                            if height > 0.0 {
                                self.rect_pos.y = mouse_pos.y;
                                self.rect_size.y = height;
                            }
                        }
                        ResizeEdge::Bottom => {
                            let height = mouse_pos.y - rect.min.y;
                            if height > 0.0 {
                                self.rect_size.y = height;
                            }
                        }
                    }
                }
            }

            // 结束调整大小
            if response.drag_released() {
                self.is_resizing = None;
            }
        });
    }
}
