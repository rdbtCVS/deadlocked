use std::cell::RefCell;

use egui::emath::GuiRounding as _;
use egui::{
    pos2, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Id, Pos2, Rect, Sense, Stroke,
    StrokeKind, Vec2,
};

use crate::{
    config::{BoxFill, BoxMode, DrawMode, EspElementLayout, EspZone},
    cs2::{
        bones::Bones,
        entity::weapon::Weapon,
    },
    ui::{
        app::App,
        esp_hud::{
            esp_hud_build_cmds, esp_union_interactive_rects, paint_cmds, preview_snap_band_rect,
            snap_esp_zone_preview, EspBoxGeom, EspHudInteractive, EspHudSample,
            ESP_INTERACTIVE_ALL,
        },
    },
};

const MIN_DRAG_PICK: f32 = 18.0;

fn preview_health_color(health: i32, alpha: u8) -> egui::Color32 {
    let health = health.clamp(0, 100);

    let (r, g) = if health <= 50 {
        let factor = health as f32 / 50.0;
        (255, (255.0 * factor) as u8)
    } else {
        let factor = 1.0 - (health - 50) as f32 / 50.0;
        ((255.0 * factor) as u8, 255)
    };

    egui::Color32::from_rgba_unmultiplied(r, g, 0, alpha)
}

fn preview_alpha(color: egui::Color32, alpha: f32) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn preview_lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

fn preview_box_stroke_rgb(player: &crate::config::PlayerConfig, health: i32) -> egui::Color32 {
    match player.draw_box {
        DrawMode::None => egui::Color32::TRANSPARENT,
        DrawMode::Health => preview_health_color(health, player.box_visible_color.a()),
        DrawMode::Color => player.box_visible_color,
    }
}

fn preview_skeleton_stroke_rgb(
    player: &crate::config::PlayerConfig,
    health: i32,
) -> Option<egui::Color32> {
    match player.draw_skeleton {
        DrawMode::None => None,
        DrawMode::Health => Some(preview_health_color(health, player.skeleton_color.a())),
        DrawMode::Color => Some(player.skeleton_color),
    }
}

fn preview_bone_pos(bone: Bones, geo: &EspBoxGeom) -> Pos2 {
    let tl = geo.tl;
    let tr = geo.tr;
    let bl = geo.bl;
    let w = tr.x - tl.x;
    let h = bl.y - tl.y;
    let cx = (tl.x + tr.x) * 0.5;
    let y = |t: f32| tl.y + t * h;
    match bone {
        Bones::Head => pos2(cx, y(0.06)),
        Bones::Neck => pos2(cx, y(0.12)),
        Bones::Spine4 => pos2(cx, y(0.20)),
        Bones::Spine3 => pos2(cx, y(0.28)),
        Bones::Spine2 => pos2(cx, y(0.36)),
        Bones::Spine1 => pos2(cx, y(0.44)),
        Bones::Hip => pos2(cx, y(0.93)),
        Bones::LeftShoulder => pos2(cx - w * 0.19, y(0.17)),
        Bones::RightShoulder => pos2(cx + w * 0.19, y(0.17)),
        Bones::LeftElbow => pos2(cx - w * 0.30, y(0.30)),
        Bones::RightElbow => pos2(cx + w * 0.30, y(0.30)),
        Bones::LeftHand => pos2(cx - w * 0.34, y(0.42)),
        Bones::RightHand => pos2(cx + w * 0.34, y(0.42)),
        Bones::LeftHip => pos2(cx - w * 0.11, y(0.88)),
        Bones::RightHip => pos2(cx + w * 0.11, y(0.88)),
        Bones::LeftKnee => pos2(cx - w * 0.10, y(0.62)),
        Bones::RightKnee => pos2(cx + w * 0.10, y(0.62)),
        Bones::LeftFoot => pos2(cx - w * 0.11, y(0.98)),
        Bones::RightFoot => pos2(cx + w * 0.11, y(0.98)),
    }
}

fn paint_preview_box_fill(
    painter: &egui::Painter,
    rect: Rect,
    fill: BoxFill,
    fill_alpha: f32,
    stroke_rgb: egui::Color32,
) {
    match fill {
        BoxFill::None => {}
        BoxFill::Solid => {
            let fill_c = preview_alpha(stroke_rgb, fill_alpha);
            painter.rect_filled(rect, CornerRadius::ZERO, fill_c);
        }
        BoxFill::Gradient => {
            let darker = egui::Color32::from_rgba_unmultiplied(
                (stroke_rgb.r() as f32 * 0.35) as u8,
                (stroke_rgb.g() as f32 * 0.35) as u8,
                (stroke_rgb.b() as f32 * 0.35) as u8,
                255,
            );
            let top_c = preview_alpha(stroke_rgb, fill_alpha);
            let bot_c = preview_alpha(darker, fill_alpha);
            let strips: usize = 14;
            let slice_h = rect.height() / strips as f32;
            let denom = (strips - 1).max(1) as f32;
            for i in 0..strips {
                let t = i as f32 / denom;
                let y0 = rect.top() + slice_h * i as f32;
                let y1 = (y0 + slice_h).min(rect.bottom());
                let slice = Rect::from_min_max(pos2(rect.left(), y0), pos2(rect.right(), y1));
                painter.rect_filled(slice, CornerRadius::ZERO, preview_lerp_color(top_c, bot_c, t));
            }
        }
    }
}

fn paint_preview_3d_box(
    painter: &egui::Painter,
    front: Rect,
    stroke: Stroke,
    fill: BoxFill,
    fill_alpha: f32,
    base_color: egui::Color32,
) {
    let skew = Vec2::new(front.width() * 0.18, -front.height() * 0.12);
    let back = front.translate(skew);

    if fill != BoxFill::None {
        paint_preview_box_fill(painter, front, fill, fill_alpha, base_color);
    }

    painter.rect_stroke(
        front,
        CornerRadius::ZERO,
        stroke,
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        back,
        CornerRadius::ZERO,
        stroke,
        StrokeKind::Inside,
    );

    painter.line_segment([front.left_top(), back.left_top()], stroke);
    painter.line_segment([front.right_top(), back.right_top()], stroke);
    painter.line_segment([front.left_bottom(), back.left_bottom()], stroke);
    painter.line_segment([front.right_bottom(), back.right_bottom()], stroke);
}

fn paint_preview_player_box(
    painter: &egui::Painter,
    player: &crate::config::PlayerConfig,
    hud: &crate::config::HudConfig,
    geo: &EspBoxGeom,
    preview_scale: f32,
    sample_health: i32,
) {
    if player.draw_box == DrawMode::None {
        return;
    }

    let line_width = (hud.line_width * preview_scale).max(1.0);
    let rgb = preview_box_stroke_rgb(player, sample_health);
    let stroke = Stroke::new(line_width, rgb);

    let tl = geo.tl;
    let tr = geo.tr;
    let bl = geo.bl;
    let br = geo.br;
    let ew = geo.ew;
    let qw = ew * 2.0;
    let rect_2d = Rect::from_min_max(tl, br);
    let fill_alpha = player.box_fill_alpha.clamp(0.0, 1.0);

    match player.box_mode {
        BoxMode::ThreeD => {
            paint_preview_3d_box(
                painter,
                rect_2d,
                stroke,
                player.box_fill,
                fill_alpha,
                rgb,
            );
        }
        BoxMode::Full => {
            if player.box_fill != BoxFill::None {
                paint_preview_box_fill(painter, rect_2d, player.box_fill, fill_alpha, rgb);
            }
            painter.rect(
                rect_2d,
                0,
                egui::Color32::TRANSPARENT,
                stroke,
                StrokeKind::Middle,
            );
        }
        BoxMode::Gap => {
            painter.line(
                vec![pos2(tl.x + ew, tl.y), tl, pos2(tl.x, tl.y + qw)],
                stroke,
            );
            painter.line(
                vec![pos2(tr.x - ew, tl.y), tr, pos2(tr.x, tr.y + qw)],
                stroke,
            );
            painter.line(
                vec![pos2(bl.x + ew, bl.y), bl, pos2(bl.x, bl.y - qw)],
                stroke,
            );
            painter.line(
                vec![pos2(br.x - ew, bl.y), br, pos2(br.x, br.y - qw)],
                stroke,
            );
        }
    }
}

fn paint_preview_skeleton(
    painter: &egui::Painter,
    player: &crate::config::PlayerConfig,
    hud: &crate::config::HudConfig,
    geo: &EspBoxGeom,
    preview_scale: f32,
    sample_health: i32,
) {
    let Some(rgb) = preview_skeleton_stroke_rgb(player, sample_health) else {
        return;
    };
    let line_width = (hud.line_width * preview_scale).max(1.0);
    let stroke = Stroke::new(line_width, rgb);

    for (a, b) in Bones::CONNECTIONS {
        let pa = preview_bone_pos(a, geo);
        let pb = preview_bone_pos(b, geo);
        painter.line_segment([pa, pb], stroke);
    }

    if !player.head_circle {
        return;
    }
    let neck = preview_bone_pos(Bones::Neck, geo);
    let spine3 = preview_bone_pos(Bones::Spine3, geo);
    let height = spine3.y - neck.y;
    let pos = pos2(
        neck.x - (spine3.x - neck.x) * 0.5,
        neck.y - height * 0.5,
    );
    painter.circle_stroke(pos, height.abs() * 0.5, stroke);
}

/// Thin vertical/horizontal HUD bars stay easy to grab without shifting the centroid much.
fn expand_pick_rect(r: Rect, min_w: f32, min_h: f32) -> Rect {
    let mut out = r;
    if out.width() < min_w {
        let ex = min_w - out.width();
        out.min.x -= ex * 0.5;
        out.max.x += ex * 0.5;
    }
    if out.height() < min_h {
        let ey = min_h - out.height();
        out.min.y -= ey * 0.5;
        out.max.y += ey * 0.5;
    }
    out
}

/// Clip interactive rects to the preview canvas only — painting uses `painter_at(canvas_rect)` so
/// targets must not extend past it or widgets and ink diverge.
fn drag_rect_for_hit(hit: Rect, canvas_rect: Rect) -> Option<Rect> {
    let ir_raw = hit.intersect(canvas_rect);
    if ir_raw.width() <= 1.0 || ir_raw.height() <= 1.0 {
        return None;
    }
    let ir = expand_pick_rect(ir_raw, MIN_DRAG_PICK, MIN_DRAG_PICK).intersect(canvas_rect);
    if ir.width() <= 1.0 || ir.height() <= 1.0 {
        None
    } else {
        Some(ir.round_ui())
    }
}

/// Ray–AABB overlap interval for `origin + dir * t`.
fn ray_axis_aligned_rect_interval(origin: Pos2, dir: Vec2, rect: Rect) -> Option<(f32, f32)> {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..2 {
        let o = if axis == 0 { origin.x } else { origin.y };
        let d = if axis == 0 { dir.x } else { dir.y };
        let mn = if axis == 0 { rect.left() } else { rect.top() };
        let mx = if axis == 0 {
            rect.right()
        } else {
            rect.bottom()
        };

        if d.abs() < 1e-9 {
            if o < mn - 1e-4 || o > mx + 1e-4 {
                return None;
            }
            continue;
        }
        let t0 = (mn - o) / d;
        let t1 = (mx - o) / d;
        let (lo, hi) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
        t_min = t_min.max(lo);
        t_max = t_max.min(hi);
    }

    if t_min <= t_max {
        Some((t_min, t_max))
    } else {
        None
    }
}

/// Snap ring: single neutral fill, hole punched with canvas bg, plus 45° bisectors from box corners.
/// Uses [`preview_snap_band_rect`] so geometry matches [`snap_esp_zone_preview`].
fn paint_snap_ring_diagram(
    painter: &egui::Painter,
    canvas_rect: Rect,
    box_rect: Rect,
    slab: f32,
    canvas_bg: Color32,
) {
    let band = preview_snap_band_rect(box_rect, slab, canvas_rect);
    if band.width() < 12.0 || band.height() < 12.0 {
        return;
    }

    let ring_tint = Color32::from_rgba_unmultiplied(55, 58, 68, 110);
    painter.rect_filled(band, CornerRadius::ZERO, ring_tint);
    painter.rect_filled(box_rect, CornerRadius::ZERO, canvas_bg);

    let bisector = Stroke::new(1.35, Color32::from_rgba_unmultiplied(220, 225, 240, 190));

    let corners = [
        (box_rect.left_top(), Vec2::new(-1.0, -1.0)),
        (box_rect.right_top(), Vec2::new(1.0, -1.0)),
        (box_rect.left_bottom(), Vec2::new(-1.0, 1.0)),
        (box_rect.right_bottom(), Vec2::new(1.0, 1.0)),
    ];

    for (corner, d_raw) in corners {
        let dir = d_raw.normalized();
        if let Some((t0, t1)) = ray_axis_aligned_rect_interval(corner, dir, band) {
            let eps = 3.0_f32;
            let start = corner + dir * (t0 + eps).max(eps);
            let end = corner + dir * (t1 - eps);
            if (end - start).length() > 6.0 {
                painter.line_segment([start, end], bisector);
            }
        }
    }

    painter.rect_stroke(
        band,
        CornerRadius::ZERO,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 72)),
        StrokeKind::Inside,
    );

    let label_font = FontId::proportional(13.0);
    let label_color = Color32::from_rgba_unmultiplied(245, 248, 255, 245);
    // Place labels near the *outer* band edge, away from corner bisectors, so the text sits in
    // the “safe” part of each Voronoi cell (same classification users get if they click there).
    let labels = [
        (
            "Top",
            pos2(
                box_rect.center().x,
                band.top() + (box_rect.top() - band.top()) * 0.22,
            ),
        ),
        (
            "Bottom",
            pos2(
                box_rect.center().x,
                box_rect.bottom() + (band.bottom() - box_rect.bottom()) * 0.78,
            ),
        ),
        (
            "Left",
            pos2(
                band.left() + (box_rect.left() - band.left()) * 0.22,
                box_rect.center().y,
            ),
        ),
        (
            "Right",
            pos2(
                box_rect.right() + (band.right() - box_rect.right()) * 0.78,
                box_rect.center().y + box_rect.height() * 0.12,
            ),
        ),
    ];
    for (text, p) in labels {
        painter.text(
            p,
            Align2::CENTER_CENTER,
            text,
            label_font.clone(),
            label_color,
        );
    }
}

impl App {
    pub(super) fn esp_layout_preview(&mut self, ui: &mut egui::Ui) {
        // Wide snap band so Top/Left/Right/Bottom drops feel roomy; canvas grows with pad.
        let snap_pad = 96.0_f32;
        let box_w = 100.0_f32;
        let box_h = 218.0_f32;
        let canvas_w = box_w + snap_pad * 2.0;
        let canvas_h = box_h + snap_pad * 2.0;

        // (player box rect, slab slider value, preview canvas rect) — must match painted ring.
        let snap_store: RefCell<Option<(Rect, f32, Rect)>> = RefCell::new(None);

        let frame = Frame::default()
            .fill(egui::Color32::TRANSPARENT)
            .inner_margin(egui::Margin::same(4));

        let (preview_inner, preview_payload) =
            ui.dnd_drop_zone::<EspHudInteractive, _>(frame, |ui_drop| {
                ui_drop.set_min_height(canvas_h);
                ui_drop.set_width(canvas_w);

                let (canvas_rect, _) =
                    ui_drop.allocate_exact_size(Vec2::new(canvas_w, canvas_h), Sense::hover());

                let canvas_bg = ui_drop.visuals().extreme_bg_color;
                ui_drop
                    .painter()
                    .rect_filled(canvas_rect, CornerRadius::same(6), canvas_bg);

                let center = canvas_rect.center();
                let tl = pos2(center.x - box_w * 0.5, center.y - box_h * 0.5);
                let tr = pos2(center.x + box_w * 0.5, tl.y);
                let bl = pos2(tl.x, center.y + box_h * 0.5);
                let br = pos2(tr.x, bl.y);

                let half_width = box_w * 0.5;
                let qw = half_width - 2.0;
                let ew = qw / 2.0;

                let painter = ui_drop.painter_at(canvas_rect);

                let box_rect_screen = Rect::from_min_max(tl, br);
                let slab_clamped = snap_pad;

                paint_snap_ring_diagram(
                    &painter,
                    canvas_rect,
                    box_rect_screen,
                    slab_clamped,
                    canvas_bg,
                );

                let preview_hud_scale = (canvas_rect.height() / 268.0).clamp(1.28, 1.82);

                let geo = EspBoxGeom { tl, tr, bl, br, ew };

                let sample = EspHudSample {
                    health: 72,
                    armor: 45,
                    name: "Preview",
                    weapon_icon: Weapon::Ak47.to_icon(),
                    ammo: (24, 90),
                    has_defuser: true,
                    has_helmet: true,
                    has_bomb: false,
                    distance_m: 42.0,
                    is_scoped: true,
                    is_flashed: false,
                };

                paint_preview_player_box(
                    &painter,
                    &self.config.player,
                    &self.config.hud,
                    &geo,
                    preview_hud_scale,
                    sample.health,
                );

                let cmds = esp_hud_build_cmds(
                    &self.config.player,
                    &self.config.hud,
                    &geo,
                    &sample,
                    1.0,
                    preview_hud_scale,
                );
                paint_cmds(&painter, &self.config.hud, &cmds);

                paint_preview_skeleton(
                    &painter,
                    &self.config.player,
                    &self.config.hud,
                    &geo,
                    preview_hud_scale,
                    sample.health,
                );

                let hits = esp_union_interactive_rects(&painter, &self.config.hud, &cmds);

                *snap_store.borrow_mut() = Some((box_rect_screen, slab_clamped, canvas_rect));

                // Register drag targets at exact rects — avoids nested `scope`/`dnd_drag_source`
                // placement differing from painted HUD geometry.
                for (feat, hit) in hits {
                    let Some(ir) = drag_rect_for_hit(hit, canvas_rect) else {
                        continue;
                    };
                    let response = ui_drop
                        .interact(ir, Id::new(("esp_hud_drag", feat)), Sense::drag())
                        .on_hover_cursor(CursorIcon::Grab);
                    response.dnd_set_drag_payload(feat);
                }
            });

        ui.add_space(4.0);

        let off_frame = Frame::default()
            .fill(egui::Color32::TRANSPARENT)
            .inner_margin(egui::Margin::same(4));
        let (_, off_payload) = ui.dnd_drop_zone::<EspHudInteractive, _>(off_frame, |ui_off| {
            ui_off.set_min_width(canvas_w);
            ui_off.set_min_height(52.0);
            ui_off.label(egui::RichText::new("Off").small().weak());
            ui_off.horizontal_wrapped(|row| {
                row.spacing_mut().item_spacing.x = 4.0;
                let ly_ref = &self.config.player.esp_layout;
                let mut any_disabled = false;
                for &feat in &ESP_INTERACTIVE_ALL {
                    if feat.zone_in(ly_ref) != EspZone::Disabled {
                        continue;
                    }
                    any_disabled = true;
                    row.dnd_drag_source(Id::new(("esp_disabled", feat)), feat, |chip| {
                        let _ = chip.small_button(feat.short_label());
                    });
                }
                if !any_disabled {
                    row.label(
                        egui::RichText::new("Drop a preview item here to turn it off")
                            .small()
                            .weak(),
                    );
                }
            });
        });

        if let Some(pl) = off_payload {
            let feat = *pl;
            if feat.zone_in(&self.config.player.esp_layout) != EspZone::Disabled {
                *feat.zone_mut(&mut self.config.player.esp_layout) = EspZone::Disabled;
                self.send_config();
            }
        }

        if let Some(pl) = preview_payload {
            let feat = *pl;
            if let Some((box_r, slab, canvas_r)) = *snap_store.borrow() {
                // On release, `Response::interact_pointer_pos` is often **not** the cursor
                // (it follows press/drag state). Use live hover / interact pos first so band drops
                // match what you see: label “Top/Left/…” and `snap_esp_zone_preview` use the same point.
                let ptr = ui
                    .ctx()
                    .pointer_hover_pos()
                    .or_else(|| ui.ctx().pointer_interact_pos())
                    .or_else(|| ui.ctx().pointer_latest_pos())
                    .or_else(|| preview_inner.response.interact_pointer_pos());
                if let Some(ptr) = ptr {
                    let zone = snap_esp_zone_preview(ptr, box_r, slab, canvas_r);
                    if zone != EspZone::Disabled
                        && feat.zone_in(&self.config.player.esp_layout) != zone
                    {
                        *feat.zone_mut(&mut self.config.player.esp_layout) = zone;
                        self.send_config();
                    }
                }
            }
        }

        if ui
            .button("Reset HUD layout")
            .on_hover_text("Restore default edge placement")
            .clicked()
        {
            self.config.player.esp_layout = EspElementLayout::default();
            self.send_config();
        }
    }
}
