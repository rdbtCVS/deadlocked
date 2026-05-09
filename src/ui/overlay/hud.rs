use std::time::Instant;

use egui::{Align2, Color32, Painter, Stroke, pos2};

use strum::IntoEnumIterator as _;

use crate::{
    config::KeyMode,
    cs2::entity::weapon_class::WeaponClass,
    data::Data,
    hotkeys::{self, HotkeySlot},
    math::world_to_screen,
    ui::app::App,
};

impl App {
    pub fn overlay_debug(&self, painter: &Painter, data: &Data) {
        if self.config.hud.debug {
            painter.line(
                vec![pos2(0.0, 0.0), pos2(data.window_size.x, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
            painter.line(
                vec![pos2(data.window_size.x, 0.0), pos2(0.0, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
        }
    }

    pub fn draw_bomb_timer(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.bomb_timer || !data.bomb.planted {
            return;
        }

        if let Some(pos) = world_to_screen(&data.bomb.position, data) {
            self.text(
                painter,
                format!("{:.3}", data.bomb.timer),
                pos,
                Align2::CENTER_CENTER,
                None,
            );
            if data.bomb.being_defused {
                self.text(
                    painter,
                    format!("defusing {:.3}", data.bomb.defuse_remain_time),
                    pos2(pos.x, pos.y + self.config.hud.font_size),
                    Align2::CENTER_CENTER,
                    None,
                );
            }
        }

        let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
        let color = self.health_color((fraction * 100.0) as i32, 255);
        painter.line(
            vec![
                pos2(0.0, data.window_size.y),
                pos2(data.window_size.x * fraction, data.window_size.y),
            ],
            Stroke::new(self.config.hud.line_width * 3.0, color),
        );
    }

    pub fn draw_fov_circle(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.fov_circle || !data.in_game {
            return;
        }

        let weapon_config = self.aimbot_config(&data.weapon);

        if !weapon_config.enabled || (weapon_config.mode == KeyMode::Toggle && !data.aimbot_active)
        {
            return;
        }

        let aim_fov = weapon_config.fov;

        if weapon_config.distance_adjusted_fov {
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 125.0, Color32::GREEN);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 250.0, Color32::YELLOW);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 500.0, Color32::RED);
        } else {
            self.draw_simple_fov_circle(painter, data, aim_fov, Color32::WHITE);
        }
    }

    pub fn draw_keybind_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.keybind_list {
            return;
        }

        let fs = self.config.hud.font_size;
        let position = pos2(10.0, data.window_size.y / 2.0);
        let mut row: f32 = 0.0;

        let cfg = &self.config;
        for slot in HotkeySlot::iter() {
            let Some(key) = hotkeys::key(cfg, slot) else {
                continue;
            };
            let name = slot.label();
            let color = if hotkeys::hud_active(data, slot) {
                Color32::GREEN
            } else {
                Color32::WHITE
            };
            self.text(
                painter,
                format!("{key:?} {name}"),
                position + egui::vec2(0.0, row * fs),
                Align2::LEFT_TOP,
                Some(color),
            );
            row += 1.0;
        }
    }

    pub fn draw_spectator_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spectator_list {
            return;
        }

        let fs = self.config.hud.font_size;
        let bound_count = HotkeySlot::iter()
            .filter(|s| hotkeys::key(&self.config, *s).is_some())
            .count();
        let y_offset = if self.config.hud.keybind_list {
            fs * (bound_count as f32 + 1.0)
        } else {
            fs * 3.0
        };
        let position = pos2(10.0, data.window_size.y / 2.0 + y_offset);
        self.text(
            painter,
            "Spectators:",
            position,
            Align2::LEFT_TOP,
            Some(Color32::WHITE),
        );

        for (i, name) in data.spectators.iter().enumerate() {
            self.text(
                painter,
                format!("> {name}"),
                position + egui::vec2(0.0, self.config.hud.font_size * (i as f32 + 1.0)),
                Align2::LEFT_TOP,
                Some(Color32::WHITE),
            );
        }
    }

    fn get_current_fov(&self) -> f32 {
        (if self.config.misc.fov_changer {
            self.config.misc.desired_fov
        } else {
            crate::constants::cs2::DEFAULT_FOV
        }) as f32
    }

    fn calculate_fov_radius(&self, data: &Data, target_fov: f32) -> f32 {
        let current_fov = self.get_current_fov();
        let screen_width = data.window_size.x;

        let current_fov_tan = (current_fov.to_radians() / 2.0).tan();
        if current_fov_tan == 0.0 {
            return 0.0;
        }

        let target_fov_tan = (target_fov.to_radians() / 2.0).tan();
        (target_fov_tan / current_fov_tan) * (screen_width / 2.0)
    }

    fn draw_fov_circle_impl(&self, painter: &Painter, data: &Data, radius: f32, color: Color32) {
        let center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);
        let stroke = Stroke::new(self.config.hud.line_width, color);
        painter.circle_stroke(center, radius, stroke);
    }

    fn get_distance_fov_scale(&self, distance: f32) -> f32 {
        (5.0 - (distance / 125.0)).max(1.0)
    }

    fn draw_simple_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        target_fov: f32,
        color: Color32,
    ) {
        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    fn draw_distance_scaled_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        base_aim_fov: f32,
        distance: f32,
        color: Color32,
    ) {
        let scale = self.get_distance_fov_scale(distance);
        let target_fov = base_aim_fov * scale;

        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    pub fn draw_sniper_crosshair(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.sniper_crosshair.enabled
            || WeaponClass::from_string(data.weapon.as_ref()) != WeaponClass::Sniper
        {
            return;
        }

        let length = self.config.hud.sniper_crosshair.line_length;
        let gap = self.config.hud.sniper_crosshair.gap / 2.0;
        let center = data.window_size / 2.0;

        let stroke = Stroke::new(
            self.config.hud.sniper_crosshair.line_width,
            self.config.hud.sniper_crosshair.color,
        );

        painter.line_segment(
            [
                pos2(center.x + gap, center.y),
                pos2(center.x + gap + length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + gap),
                pos2(center.x, center.y + gap + length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y),
                pos2(center.x - gap - length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - gap),
                pos2(center.x, center.y - gap - length),
            ],
            stroke,
        );
    }

    pub fn draw_vote_hud(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.vote_hud || !data.in_game {
            return;
        }
        let v = &data.vote;
        if !v.active {
            return;
        }
        let x = data.window_size.x * 0.5;
        let y = 10.0;
        let fs = self.config.hud.font_size * 1.05;
        let line = if v.is_yes_no {
            format!(
                "Vote  YES {}  ·  NO {}  ·  {}/{}",
                v.options[0],
                v.options[1],
                v.options[0] + v.options[1],
                v.potential_votes.max(1),
            )
        } else {
            format!("Vote  {:?}  potential {}", v.options, v.potential_votes)
        };
        self.text_sized(
            painter,
            line,
            pos2(x, y),
            Align2::CENTER_TOP,
            Some(Color32::YELLOW),
            fs,
        );
    }

    pub fn draw_hit_marker(&mut self, painter: &Painter, window_size: glam::Vec2, in_game: bool) {
        if !self.config.hud.hit_marker || !in_game {
            return;
        }
        let Some(until) = self.hit_marker_until else {
            return;
        };
        let now = Instant::now();
        let Some(remaining) = until.checked_duration_since(now) else {
            self.hit_marker_until = None;
            return;
        };
        let fade = (remaining.as_secs_f32() / 0.42).clamp(0.0, 1.0);
        let a = (fade * 255.0) as u8;
        let center = window_size / 2.0;
        let len = 18.0 * fade.max(0.2);
        let gap = 5.0;
        let col = Color32::from_rgba_unmultiplied(255, 220, 120, a);
        let stroke = Stroke::new(self.config.hud.line_width * 1.5, col);
        painter.line_segment(
            [
                pos2(center.x + gap, center.y),
                pos2(center.x + gap + len, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y),
                pos2(center.x - gap - len, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + gap),
                pos2(center.x, center.y + gap + len),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - gap),
                pos2(center.x, center.y - gap - len),
            ],
            stroke,
        );

        if self.last_hit_damage >= 1.0 && fade > 0.15 {
            self.text_sized(
                painter,
                format!("-{:.0}", self.last_hit_damage),
                pos2(
                    center.x,
                    center.y - gap - len - self.config.hud.font_size * 1.2,
                ),
                Align2::CENTER_BOTTOM,
                Some(Color32::from_rgba_unmultiplied(255, 200, 80, a)),
                self.config.hud.font_size * 1.15,
            );
        }
    }
}
