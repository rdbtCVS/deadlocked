#![allow(unused)]
use egui::Color32;
use serde::{Deserialize, Serialize};

pub struct Colors;

impl Colors {
    pub const BG: Color32 = Color32::from_rgb(0x0A, 0x0A, 0x0A);
    pub const PANEL: Color32 = Color32::from_rgb(0x1A, 0x1A, 0x1A);
    pub const PANEL_ALT: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
    pub const INPUT: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
    pub const HOVER: Color32 = Color32::from_rgb(0x22, 0x22, 0x22);
    pub const TEXT: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF5);
    pub const SUBTEXT: Color32 = Color32::from_rgb(0x8A, 0x8A, 0x8A);
    pub const MUTED: Color32 = Color32::from_rgb(0x55, 0x55, 0x55);

    pub const BACKDROP: Color32 = Self::BG;
    pub const BASE: Color32 = Self::PANEL;
    pub const HIGHLIGHT: Color32 = Self::HOVER;
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(160, 240, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(100, 150, 240);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);

    pub const ACCENT_COLORS: [(&str, Color32); 7] = [
        ("Red", Self::RED),
        ("Orange", Self::ORANGE),
        ("Yellow", Self::YELLOW),
        ("Green", Self::GREEN),
        ("Teal", Self::TEAL),
        ("Blue", Self::BLUE),
        ("Purple", Self::PURPLE),
    ];
}
