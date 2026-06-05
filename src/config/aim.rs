use std::{collections::HashMap, ops::RangeInclusive};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::cs2::{bones::Bones, entity::weapon::Weapon, key_codes::KeyCode};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: KeyMode,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub flash_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub inertia: f32,
    pub curve: f32,
    pub humanization: f32,
    pub bones: Vec<Bones>,
    pub targeting_mode: TargetingMode,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            mode: KeyMode::Hold,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 0,
            visibility_check: true,
            flash_check: true,
            fov: 2.5,
            smooth: 5.0,
            inertia: 1.0,
            curve: 0.0,
            humanization: 0.0,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            targeting_mode: TargetingMode::Fov,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub strength: Vec2,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            strength: Vec2::splat(0.5),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum KeyMode {
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TargetingMode {
    Fov,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub magnetized: bool,
    pub pixel_radius: f32,
    pub visible_only: bool,
    pub team_check: bool,
    pub extra_hold_us: u32,
    pub extra_min_interval_ms: u32,
    pub extra_reaction_delay_ms: u32,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub mode: KeyMode,
    pub flash_check: bool,
    pub scope_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub head_only: bool,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            magnetized: false,
            pixel_radius: 2.5,
            visible_only: true,
            team_check: true,
            extra_hold_us: 0,
            extra_min_interval_ms: 0,
            extra_reaction_delay_ms: 0,
            delay: 100..=200,
            shot_duration: 200,
            mode: KeyMode::Hold,
            flash_check: true,
            scope_check: true,
            velocity_check: true,
            velocity_threshold: 100.0,
            head_only: false,
        }
    }
}

fn default_aimbot_hotkey() -> Option<KeyCode> {
    Some(KeyCode::Mouse5)
}

fn default_triggerbot_hotkey() -> Option<KeyCode> {
    Some(KeyCode::Mouse4)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    #[serde(
        default = "default_aimbot_hotkey",
        skip_serializing_if = "Option::is_none"
    )]
    pub aimbot_hotkey: Option<KeyCode>,
    #[serde(
        default = "default_triggerbot_hotkey",
        skip_serializing_if = "Option::is_none"
    )]
    pub triggerbot_hotkey: Option<KeyCode>,
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aimbot_hotkey: default_aimbot_hotkey(),
            triggerbot_hotkey: default_triggerbot_hotkey(),
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}
