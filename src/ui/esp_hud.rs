//! Shared 2D ESP HUD elements (bars, name, weapon, distance) for overlay and GUI preview.

use std::collections::HashMap;

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2, pos2};

use crate::config::{EspElementLayout, EspZone, PlayerConfig};

const GAP: f32 = 4.0;
const AMMO_GAP: f32 = 3.0;
const OUTLINE_WIDTH: f32 = 1.0;

#[derive(Clone, Copy)]
pub struct EspBoxGeom {
    pub tl: Pos2,
    pub tr: Pos2,
    pub bl: Pos2,
    pub br: Pos2,
    pub ew: f32,
}

pub struct EspHudSample<'a> {
    pub health: i32,
    pub armor: i32,
    pub name: &'a str,
    pub weapon_icon: &'a str,
    pub ammo: (i32, i32),
    pub has_defuser: bool,
    pub has_helmet: bool,
    pub has_bomb: bool,
    pub distance_m: f32,
    pub is_scoped: bool,
    pub is_flashed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EspHudInteractive {
    HealthBar,
    ArmorBar,
    PlayerName,
    Tags,
    WeaponIcon,
    Distance,
    StatusFlags,
}

/// Fixed iteration order for settings UI / resets.
pub const ESP_INTERACTIVE_ALL: [EspHudInteractive; 7] = [
    EspHudInteractive::HealthBar,
    EspHudInteractive::ArmorBar,
    EspHudInteractive::PlayerName,
    EspHudInteractive::Tags,
    EspHudInteractive::WeaponIcon,
    EspHudInteractive::Distance,
    EspHudInteractive::StatusFlags,
];

impl EspHudInteractive {
    pub fn zone_in(self, layout: &EspElementLayout) -> EspZone {
        match self {
            Self::HealthBar => layout.health_bar,
            Self::ArmorBar => layout.armor_bar,
            Self::PlayerName => layout.player_name,
            Self::Tags => layout.tags,
            Self::WeaponIcon => layout.weapon_icon,
            Self::Distance => layout.distance_meters,
            Self::StatusFlags => layout.status_flags,
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::HealthBar => "Health",
            Self::ArmorBar => "Armor",
            Self::PlayerName => "Name",
            Self::Tags => "Tags",
            Self::WeaponIcon => "Weapon",
            Self::Distance => "Distance",
            Self::StatusFlags => "Flags",
        }
    }

    pub fn zone_mut(self, layout: &mut EspElementLayout) -> &mut EspZone {
        match self {
            Self::HealthBar => &mut layout.health_bar,
            Self::ArmorBar => &mut layout.armor_bar,
            Self::PlayerName => &mut layout.player_name,
            Self::Tags => &mut layout.tags,
            Self::WeaponIcon => &mut layout.weapon_icon,
            Self::Distance => &mut layout.distance_meters,
            Self::StatusFlags => &mut layout.status_flags,
        }
    }
}

#[derive(Clone, Debug)]
pub enum EspHudCmd {
    LineSegment {
        feature: EspHudInteractive,
        points: Vec<Pos2>,
        stroke: Stroke,
    },
    PropText {
        feature: EspHudInteractive,
        pos: Pos2,
        align: Align2,
        text: String,
        font_size: f32,
        color: Option<Color32>,
    },
    MonoGlyph {
        feature: EspHudInteractive,
        pos: Pos2,
        align: Align2,
        text: String,
        font_px: f32,
        color: Color32,
    },
}

fn esp_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn health_color(health: i32, alpha: u8) -> Color32 {
    let health = health.clamp(0, 100);

    let (r, g) = if health <= 50 {
        let factor = health as f32 / 50.0;
        (255, (255.0 * factor) as u8)
    } else {
        let factor = 1.0 - (health - 50) as f32 / 50.0;
        ((255.0 * factor) as u8, 255)
    };

    Color32::from_rgba_unmultiplied(r, g, 0, alpha)
}

fn outline(pos: Pos2, color: Color32) -> [(Pos2, Color32); 5] {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, color.a());
    [
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y - OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x - OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            outline_color,
        ),
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            outline_color,
        ),
        (pos, color),
    ]
}

fn text_sized_hud_paint(
    painter: &Painter,
    hud: &crate::config::HudConfig,
    text: impl AsRef<str>,
    position: Pos2,
    align: Align2,
    color: Option<Color32>,
    font_size: f32,
) {
    let font = FontId::proportional(font_size);
    let color = match color {
        Some(c) => c,
        None => hud.text_color,
    };
    if hud.text_outline {
        for (pos, col) in outline(position, color) {
            painter.text(pos, align, text.as_ref(), font.clone(), col);
        }
    } else {
        painter.text(position, align, text.as_ref(), font, color);
    }
}

fn collect_features(layout: &EspElementLayout) -> HashMap<EspZone, Vec<EspHudInteractive>> {
    let mut map: HashMap<EspZone, Vec<EspHudInteractive>> = HashMap::new();
    for f in [
        EspHudInteractive::HealthBar,
        EspHudInteractive::ArmorBar,
        EspHudInteractive::PlayerName,
        EspHudInteractive::Tags,
        EspHudInteractive::WeaponIcon,
        EspHudInteractive::Distance,
        EspHudInteractive::StatusFlags,
    ] {
        let z = f.zone_in(layout);
        if z == EspZone::Disabled {
            continue;
        }
        map.entry(z).or_default().push(f);
    }
    for v in map.values_mut() {
        v.sort_unstable();
    }
    map
}

fn vertical_bar_outer_x(anchor_x: f32, side_left: bool, slot: usize, line_width: f32) -> f32 {
    let step = line_width * 2.0;
    if side_left {
        anchor_x - step * (slot as f32 + 1.0)
    } else {
        anchor_x + step * (slot as f32 + 1.0)
    }
}

fn line_hit_rect(points: &[Pos2], lw: f32) -> Rect {
    if points.len() < 2 {
        return Rect::NOTHING;
    }
    let mut r = Rect::from_two_pos(points[0], points[1]);
    for p in &points[2..] {
        r = r.union(Rect::from_center_size(*p, Vec2::splat(1.0)));
    }
    let pad = (lw.mul_add(3.5, 8.0)).clamp(8.0, 20.0);
    r.expand(pad)
}

fn esp_hud_cmd_hit_rect(
    cmd: &EspHudCmd,
    painter: &Painter,
    hud: &crate::config::HudConfig,
) -> Option<(EspHudInteractive, Rect)> {
    match cmd {
        EspHudCmd::LineSegment {
            feature,
            points,
            stroke,
        } => Some((*feature, line_hit_rect(points, stroke.width))),
        EspHudCmd::PropText {
            feature,
            pos,
            align,
            text,
            font_size,
            color,
        } => {
            let galley_color = color.unwrap_or(hud.text_color);
            let galley = painter.layout_no_wrap(
                text.clone(),
                FontId::proportional(*font_size),
                galley_color,
            );
            let mut rect = align.anchor_size(*pos, galley.size());
            if hud.text_outline {
                rect = rect.expand(OUTLINE_WIDTH + 2.0);
            }
            Some((*feature, rect))
        }
        EspHudCmd::MonoGlyph {
            feature,
            pos,
            align,
            text,
            font_px,
            color,
            ..
        } => {
            let galley = painter.layout_no_wrap(text.clone(), FontId::monospace(*font_px), *color);
            Some((*feature, align.anchor_size(*pos, galley.size())))
        }
    }
}

/// Bounding boxes around each draggable ESP piece (merged per feature), using the same font
/// layouts as [`paint_cmds`] so hit targets match pixels.
pub fn esp_union_interactive_rects(
    painter: &Painter,
    hud: &crate::config::HudConfig,
    cmds: &[EspHudCmd],
) -> Vec<(EspHudInteractive, Rect)> {
    let mut map: HashMap<EspHudInteractive, Rect> = HashMap::new();
    for cmd in cmds {
        let Some((f, mut r)) = esp_hud_cmd_hit_rect(cmd, painter, hud) else {
            continue;
        };
        r = r.expand(2.5);
        map.entry(f)
            .and_modify(|existing| *existing = existing.union(r))
            .or_insert(r);
    }
    map.into_iter().collect()
}

pub(crate) fn paint_cmds(painter: &Painter, hud: &crate::config::HudConfig, cmds: &[EspHudCmd]) {
    for cmd in cmds {
        match cmd {
            EspHudCmd::LineSegment { points, stroke, .. } => {
                painter.line(points.clone(), *stroke);
            }
            EspHudCmd::PropText {
                pos,
                align,
                text,
                font_size,
                color,
                ..
            } => {
                text_sized_hud_paint(painter, hud, text, *pos, *align, *color, *font_size);
            }
            EspHudCmd::MonoGlyph {
                pos,
                align,
                text,
                font_px,
                color,
                ..
            } => {
                painter.text(
                    *pos,
                    *align,
                    text.as_str(),
                    FontId::monospace(*font_px),
                    *color,
                );
            }
        }
    }
}

fn visible_horizontal_bar_count(items: &[EspHudInteractive], sample: &EspHudSample<'_>) -> usize {
    items
        .iter()
        .filter(|&&f| {
            matches!(f, EspHudInteractive::HealthBar)
                || (matches!(f, EspHudInteractive::ArmorBar) && sample.armor > 0)
        })
        .count()
}

pub fn esp_hud_build_cmds(
    player: &PlayerConfig,
    hud: &crate::config::HudConfig,
    geo: &EspBoxGeom,
    sample: &EspHudSample<'_>,
    alpha: f32,
    esp_scale: f32,
) -> Vec<EspHudCmd> {
    let line_width = hud.line_width * esp_scale;
    let font_size = hud.font_size * esp_scale;
    let icon_line = hud.icon_size * esp_scale;

    let text_color = esp_alpha(hud.text_color, alpha);
    let health_stroke_rgb = health_color(sample.health, 255);
    let health_stroke = esp_alpha(health_stroke_rgb, alpha);

    let delta_y = geo.bl.y - geo.tl.y;
    let layout = &player.esp_layout;
    let by_zone = collect_features(layout);

    let mut cmds = Vec::new();

    for (side_left, zone) in [(true, EspZone::Left), (false, EspZone::Right)] {
        let Some(items) = by_zone.get(&zone) else {
            continue;
        };
        let mut slot = 0usize;
        for &f in items {
            match f {
                EspHudInteractive::HealthBar => {
                    let anchor = if side_left { geo.bl.x } else { geo.br.x };
                    let x = vertical_bar_outer_x(anchor, side_left, slot, line_width);
                    slot += 1;
                    let y1 = geo.bl.y - (delta_y * sample.health as f32 / 100.0);
                    cmds.push(EspHudCmd::LineSegment {
                        feature: f,
                        points: vec![pos2(x, geo.bl.y), pos2(x, y1)],
                        stroke: Stroke::new(line_width, health_stroke),
                    });
                }
                EspHudInteractive::ArmorBar if sample.armor > 0 => {
                    let anchor = if side_left { geo.bl.x } else { geo.br.x };
                    let x = vertical_bar_outer_x(anchor, side_left, slot, line_width);
                    slot += 1;
                    let y1 = geo.bl.y - (delta_y * sample.armor as f32 / 100.0);
                    cmds.push(EspHudCmd::LineSegment {
                        feature: f,
                        points: vec![pos2(x, geo.bl.y), pos2(x, y1)],
                        stroke: Stroke::new(line_width, esp_alpha(Color32::BLUE, alpha)),
                    });
                }
                _ => {}
            }
        }
    }

    let bar_span = geo.tr.x - geo.tl.x;
    for (zone, from_top) in [(EspZone::Top, true), (EspZone::Bottom, false)] {
        let Some(items) = by_zone.get(&zone) else {
            continue;
        };
        let mut bar_i = 0usize;
        for &f in items {
            let y = if from_top {
                geo.tl.y - GAP - bar_i as f32 * (line_width + 3.0)
            } else {
                geo.bl.y + GAP + bar_i as f32 * (line_width + 3.0)
            };
            match f {
                EspHudInteractive::HealthBar => {
                    let w = bar_span * sample.health as f32 / 100.0;
                    cmds.push(EspHudCmd::LineSegment {
                        feature: f,
                        points: vec![pos2(geo.tl.x, y), pos2(geo.tl.x + w, y)],
                        stroke: Stroke::new(line_width, health_stroke),
                    });
                    bar_i += 1;
                }
                EspHudInteractive::ArmorBar if sample.armor > 0 => {
                    let w = bar_span * sample.armor as f32 / 100.0;
                    cmds.push(EspHudCmd::LineSegment {
                        feature: f,
                        points: vec![pos2(geo.tl.x, y), pos2(geo.tl.x + w, y)],
                        stroke: Stroke::new(line_width, esp_alpha(Color32::BLUE, alpha)),
                    });
                    bar_i += 1;
                }
                _ => {}
            }
        }
    }

    let cx_top = (geo.tl.x + geo.tr.x) * 0.5;
    let cx_bottom = (geo.bl.x + geo.br.x) * 0.5;

    if let Some(items) = by_zone.get(&EspZone::Top) {
        let reserved_bar_rows = visible_horizontal_bar_count(items, sample) as f32;
        let mut y = geo.tl.y - GAP - reserved_bar_rows * (line_width + 3.0);
        for &f in items {
            match f {
                EspHudInteractive::PlayerName => {
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_top, y),
                        align: Align2::CENTER_BOTTOM,
                        text: sample.name.to_string(),
                        font_size,
                        color: Some(text_color),
                    });
                    y -= font_size + 2.0;
                }
                EspHudInteractive::Tags => {
                    if sample.has_defuser {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(cx_top, y),
                            align: Align2::CENTER_BOTTOM,
                            text: "\u{e00f}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y -= font_size;
                    }
                    if sample.has_helmet {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(cx_top, y),
                            align: Align2::CENTER_BOTTOM,
                            text: "\u{e017}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y -= font_size;
                    }
                    if sample.has_bomb {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(cx_top, y),
                            align: Align2::CENTER_BOTTOM,
                            text: "\u{e01e}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y -= font_size;
                    }
                    y -= 2.0;
                }
                EspHudInteractive::WeaponIcon => {
                    cmds.push(EspHudCmd::MonoGlyph {
                        feature: f,
                        pos: pos2(cx_top, y),
                        align: Align2::CENTER_BOTTOM,
                        text: sample.weapon_icon.into(),
                        font_px: icon_line,
                        color: text_color,
                    });
                    y -= icon_line;
                    if sample.ammo.0 >= 0 {
                        cmds.push(EspHudCmd::PropText {
                            feature: EspHudInteractive::WeaponIcon,
                            pos: pos2(cx_top, y),
                            align: Align2::CENTER_BOTTOM,
                            text: format!("{}/{}", sample.ammo.0, sample.ammo.1),
                            font_size,
                            color: Some(text_color),
                        });
                        y -= font_size + AMMO_GAP;
                    }
                    y -= GAP;
                }
                EspHudInteractive::Distance => {
                    let dist_font = font_size * 0.95;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_top, y),
                        align: Align2::CENTER_BOTTOM,
                        text: format!("{:.0} m", sample.distance_m),
                        font_size: dist_font,
                        color: Some(text_color),
                    });
                    y -= dist_font + 2.0;
                }
                EspHudInteractive::StatusFlags if sample.is_scoped || sample.is_flashed => {
                    let mut flags = String::new();
                    if sample.is_scoped {
                        flags.push_str("[SCOPE]");
                    }
                    if sample.is_flashed {
                        if !flags.is_empty() {
                            flags.push(' ');
                        }
                        flags.push_str("[FLASH]");
                    }
                    let flag_color = if sample.is_flashed {
                        Color32::YELLOW
                    } else {
                        text_color
                    };
                    let flags_font = font_size * 0.85;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_top, y),
                        align: Align2::CENTER_BOTTOM,
                        text: flags,
                        font_size: flags_font,
                        color: Some(esp_alpha(flag_color, alpha)),
                    });
                    y -= flags_font + 2.0;
                }
                _ => {}
            }
        }
    }

    if let Some(items) = by_zone.get(&EspZone::Right) {
        let mut y_off = 0.0f32;
        let side_x = geo.tr.x + geo.ew;
        for &f in items {
            match f {
                EspHudInteractive::PlayerName => {
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tr.y + y_off),
                        align: Align2::LEFT_TOP,
                        text: sample.name.into(),
                        font_size,
                        color: Some(text_color),
                    });
                    y_off += font_size;
                }
                EspHudInteractive::Tags => {
                    if sample.has_defuser {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tr.y + y_off),
                            align: Align2::LEFT_TOP,
                            text: "\u{e00f}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                    if sample.has_helmet {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tr.y + y_off),
                            align: Align2::LEFT_TOP,
                            text: "\u{e017}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                    if sample.has_bomb {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tr.y + y_off),
                            align: Align2::LEFT_TOP,
                            text: "\u{e01e}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                }
                EspHudInteractive::WeaponIcon => {
                    cmds.push(EspHudCmd::MonoGlyph {
                        feature: f,
                        pos: pos2(side_x, geo.tr.y + y_off),
                        align: Align2::LEFT_TOP,
                        text: sample.weapon_icon.into(),
                        font_px: icon_line,
                        color: text_color,
                    });
                    y_off += icon_line;
                    if sample.ammo.0 >= 0 {
                        cmds.push(EspHudCmd::PropText {
                            feature: EspHudInteractive::WeaponIcon,
                            pos: pos2(side_x, geo.tr.y + y_off),
                            align: Align2::LEFT_TOP,
                            text: format!("{}/{}", sample.ammo.0, sample.ammo.1),
                            font_size,
                            color: Some(text_color),
                        });
                        y_off += font_size + AMMO_GAP;
                    }
                }
                EspHudInteractive::Distance => {
                    let dist_font = font_size * 0.95;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tr.y + y_off),
                        align: Align2::LEFT_TOP,
                        text: format!("{:.0} m", sample.distance_m),
                        font_size: dist_font,
                        color: Some(text_color),
                    });
                    y_off += dist_font + 2.0;
                }
                EspHudInteractive::StatusFlags if sample.is_scoped || sample.is_flashed => {
                    let mut flags = String::new();
                    if sample.is_scoped {
                        flags.push_str("[SCOPE]");
                    }
                    if sample.is_flashed {
                        if !flags.is_empty() {
                            flags.push(' ');
                        }
                        flags.push_str("[FLASH]");
                    }
                    let flag_color = if sample.is_flashed {
                        Color32::YELLOW
                    } else {
                        text_color
                    };
                    let flags_font = font_size * 0.85;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tr.y + y_off),
                        align: Align2::LEFT_TOP,
                        text: flags,
                        font_size: flags_font,
                        color: Some(esp_alpha(flag_color, alpha)),
                    });
                    y_off += flags_font + 2.0;
                }
                _ => {}
            }
        }
    }

    if let Some(items) = by_zone.get(&EspZone::Left) {
        let mut y_off = 0.0f32;
        let side_x = geo.tl.x - geo.ew;
        for &f in items {
            match f {
                EspHudInteractive::PlayerName => {
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tl.y + y_off),
                        align: Align2::RIGHT_TOP,
                        text: sample.name.into(),
                        font_size,
                        color: Some(text_color),
                    });
                    y_off += font_size;
                }
                EspHudInteractive::Tags => {
                    if sample.has_defuser {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tl.y + y_off),
                            align: Align2::RIGHT_TOP,
                            text: "\u{e00f}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                    if sample.has_helmet {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tl.y + y_off),
                            align: Align2::RIGHT_TOP,
                            text: "\u{e017}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                    if sample.has_bomb {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(side_x, geo.tl.y + y_off),
                            align: Align2::RIGHT_TOP,
                            text: "\u{e01e}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        y_off += font_size;
                    }
                }
                EspHudInteractive::WeaponIcon => {
                    cmds.push(EspHudCmd::MonoGlyph {
                        feature: f,
                        pos: pos2(side_x, geo.tl.y + y_off),
                        align: Align2::RIGHT_TOP,
                        text: sample.weapon_icon.into(),
                        font_px: icon_line,
                        color: text_color,
                    });
                    y_off += icon_line;
                    if sample.ammo.0 >= 0 {
                        cmds.push(EspHudCmd::PropText {
                            feature: EspHudInteractive::WeaponIcon,
                            pos: pos2(side_x, geo.tl.y + y_off),
                            align: Align2::RIGHT_TOP,
                            text: format!("{}/{}", sample.ammo.0, sample.ammo.1),
                            font_size,
                            color: Some(text_color),
                        });
                        y_off += font_size + AMMO_GAP;
                    }
                }
                EspHudInteractive::Distance => {
                    let dist_font = font_size * 0.95;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tl.y + y_off),
                        align: Align2::RIGHT_TOP,
                        text: format!("{:.0} m", sample.distance_m),
                        font_size: dist_font,
                        color: Some(text_color),
                    });
                    y_off += dist_font + 2.0;
                }
                EspHudInteractive::StatusFlags if sample.is_scoped || sample.is_flashed => {
                    let mut flags = String::new();
                    if sample.is_scoped {
                        flags.push_str("[SCOPE]");
                    }
                    if sample.is_flashed {
                        if !flags.is_empty() {
                            flags.push(' ');
                        }
                        flags.push_str("[FLASH]");
                    }
                    let flag_color = if sample.is_flashed {
                        Color32::YELLOW
                    } else {
                        text_color
                    };
                    let flags_font = font_size * 0.85;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(side_x, geo.tl.y + y_off),
                        align: Align2::RIGHT_TOP,
                        text: flags,
                        font_size: flags_font,
                        color: Some(esp_alpha(flag_color, alpha)),
                    });
                    y_off += flags_font + 2.0;
                }
                _ => {}
            }
        }
    }

    if let Some(items) = by_zone.get(&EspZone::Bottom) {
        let reserved_bar_rows = visible_horizontal_bar_count(items, sample) as f32;
        let mut y = geo.bl.y + GAP + reserved_bar_rows * (line_width + 3.0);
        for &f in items {
            match f {
                EspHudInteractive::PlayerName => {
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_bottom, y),
                        align: Align2::CENTER_TOP,
                        text: sample.name.into(),
                        font_size,
                        color: Some(text_color),
                    });
                    y += font_size + 2.0;
                }
                EspHudInteractive::Tags => {
                    let n = sample.has_defuser as u32
                        + sample.has_helmet as u32
                        + sample.has_bomb as u32;
                    if n == 0 {
                        continue;
                    }
                    let total_w = font_size * n as f32;
                    let mut x = cx_bottom - total_w * 0.5;
                    if sample.has_defuser {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(x, y),
                            align: Align2::LEFT_TOP,
                            text: "\u{e00f}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        x += font_size;
                    }
                    if sample.has_helmet {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(x, y),
                            align: Align2::LEFT_TOP,
                            text: "\u{e017}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                        x += font_size;
                    }
                    if sample.has_bomb {
                        cmds.push(EspHudCmd::MonoGlyph {
                            feature: f,
                            pos: pos2(x, y),
                            align: Align2::LEFT_TOP,
                            text: "\u{e01e}".into(),
                            font_px: icon_line.min(font_size),
                            color: text_color,
                        });
                    }
                    y += font_size + 2.0;
                }
                EspHudInteractive::WeaponIcon => {
                    cmds.push(EspHudCmd::MonoGlyph {
                        feature: f,
                        pos: pos2(cx_bottom, y),
                        align: Align2::CENTER_TOP,
                        text: sample.weapon_icon.into(),
                        font_px: icon_line,
                        color: text_color,
                    });
                    y += icon_line;
                    if sample.ammo.0 >= 0 {
                        cmds.push(EspHudCmd::PropText {
                            feature: EspHudInteractive::WeaponIcon,
                            pos: pos2(cx_bottom, y),
                            align: Align2::CENTER_TOP,
                            text: format!("{}/{}", sample.ammo.0, sample.ammo.1),
                            font_size,
                            color: Some(text_color),
                        });
                        y += font_size + AMMO_GAP;
                    }
                    y += GAP;
                }
                EspHudInteractive::Distance => {
                    let dist_font = font_size * 0.95;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_bottom, y),
                        align: Align2::CENTER_TOP,
                        text: format!("{:.0} m", sample.distance_m),
                        font_size: dist_font,
                        color: Some(text_color),
                    });
                    y += dist_font + 2.0;
                }
                EspHudInteractive::StatusFlags if sample.is_scoped || sample.is_flashed => {
                    let mut flags = String::new();
                    if sample.is_scoped {
                        flags.push_str("[SCOPE]");
                    }
                    if sample.is_flashed {
                        if !flags.is_empty() {
                            flags.push(' ');
                        }
                        flags.push_str("[FLASH]");
                    }
                    let flag_color = if sample.is_flashed {
                        Color32::YELLOW
                    } else {
                        text_color
                    };
                    let flags_font = font_size * 0.85;
                    cmds.push(EspHudCmd::PropText {
                        feature: f,
                        pos: pos2(cx_bottom, y),
                        align: Align2::CENTER_TOP,
                        text: flags,
                        font_size: flags_font,
                        color: Some(esp_alpha(flag_color, alpha)),
                    });
                    y += flags_font + 2.0;
                }
                _ => {}
            }
        }
    }

    cmds
}

/// Draw configurable ESP HUD (bars / text / weapon row) relative to [`EspBoxGeom`].
pub fn draw_esp_hud_extras(
    painter: &Painter,
    player: &PlayerConfig,
    hud: &crate::config::HudConfig,
    geo: &EspBoxGeom,
    sample: &EspHudSample<'_>,
    alpha: f32,
    esp_scale: f32,
) {
    let cmds = esp_hud_build_cmds(player, hud, geo, sample, alpha, esp_scale);
    paint_cmds(painter, hud, &cmds);
}

/// Outer padding for snap-drop ring ([`snap_esp_zone_preview`]). Kept in sync with GUI preview scaling.
#[inline]
pub(crate) fn snap_band_pad(slab: f32) -> f32 {
    slab.clamp(32.0, 150.0)
}

fn nearest_edge_zone_from_distances(box_rect: Rect, p: Pos2) -> EspZone {
    let top = box_rect.top();
    let bottom = box_rect.bottom();
    let left = box_rect.left();
    let right = box_rect.right();

    // Match the rendered snap guide: cardinal side strips stay cardinal, and only the
    // four outside corner quadrants are split by the 45-degree corner bisectors.
    if p.y < top {
        let dy = top - p.y;
        if p.x < left {
            return if dy >= left - p.x {
                EspZone::Top
            } else {
                EspZone::Left
            };
        }
        if p.x > right {
            return if dy >= p.x - right {
                EspZone::Top
            } else {
                EspZone::Right
            };
        }
        return EspZone::Top;
    }

    if p.y > bottom {
        let dy = p.y - bottom;
        if p.x < left {
            return if dy >= left - p.x {
                EspZone::Bottom
            } else {
                EspZone::Left
            };
        }
        if p.x > right {
            return if dy >= p.x - right {
                EspZone::Bottom
            } else {
                EspZone::Right
            };
        }
        return EspZone::Bottom;
    }

    if p.x < left {
        EspZone::Left
    } else if p.x > right {
        EspZone::Right
    } else {
        EspZone::Disabled
    }
}

/// Same painted ring as the ESP layout preview: `expand(pad) ∩ canvas` (not the full expanded rect).
#[inline]
pub(crate) fn preview_snap_band_rect(box_rect: Rect, slab: f32, canvas_rect: Rect) -> Rect {
    box_rect.expand(snap_band_pad(slab)).intersect(canvas_rect)
}

/// Nearest-edge snap for the **on-screen** preview: ring clipped to `canvas_rect`, matching
/// [`preview_snap_band_rect`] and what is drawn in the ESP layout preview.
pub(crate) fn snap_esp_zone_preview(
    p: Pos2,
    box_rect: Rect,
    slab: f32,
    canvas_rect: Rect,
) -> EspZone {
    if box_rect.contains(p) {
        return EspZone::Disabled;
    }
    let band = preview_snap_band_rect(box_rect, slab, canvas_rect);
    if !band.contains(p) {
        return EspZone::Disabled;
    }
    nearest_edge_zone_from_distances(box_rect, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap_test_rect() -> Rect {
        Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 300.0))
    }

    #[test]
    fn side_strips_do_not_snap_to_top_or_bottom_near_corners() {
        let rect = snap_test_rect();

        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(95.0, 101.0)),
            EspZone::Left
        );
        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(205.0, 299.0)),
            EspZone::Right
        );
    }

    #[test]
    fn corner_quadrants_follow_rendered_diagonal_boundaries() {
        let rect = snap_test_rect();

        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(90.0, 95.0)),
            EspZone::Left
        );
        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(95.0, 90.0)),
            EspZone::Top
        );
        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(210.0, 305.0)),
            EspZone::Right
        );
        assert_eq!(
            nearest_edge_zone_from_distances(rect, pos2(205.0, 310.0)),
            EspZone::Bottom
        );
    }
}
