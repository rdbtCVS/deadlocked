use crate::{
    config::{Config, KeyMode},
    cs2::CS2,
};

#[derive(Debug)]
pub struct EspToggle {
    pub active: bool,
}

impl Default for EspToggle {
    fn default() -> Self {
        Self { active: true }
    }
}

impl CS2 {
    pub fn esp_toggle(&mut self, config: &Config) {
        if config.player.esp_mode != KeyMode::Toggle {
            return;
        }
        let Some(key) = config.player.esp_hotkey else {
            return;
        };
        if self.input.key_just_pressed(key) {
            self.esp.active = !self.esp.active;
        }
    }

    pub fn esp_enabled(&self, config: &Config) -> bool {
        if !config.player.enabled {
            return false;
        }
        match config.player.esp_mode {
            KeyMode::Toggle => self.esp.active,
            KeyMode::Hold => config
                .player
                .esp_hotkey
                .is_some_and(|k| self.input.is_key_pressed(k)),
        }
    }
}
