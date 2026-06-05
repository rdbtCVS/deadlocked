use strum::EnumIter;

use crate::{
    config::{
        Config,
        aim::{AimConfig, AimbotConfig, TriggerbotConfig},
    },
    cs2::{entity::weapon::Weapon, key_codes::KeyCode},
    data::Data,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum HotkeySlot {
    Aimbot,
    Triggerbot,
    Esp,
}

impl HotkeySlot {
    pub fn label(self) -> &'static str {
        match self {
            HotkeySlot::Aimbot => "Aimbot",
            HotkeySlot::Triggerbot => "Triggerbot",
            HotkeySlot::Esp => "ESP",
        }
    }

    pub fn ui_id(self) -> &'static str {
        match self {
            HotkeySlot::Aimbot => "aimbot_hotkey",
            HotkeySlot::Triggerbot => "triggerbot_hotkey",
            HotkeySlot::Esp => "esp_hotkey",
        }
    }
}

pub fn resolved_aimbot_config<'a>(aim: &'a AimConfig, weapon: &Weapon) -> &'a AimbotConfig {
    if let Some(wc) = aim.weapons.get(weapon)
        && wc.aimbot.enable_override
    {
        return &wc.aimbot;
    }
    &aim.global.aimbot
}

pub fn resolved_triggerbot_config<'a>(aim: &'a AimConfig, weapon: &Weapon) -> &'a TriggerbotConfig {
    if let Some(wc) = aim.weapons.get(weapon)
        && wc.triggerbot.enable_override
    {
        return &wc.triggerbot;
    }
    &aim.global.triggerbot
}

pub fn key(config: &Config, slot: HotkeySlot) -> Option<KeyCode> {
    match slot {
        HotkeySlot::Aimbot => config.aim.aimbot_hotkey,
        HotkeySlot::Triggerbot => config.aim.triggerbot_hotkey,
        HotkeySlot::Esp => config.player.esp_hotkey,
    }
}

pub fn hud_active(data: &Data, slot: HotkeySlot) -> bool {
    match slot {
        HotkeySlot::Aimbot => data.aimbot_active,
        HotkeySlot::Triggerbot => data.triggerbot_active,
        HotkeySlot::Esp => data.esp_active,
    }
}
