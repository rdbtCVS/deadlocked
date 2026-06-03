use std::hash::Hash;

use egui::{
    Align2, CollapsingHeader, Color32, CornerRadius, DragValue, Event, FontId, Frame, Margin, Rect,
    Response, RichText, Sense, Stroke, Ui, Vec2, Widget,
};
use strum::IntoEnumIterator as _;

use crate::{config::KeyMode, cs2::key_codes::KeyCode, ui::color::Colors};

pub fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    card_impl(ui, title, true, add_body);
}

pub fn card(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    card_impl(ui, title, false, add_body);
}

fn card_impl(ui: &mut Ui, title: &str, default_open: bool, add_body: impl FnOnce(&mut Ui)) {
    let outer_width = ui.available_width();
    let inner_width = (outer_width - 28.0).max(0.0);

    Frame::new()
        .fill(Colors::PANEL)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.set_width(inner_width);
            CollapsingHeader::new(header_text(title))
                .default_open(default_open)
                .show(ui, add_body);
        });
}

pub fn category_header(ui: &mut Ui, title: &str) {
    ui.label(header_text(title));
}

pub fn sidebar_item<T: Copy + PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    icon: &str,
    label: &str,
) -> Response {
    let selected = *current == value;
    let height = 34.0;
    let width = ui.available_width();
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    if response.clicked() {
        *current = value;
        response.mark_changed();
    }

    let fill = if selected {
        Colors::PANEL
    } else if response.hovered() {
        Colors::HOVER
    } else {
        Color32::TRANSPARENT
    };

    if fill != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, CornerRadius::same(9), fill);
    }

    if selected {
        let accent_rect = Rect::from_min_max(
            rect.left_top() + Vec2::new(0.0, 7.0),
            rect.left_top() + Vec2::new(3.0, height - 7.0),
        );
        ui.painter().rect_filled(
            accent_rect,
            CornerRadius {
                nw: 2,
                ne: 2,
                sw: 2,
                se: 2,
            },
            ui.visuals().selection.bg_fill,
        );
    }

    let icon_color = if selected {
        ui.visuals().selection.bg_fill
    } else {
        Colors::MUTED
    };
    let label_color = if selected {
        Colors::TEXT
    } else {
        Colors::SUBTEXT
    };
    let icon_pos = rect.left_center() + Vec2::new(24.0, 0.0);
    let label_pos = rect.left_center() + Vec2::new(42.0, 0.0);

    ui.painter().text(
        icon_pos,
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(15.0),
        icon_color,
    );
    ui.painter().text(
        label_pos,
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        label_color,
    );

    response
}

pub fn top_tab<T: Copy + PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    label: &str,
) -> Response {
    let selected = *current == value;
    let color = if selected {
        Colors::TEXT
    } else {
        Colors::SUBTEXT
    };
    let text = if selected {
        RichText::new(label).color(color).strong()
    } else {
        RichText::new(label).color(color)
    };
    let mut response = ui.add(egui::Label::new(text).sense(Sense::click()));

    if response.clicked() {
        *current = value;
        response.mark_changed();
    }

    if selected {
        let underline = Rect::from_min_max(
            response.rect.left_bottom() + Vec2::new(0.0, 4.0),
            response.rect.right_bottom() + Vec2::new(0.0, 6.0),
        );
        ui.painter().rect_filled(
            underline,
            CornerRadius::same(1),
            ui.visuals().selection.bg_fill,
        );
    }

    response
}

fn header_text(title: &str) -> RichText {
    RichText::new(title.to_uppercase())
        .heading()
        .color(Colors::SUBTEXT)
        .small()
        .strong()
}

pub fn scroll(ui: &mut Ui, id: &str, add_content: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .id_salt(id)
        .show(ui, add_content);
}

pub fn checkbox(ui: &mut Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label).changed()
}

pub fn checkbox_hover(ui: &mut Ui, label: &str, hover_text: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label)
        .on_hover_text(hover_text)
        .changed()
}

pub fn drag(ui: &mut Ui, label: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(drag);
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub fn combo_box<T: std::fmt::Debug + strum::IntoEnumIterator + PartialEq>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut T,
) -> bool {
    let mut changed = false;
    egui::ComboBox::new(id, label)
        .selected_text(format!("{:?}", *value))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                let text = format!("{:?}", &mode);
                if ui.selectable_value(value, mode, text).clicked() {
                    changed = true;
                }
            }
        });
    changed
}

pub fn color_picker(ui: &mut Ui, label: &str, color: &mut Color32) -> bool {
    let [mut r, mut g, mut b, mut a] = color.to_srgba_unmultiplied();
    let res = ui
        .horizontal(|ui| {
            let (response, painter) =
                ui.allocate_painter(ui.spacing().interact_size, Sense::hover());
            painter.rect_filled(
                response.rect,
                ui.style().visuals.widgets.inactive.corner_radius,
                *color,
            );
            let mut res = ui.add(DragValue::new(&mut r).prefix("r: "));
            res = res.union(ui.add(DragValue::new(&mut g).prefix("g: ")));
            res = res.union(ui.add(DragValue::new(&mut b).prefix("b: ")));
            res = res.union(ui.add(DragValue::new(&mut a).prefix("a: ")));
            ui.label(label);
            res
        })
        .inner;

    let changed = res.changed();
    if changed {
        *color = Color32::from_rgba_premultiplied(r, g, b, a);
    }

    changed
}

pub fn hotkey_row(
    ui: &mut Ui,
    id_base: &str,
    name: &str,
    key: &mut Option<KeyCode>,
    mode: &mut KeyMode,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui.add(KeybindOpt::new(key, id_base)).changed();
        egui::ComboBox::new(format!("{id_base}_mode"), "")
            .width(72.0)
            .selected_text(format!("{mode:?}"))
            .show_ui(ui, |ui| {
                for m in KeyMode::iter() {
                    changed |= ui.selectable_value(mode, m, format!("{m:?}")).clicked();
                }
            });
        ui.label(name);
    });
    changed
}

pub fn keybind(ui: &mut Ui, id_base: &str, name: &str, key: &mut KeyCode) -> bool {
    let mut key_opt = (*key != KeyCode::None).then_some(*key);
    let changed = ui
        .horizontal(|ui| {
            let changed = ui.add(KeybindOpt::new(&mut key_opt, id_base)).changed();
            ui.label(name);
            changed
        })
        .inner;

    if changed {
        *key = key_opt.unwrap_or(KeyCode::None);
    }

    changed
}

pub struct KeybindOpt<'gui> {
    keycode: &'gui mut Option<KeyCode>,
    id: egui::Id,
}

impl<'gui> KeybindOpt<'gui> {
    pub fn new(keycode: &'gui mut Option<KeyCode>, id: impl Hash) -> Self {
        Self {
            keycode,
            id: egui::Id::new(id),
        }
    }
}

impl<'gui> Widget for KeybindOpt<'gui> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let listening_id = ui.make_persistent_id(self.id);

        let mut listening = {
            let ctx = ui.ctx();
            ctx.memory(|mem| mem.data.get_temp::<bool>(listening_id).unwrap_or(false))
        };

        let text = if listening {
            "...".to_string()
        } else {
            self.keycode
                .map(|k| format!("{k:?}"))
                .unwrap_or_else(|| "\u{2014}".to_string())
        };

        let mut response = ui.button(text);

        if response.clicked() {
            listening = !listening;
        }

        if response.secondary_clicked() {
            listening = false;
        }

        if listening {
            let key_opt = ui.input(|i| {
                for event in &i.events {
                    if let Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if *key == egui::Key::Backspace {
                            return Some(None);
                        }
                        if *key == egui::Key::F35 {
                            return KeyCode::from_egui_modifiers(*modifiers).map(Some);
                        } else if let Some(code) = KeyCode::from_egui(*key) {
                            return Some(Some(code));
                        }
                    }

                    if let Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } = event
                    {
                        return Some(Some(KeyCode::from_egui_mouse(*button)));
                    }
                }
                None
            });

            if let Some(assignment) = key_opt {
                if assignment.map_or(true, |k| k != KeyCode::Escape) {
                    *self.keycode = assignment;
                    response.mark_changed();
                }
                listening = false;
            }
        }

        let ctx = ui.ctx();
        ctx.memory_mut(|mem| mem.data.insert_temp(listening_id, listening));

        response
    }
}
