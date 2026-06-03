use std::{
    collections::HashMap,
    fs::read_to_string,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use egui::Color32;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    cs2::{bones::Bones, entity::weapon::Weapon, key_codes::KeyCode},
    ui::color::Colors,
};

pub const SLEEP_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONFIG_NAME: &str = "deadlocked.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
    pub first_launch: bool,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self { first_launch: true }
    }
}

pub fn read_app_config() -> ApplicationConfig {
    if !APP_CONFIG_PATH.exists() {
        return ApplicationConfig::default();
    }

    let Ok(config_string) = read_to_string(APP_CONFIG_PATH.as_path()) else {
        return ApplicationConfig::default();
    };

    let config = toml::from_str(&config_string);
    config.unwrap_or_default()
}

#[allow(dead_code)]
pub fn write_app_config(config: &ApplicationConfig) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(APP_CONFIG_PATH.as_path(), out);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub aim: AimConfig,
    pub player: PlayerConfig,
    pub hud: HudConfig,
    pub grenade: GrenadeConfig,
    pub misc: UnsafeConfig,
    pub accent_color: Color32,
    pub fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            player: PlayerConfig::default(),
            hud: HudConfig::default(),
            grenade: GrenadeConfig::default(),
            misc: UnsafeConfig::default(),
            accent_color: Colors::BLUE,
            fps: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrenadeConfig {
    pub automation_enabled: bool,
    pub automation_hotkey: KeyCode,
    pub activation_distance: f32,
    pub marker_draw_distance: f32,
    pub position_tolerance: f32,
    pub velocity_minimum: f32,
    pub aim_fov: f32,
    pub aim_smooth: f32,
    pub aim_inertia: f32,
    pub aim_curve: f32,
    pub aim_humanization: f32,
    pub aim_tolerance: f32,
}

impl Default for GrenadeConfig {
    fn default() -> Self {
        Self {
            automation_enabled: true,
            automation_hotkey: KeyCode::G,
            activation_distance: 24.0,
            marker_draw_distance: 500.0,
            position_tolerance: 2.0,
            velocity_minimum: 10.0,
            aim_fov: 25.0,
            aim_smooth: 8.0,
            aim_inertia: 0.0,
            aim_curve: 0.0,
            aim_humanization: 0.0,
            aim_tolerance: 0.2,
        }
    }
}

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
    /// Yaw-oriented 3D wireframe in world space (mutually exclusive with 2D box styles).
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

/// Where HUD elements are anchored relative to the 2D player box (`Disabled` skips drawing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EspZone {
    Top,
    Left,
    Right,
    Bottom,
    /// Do not draw this element on the overlay.
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
    /// Legacy checkbox; superseded by `esp_layout`; read from `[player]` TOML only.
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
    /// Fill inside the 2D box (gap mode has no inner rect; use full box for fill).
    pub box_fill: BoxFill,
    /// 0–1 alpha used for solid / gradient fill.
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
    /// Migrate pre-layout-only configs: `[player] health_bar = false` etc. map to `esp_layout.* =
    /// Disabled`, then canonicalize legacy flags so they aren't re-serialized.
    ///
    /// If `esp_layout` already differs from defaults, it is assumed intentional (explicit TOML
    /// layout) and legacy booleans do not overwrite it — only legacy keys are canonicalized away.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub fov_circle: bool,
    pub sniper_crosshair: CrosshairConfig,
    pub dropped_weapons: bool,
    pub keybind_list: bool,
    pub spectator_list: bool,
    pub grenade_trails: bool,
    pub smoke_trail_color: Color32,
    pub molotov_trail_color: Color32,
    pub incendiary_trail_color: Color32,
    pub flash_trail_color: Color32,
    pub he_trail_color: Color32,
    pub decoy_trail_color: Color32,
    pub text_outline: bool,
    pub text_color: Color32,
    pub line_width: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub debug: bool,
    pub vote_hud: bool,
    pub hit_marker: bool,
    pub hit_sound: bool,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            fov_circle: false,
            sniper_crosshair: CrosshairConfig::default(),
            dropped_weapons: true,
            keybind_list: false,
            spectator_list: false,
            grenade_trails: true,
            smoke_trail_color: Color32::LIGHT_GRAY,
            molotov_trail_color: Color32::RED,
            incendiary_trail_color: Color32::ORANGE,
            flash_trail_color: Color32::WHITE,
            he_trail_color: Color32::DARK_GRAY,
            decoy_trail_color: Color32::PURPLE,
            text_outline: true,
            text_color: Colors::TEXT,
            line_width: 2.0,
            font_size: 16.0,
            icon_size: 20.0,
            debug: false,
            vote_hud: true,
            hit_marker: true,
            hit_sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosshairConfig {
    pub enabled: bool,
    pub color: Color32,
    pub line_length: f32,
    pub line_width: f32,
    pub gap: f32,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Color32::WHITE,
            line_length: 50.0,
            line_width: 2.0,
            gap: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnsafeConfig {
    pub no_flash: bool,
    pub max_flash_alpha: f32,
    pub fov_changer: bool,
    pub desired_fov: u32,
    pub no_smoke: bool,
    pub change_smoke_color: bool,
    pub smoke_color: Color32,
}

impl Default for UnsafeConfig {
    fn default() -> Self {
        Self {
            no_flash: false,
            max_flash_alpha: 127.0,
            fov_changer: false,
            desired_fov: 90,
            no_smoke: false,
            change_smoke_color: false,
            smoke_color: Color32::RED,
        }
    }
}

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("deadlocked"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("configs");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static APP_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| BASE_PATH.join("deadlocked.toml"));

pub fn parse_config(path: &Path) -> Config {
    if !path.exists() || path.is_dir() {
        return Config::default();
    }

    let Ok(config_string) = read_to_string(path) else {
        return Config::default();
    };

    let mut config: Config = match toml::from_str(&config_string) {
        Ok(c) => {
            if let Some(file_name) = path.file_name() {
                utils::info!("loaded config {:?}", file_name);
            }
            c
        }
        Err(_) => {
            utils::warn!("config file invalid");
            Config::default()
        }
    };
    config.player.migrate_legacy_esp_toggles_into_layout();
    config
}

pub fn write_config(config: &Config, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(path, out);
}

pub fn delete_config(path: &Path) {
    if !path.exists() {
        return;
    }

    if std::fs::remove_file(path).is_ok()
        && let Some(file_name) = path.file_name()
    {
        utils::info!("deleted config {:?}", file_name);
    }
}

pub fn available_configs() -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(8);
    let Ok(dir) = std::fs::read_dir::<&Path>(CONFIG_PATH.as_ref()) else {
        return files;
    };

    for path in dir {
        let Ok(file) = path else {
            continue;
        };
        let Ok(file_type) = file.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.ends_with(".toml") {
            continue;
        }
        files.push(file.path())
    }
    if files.is_empty() {
        let path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        write_config(&Config::default(), &path);
        files.push(path);
    }
    files
}
