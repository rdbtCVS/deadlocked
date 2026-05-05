use std::time::{Duration, Instant};

use egui::{Align2, Color32, FontId, Painter, Shape, Stroke, pos2};
use epaint::Mesh;
use glam::{Vec3, vec3};

use crate::{
    config::{BoxFill, BoxMode, DrawMode},
    constants::cs2::HAMMER_UNITS_PER_METER,
    cs2::bones::Bones,
    data::{Data, PlayerData, SoundType},
    math::world_to_screen,
    ui::app::App,
};

impl App {
    pub fn draw_player(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        if self.config.player.visible_only && !player.visible {
            return;
        }

        let sound = self.player_sounds.get(&player.steam_id);
        let sound_alpha = if self.config.player.sound.enabled {
            self.player_sound_alpha(player, sound, data)
        } else {
            None
        };

        self.player_box(painter, player, data, sound_alpha);
        self.skeleton(painter, player, data, sound_alpha);
    }

    fn player_sound_alpha(
        &self,
        player: &PlayerData,
        sound: Option<&(Instant, SoundType)>,
        data: &Data,
    ) -> Option<f32> {
        if self.config.player.sound.show_visible && player.visible {
            return Some(1.0);
        }

        let Some((time, sound)) = sound else {
            return Some(0.0);
        };

        let local_player = &data.local_player;
        let max_distance = match sound {
            SoundType::Footstep => self.config.player.sound.footstep_diameter,
            SoundType::Gunshot => self.config.player.sound.gunshot_diameter,
            SoundType::Weapon => self.config.player.sound.weapon_diameter,
        };
        if local_player.position.distance(player.position) > max_distance {
            return Some(0.0);
        }

        if time.elapsed() > self.total_sound_duration() {
            return Some(0.0);
        }

        Some(
            1.0 - ((time.elapsed().as_secs_f32() - self.config.player.sound.fadeout_start)
                / self.config.player.sound.fadeout_duration),
        )
    }

    fn total_sound_duration(&self) -> Duration {
        Duration::from_secs_f32(
            self.config.player.sound.fadeout_start + self.config.player.sound.fadeout_duration,
        )
    }

    fn alpha(color: Color32, alpha: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (alpha.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        Color32::from_rgba_unmultiplied(
            (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
            (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
            (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
            (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
        )
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        let alpha = match alpha {
            Some(alpha) => alpha.clamp(0.0, 1.0),
            None => 1.0,
        };
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);

        let esp_scale = (500.0 / distance).clamp(0.4, 1.0);
        let line_width = self.config.hud.line_width * esp_scale;

        let health_color =
            self.health_color(player.health, self.config.player.box_visible_color.a());
        let mut color = match &self.config.player.draw_box {
            DrawMode::None => health_color,
            DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.visible {
                    self.config.player.box_visible_color
                } else {
                    self.config.player.box_invisible_color
                }
            }
        };

        color = Self::alpha(color, alpha);

        let stroke = Stroke::new(line_width, color);
        let icon_line = self.config.hud.icon_size * esp_scale;
        let icon_font = FontId::monospace(icon_line);

        let midpoint = (player.position + player.head) / 2.0;
        let height = player.head.z - player.position.z + 24.0;
        let half_height = height / 2.0;
        let top = midpoint + vec3(0.0, 0.0, half_height);
        let bottom = midpoint - vec3(0.0, 0.0, half_height);

        let Some(top) = world_to_screen(&top, data) else {
            return;
        };
        let Some(bottom) = world_to_screen(&bottom, data) else {
            return;
        };
        let half_height = bottom.y - top.y;
        let width = half_height / 2.0;
        let half_width = width / 2.0;
        // quarter width
        let qw = half_width - 2.0;
        // eigth width
        let ew = qw / 2.0;

        let tl = pos2(top.x - half_width, top.y);
        let tr = pos2(top.x + half_width, top.y);
        let bl = pos2(bottom.x - half_width, bottom.y);
        let br = pos2(bottom.x + half_width, bottom.y);
        let rect_2d = egui::Rect::from_min_max(tl, br);

        if self.config.player.draw_box != DrawMode::None {
            match self.config.player.box_mode {
                BoxMode::ThreeD => {
                    self.draw_esp_3d_box(painter, player, data, color, line_width, alpha);
                }
                BoxMode::Full => {
                    if self.config.player.box_fill != BoxFill::None {
                        let fill_alpha = self.config.player.box_fill_alpha.clamp(0.0, 1.0) * alpha;
                        match self.config.player.box_fill {
                            BoxFill::None => {}
                            BoxFill::Solid => {
                                let fill_c = Self::alpha(color, fill_alpha);
                                painter.rect_filled(rect_2d, 0.0, fill_c);
                            }
                            BoxFill::Gradient => {
                                let darker = Color32::from_rgba_unmultiplied(
                                    (color.r() as f32 * 0.35) as u8,
                                    (color.g() as f32 * 0.35) as u8,
                                    (color.b() as f32 * 0.35) as u8,
                                    255,
                                );
                                let top_c = Self::alpha(color, fill_alpha);
                                let bot_c = Self::alpha(darker, fill_alpha);
                                let strips: usize = 14;
                                let h = rect_2d.height() / strips as f32;
                                for i in 0..strips {
                                    let denom = (strips - 1).max(1) as f32;
                                    let t = i as f32 / denom;
                                    let y0 = rect_2d.top() + h * i as f32;
                                    let y1 = (y0 + h).min(rect_2d.bottom());
                                    let slice = egui::Rect::from_min_max(
                                        pos2(rect_2d.left(), y0),
                                        pos2(rect_2d.right(), y1),
                                    );
                                    let c = Self::lerp_color(top_c, bot_c, t);
                                    painter.rect_filled(slice, 0.0, c);
                                }
                            }
                        }
                    }

                    painter.rect(
                        rect_2d,
                        0,
                        Color32::TRANSPARENT,
                        stroke,
                        egui::StrokeKind::Middle,
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

        // health bar
        if self.config.player.health_bar {
            let x = bl.x - line_width * 2.0;
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.health as f32 / 100.0)),
                ],
                Stroke::new(line_width, Self::alpha(health_color, alpha)),
            );
        }

        if self.config.player.armor_bar && player.armor > 0 {
            let x = bl.x
                - line_width
                    * if self.config.player.health_bar {
                        4.0
                    } else {
                        2.0
                    };
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.armor as f32 / 100.0)),
                ],
                Stroke::new(
                    line_width,
                    Self::alpha(Color32::BLUE, alpha),
                ),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size * esp_scale;
        let text_color = Self::alpha(self.config.hud.text_color, alpha);
        if self.config.player.player_name {
            self.text_sized(
                painter,
                &player.name,
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                Some(text_color),
                font_size,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_defuser {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e00f}",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_helmet {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e017}",
                icon_font.clone(),
                text_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_bomb {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e01e}",
                icon_font.clone(),
                text_color,
            );
        }

        if self.config.player.weapon_icon {
            painter.text(
                pos2(bl.x + half_width, bl.y),
                Align2::CENTER_TOP,
                player.weapon.to_icon(),
                icon_font.clone(),
                text_color,
            );
            if player.ammo.0 >= 0 {
                const AMMO_GAP: f32 = 3.0;
                self.text_sized(
                    painter,
                    format!("{}/{}", player.ammo.0, player.ammo.1),
                    pos2(bl.x + half_width, bl.y + icon_line + AMMO_GAP),
                    Align2::CENTER_TOP,
                    Some(text_color),
                    font_size,
                );
            }
        }

        // Place distance / flags below the weapon row (icon uses icon_size, not font_size).
        const SECTION_GAP: f32 = 4.0;
        let mut below = bl.y;
        if self.config.player.weapon_icon {
            below += icon_line;
            if player.ammo.0 >= 0 {
                below += SECTION_GAP + font_size;
            }
            below += SECTION_GAP;
        } else {
            below += SECTION_GAP;
        }

        if self.config.player.esp_distance_meters {
            let dist_font = font_size * 0.95;
            let m = distance / HAMMER_UNITS_PER_METER;
            self.text_sized(
                painter,
                format!("{:.0} m", m),
                pos2(bottom.x, below),
                Align2::CENTER_TOP,
                Some(text_color),
                dist_font,
            );
            below += dist_font + 2.0;
        }

        if self.config.player.esp_status_flags && (player.is_scoped || player.is_flashed) {
            let mut flags = String::new();
            if player.is_scoped {
                flags.push_str("[SCOPE]");
            }
            if player.is_flashed {
                if !flags.is_empty() {
                    flags.push(' ');
                }
                flags.push_str("[FLASH]");
            }
            let flag_color = if player.is_flashed {
                Color32::YELLOW
            } else {
                text_color
            };
            let flags_font = font_size * 0.85;
            self.text_sized(
                painter,
                flags,
                pos2(bottom.x, below),
                Align2::CENTER_TOP,
                Some(Self::alpha(flag_color, alpha)),
                flags_font,
            );
        }
    }

    fn draw_esp_3d_box(
        &self,
        painter: &Painter,
        player: &PlayerData,
        data: &Data,
        stroke_rgb: Color32,
        line_width: f32,
        fade: f32,
    ) {
        let stroke_c = Self::alpha(stroke_rgb, fade);
        let stroke = Stroke::new(line_width, stroke_c);

        let yaw = player.rotation.to_radians();
        let fwd = vec3(yaw.cos(), yaw.sin(), 0.0);
        let right = vec3(-(yaw.sin()), yaw.cos(), 0.0);

        // Tighter than the 2D ESP (+24): 3D hull should hug the model more closely.
        let z_span = (player.head.z - player.position.z).max(8.0);
        let height = z_span + 10.0;
        // Standing hull ~32×32 in Source; previous 20×14 read fat (especially on screen).
        let hw = 15.0f32;
        let hd = 11.0f32;
        let feet = player.position;
        let base = [
            feet + right * hw + fwd * hd,
            feet - right * hw + fwd * hd,
            feet - right * hw - fwd * hd,
            feet + right * hw - fwd * hd,
        ];
        let up = vec3(0.0, 0.0, height);
        let w: [Vec3; 8] = [
            base[0],
            base[1],
            base[2],
            base[3],
            base[0] + up,
            base[1] + up,
            base[2] + up,
            base[3] + up,
        ];

        let mut scr: Vec<egui::Pos2> = Vec::with_capacity(8);
        for c in &w {
            let Some(s) = world_to_screen(c, data) else {
                return;
            };
            scr.push(s);
        }

        let box_fill = self.config.player.box_fill;
        if box_fill != BoxFill::None {
            let fill_alpha = self.config.player.box_fill_alpha.clamp(0.0, 1.0) * fade;
            let eye = data.local_player.head;
            let z_min = w.iter().map(|p| p.z).fold(f32::INFINITY, f32::min);
            let z_max = w.iter().map(|p| p.z).fold(f32::NEG_INFINITY, f32::max);
            let z_den = (z_max - z_min).max(1e-3);

            let darker = Color32::from_rgba_unmultiplied(
                (stroke_rgb.r() as f32 * 0.35) as u8,
                (stroke_rgb.g() as f32 * 0.35) as u8,
                (stroke_rgb.b() as f32 * 0.35) as u8,
                255,
            );
            let top_c = Self::alpha(stroke_rgb, fill_alpha);
            let bot_c = Self::alpha(darker, fill_alpha);

            let vert_color = |z: f32| -> Color32 {
                match box_fill {
                    BoxFill::None => Color32::TRANSPARENT,
                    BoxFill::Solid => top_c,
                    BoxFill::Gradient => {
                        let t = ((z - z_min) / z_den).clamp(0.0, 1.0);
                        Self::lerp_color(top_c, bot_c, t)
                    }
                }
            };

            // World-space face quads (CCW from outside). Skip bottom cap (not useful; clips ground).
            let faces: [(usize, usize, usize, usize); 5] = [
                (0, 1, 5, 4), // +fwd
                (3, 7, 6, 2), // -fwd
                (0, 4, 7, 3), // +right
                (1, 2, 6, 5), // -right
                (4, 5, 6, 7), // +Z top
            ];

            let mut filled: Vec<(f32, Mesh)> = Vec::with_capacity(5);
            'faces: for &(a, b, c, d) in &faces {
                let wa = w[a];
                let wb = w[b];
                let wc = w[c];
                let wd = w[d];
                let center = (wa + wb + wc + wd) * 0.25;
                let n = (wb - wa).cross(wc - wa);
                if n.length_squared() < 1e-12 {
                    continue;
                }
                let n = n.normalize();
                if n.dot(eye - center) <= 0.01 {
                    continue;
                }

                let mut max_d2 = 0.0f32;
                for p in [wa, wb, wc, wd] {
                    max_d2 = max_d2.max(eye.distance_squared(p));
                }

                let mut projected: Vec<(egui::Pos2, Color32)> = Vec::with_capacity(4);
                for corner in [wa, wb, wc, wd] {
                    let Some(s) = world_to_screen(&corner, data) else {
                        continue 'faces;
                    };
                    projected.push((s, vert_color(corner.z)));
                }
                if projected.len() != 4 {
                    continue;
                }

                let mut mesh = Mesh::default();
                let i0 = mesh.vertices.len() as u32;
                for (s, col) in projected {
                    mesh.colored_vertex(s, col);
                }
                mesh.add_triangle(i0, i0 + 1, i0 + 2);
                mesh.add_triangle(i0, i0 + 2, i0 + 3);
                filled.push((max_d2, mesh));
            }

            filled.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            for (_, mesh) in filled {
                if !mesh.is_empty() {
                    painter.add(Shape::Mesh(mesh.into()));
                }
            }
        }

        let edge = |p: &Painter, i: usize, j: usize| {
            if let (Some(a), Some(b)) = (scr.get(i), scr.get(j)) {
                p.line_segment([*a, *b], stroke);
            }
        };

        edge(painter, 0, 1);
        edge(painter, 1, 2);
        edge(painter, 2, 3);
        edge(painter, 3, 0);
        edge(painter, 4, 5);
        edge(painter, 5, 6);
        edge(painter, 6, 7);
        edge(painter, 7, 4);
        edge(painter, 0, 4);
        edge(painter, 1, 5);
        edge(painter, 2, 6);
        edge(painter, 3, 7);
    }

    fn skeleton(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);
        let esp_scale = (500.0 / distance).clamp(0.25, 1.0);

        let mut color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => {
                self.health_color(player.health, self.config.player.skeleton_color.a())
            }
            DrawMode::Color => self.config.player.skeleton_color,
        };
        if let Some(alpha) = alpha {
            color = Self::alpha(color, alpha);
        }
        let stroke = Stroke::new(self.config.hud.line_width * esp_scale, color);

        for (a, b) in &Bones::CONNECTIONS {
            let Some(a) = player.bones.get(a) else {
                continue;
            };
            let Some(b) = player.bones.get(b) else {
                continue;
            };

            let Some(a) = world_to_screen(a, data) else {
                continue;
            };
            let Some(b) = world_to_screen(b, data) else {
                continue;
            };

            painter.line(vec![a, b], stroke);
        }

        // head circle
        if !self.config.player.head_circle {
            return;
        }
        let Some(neck) = player.bones.get(&Bones::Neck) else {
            return;
        };
        let Some(spine) = player.bones.get(&Bones::Spine3) else {
            return;
        };

        let Some(neck) = world_to_screen(neck, data) else {
            return;
        };
        let Some(spine) = world_to_screen(spine, data) else {
            return;
        };

        let height = spine.y - neck.y;
        let pos = pos2(neck.x - (spine.x - neck.x) / 2.0, neck.y - height / 2.0);
        painter.circle_stroke(pos, height / 2.0, stroke);
    }

    pub fn update_player_sounds(&mut self) {
        let data = self.data.lock();

        for player in &data.players {
            let Some(sound) = &player.sound else {
                continue;
            };

            self.player_sounds
                .insert(player.steam_id, (Instant::now(), *sound));
        }

        let total_duration = self.total_sound_duration();
        self.player_sounds
            .retain(|_, (time, _)| time.elapsed() < total_duration);
    }
}
