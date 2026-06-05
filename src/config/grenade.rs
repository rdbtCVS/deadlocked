use serde::{Deserialize, Serialize};

use crate::cs2::key_codes::KeyCode;

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
