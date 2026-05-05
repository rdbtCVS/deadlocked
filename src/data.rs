use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};
use serde::Serialize;

use crate::cs2::{
    bones::Bones,
    entity::{EntityInfo, weapon::Weapon},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SoundType {
    Footstep,
    Gunshot,
    Weapon,
}

#[derive(Debug, Default, Serialize)]
pub struct Data {
    pub in_game: bool,
    pub is_ffa: bool,
    pub weapon: Weapon,
    pub players: Vec<PlayerData>,
    pub friendlies: Vec<PlayerData>,
    pub spectators: Vec<String>,
    pub local_player: PlayerData,
    pub entities: Vec<EntityInfo>,
    pub bomb: BombData,
    pub map_name: String,
    pub view_matrix: Mat4,
    pub view_angles: Vec2,
    pub window_position: Vec2,
    pub window_size: Vec2,
    pub aimbot_active: bool,
    pub triggerbot_active: bool,
    pub esp_active: bool,
    /// True for one frame when local player's round damage increased (hit feedback).
    pub hit_pulse: bool,
    pub hit_damage_delta: f32,
    pub vote: VoteState,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PlayerData {
    pub steam_id: u64,
    pub health: i32,
    pub armor: i32,
    pub position: Vec3,
    pub head: Vec3,
    pub name: String,
    pub weapon: Weapon,
    pub ammo: (i32, i32),
    pub bones: HashMap<Bones, Vec3>,
    pub has_defuser: bool,
    pub has_helmet: bool,
    pub has_bomb: bool,
    pub visible: bool,
    pub color: i32,
    pub rotation: f32,
    pub sound: Option<SoundType>,
    pub is_scoped: bool,
    pub is_flashed: bool,
    pub team: u8,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct VoteState {
    pub active: bool,
    pub is_yes_no: bool,
    pub potential_votes: i32,
    pub options: [i32; 5],
}

#[derive(Debug, Default, Serialize)]
pub struct BombData {
    pub planted: bool,
    pub timer: f32,
    pub being_defused: bool,
    pub position: Vec3,
    pub defuse_remain_time: f32,
}
