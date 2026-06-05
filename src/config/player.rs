use egui::Color32;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{config::aim::KeyMode, cs2::key_codes::KeyCode};

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum DrawMode {
    None,
    Health,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum BoxMode {
    Gap,
    Full,
    /// Yaw-oriented 3D wireframe in world space.
    ThreeD,
}

impl std::fmt::Display for BoxMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoxMode::Gap => write!(f, "2D gap"),
            BoxMode::Full => write!(f, "2D full"),
            BoxMode::ThreeD => write!(f, "3D"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum BoxFill {
    None,
    Solid,
    Gradient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EspZone {
    Top,
    Left,
    Right,
    Bottom,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EspElementLayout {
    pub health_bar: EspZone,
    pub armor_bar: EspZone,
    pub player_name: EspZone,
    pub tags: EspZone,
    pub weapon_icon: EspZone,
    pub distance_meters: EspZone,
    pub status_flags: EspZone,
}

impl Default for EspElementLayout {
    fn default() -> Self {
        Self {
            health_bar: EspZone::Left,
            armor_bar: EspZone::Left,
            player_name: EspZone::Right,
            tags: EspZone::Right,
            weapon_icon: EspZone::Bottom,
            distance_meters: EspZone::Bottom,
            status_flags: EspZone::Bottom,
        }
    }
}

fn default_legacy_esp_enabled() -> bool {
    true
}

fn default_esp_hotkey() -> Option<KeyCode> {
    Some(KeyCode::X)
}

fn default_esp_mode() -> KeyMode {
    KeyMode::Toggle
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    #[serde(
        default = "default_esp_hotkey",
        skip_serializing_if = "Option::is_none"
    )]
    pub esp_hotkey: Option<KeyCode>,
    #[serde(default = "default_esp_mode")]
    pub esp_mode: KeyMode,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub head_circle: bool,
    #[serde(
        rename = "health_bar",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub health_bar_legacy: bool,
    #[serde(
        rename = "armor_bar",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub armor_bar_legacy: bool,
    #[serde(
        rename = "player_name",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub player_name_legacy: bool,
    #[serde(
        rename = "weapon_icon",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub weapon_icon_legacy: bool,
    #[serde(
        rename = "tags",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub tags_legacy: bool,
    pub visible_only: bool,
    pub sound: SoundConfig,
    pub box_fill: BoxFill,
    pub box_fill_alpha: f32,
    #[serde(
        rename = "esp_distance_meters",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub esp_distance_meters_legacy: bool,
    #[serde(
        rename = "esp_status_flags",
        default = "default_legacy_esp_enabled",
        skip_serializing
    )]
    pub esp_status_flags_legacy: bool,
    pub esp_layout: EspElementLayout,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            esp_hotkey: default_esp_hotkey(),
            esp_mode: KeyMode::Toggle,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            head_circle: true,
            health_bar_legacy: true,
            armor_bar_legacy: true,
            player_name_legacy: true,
            weapon_icon_legacy: true,
            tags_legacy: true,
            visible_only: false,
            sound: SoundConfig::default(),
            box_fill: BoxFill::None,
            box_fill_alpha: 0.25,
            esp_distance_meters_legacy: true,
            esp_status_flags_legacy: true,
            esp_layout: EspElementLayout::default(),
        }
    }
}

impl PlayerConfig {
    pub fn migrate_legacy_esp_toggles_into_layout(&mut self) {
        use EspZone::Disabled;

        let layout_is_builtin_default = self.esp_layout == EspElementLayout::default();

        if layout_is_builtin_default {
            if !self.health_bar_legacy {
                self.esp_layout.health_bar = Disabled;
            }
            if !self.armor_bar_legacy {
                self.esp_layout.armor_bar = Disabled;
            }
            if !self.player_name_legacy {
                self.esp_layout.player_name = Disabled;
            }
            if !self.weapon_icon_legacy {
                self.esp_layout.weapon_icon = Disabled;
            }
            if !self.tags_legacy {
                self.esp_layout.tags = Disabled;
            }
            if !self.esp_distance_meters_legacy {
                self.esp_layout.distance_meters = Disabled;
            }
            if !self.esp_status_flags_legacy {
                self.esp_layout.status_flags = Disabled;
            }
        }

        self.health_bar_legacy = true;
        self.armor_bar_legacy = true;
        self.player_name_legacy = true;
        self.weapon_icon_legacy = true;
        self.tags_legacy = true;
        self.esp_distance_meters_legacy = true;
        self.esp_status_flags_legacy = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    pub footstep_diameter: f32,
    pub gunshot_diameter: f32,
    pub weapon_diameter: f32,
    pub fadeout_start: f32,
    pub fadeout_duration: f32,
    pub show_visible: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            footstep_diameter: crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT,
            gunshot_diameter: crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT,
            weapon_diameter: crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT,
            fadeout_start: 1.0,
            fadeout_duration: 1.0,
            show_visible: true,
        }
    }
}
