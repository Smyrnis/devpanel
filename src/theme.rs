// src/theme.rs — centralized design tokens
// Brand: #33BCAC (teal) on #000000 (black)

use iced::Color;

// ── Brand colors (from logo) ──────────────────────────────────────────────
pub const TEAL: Color       = Color { r: 0.200, g: 0.737, b: 0.675, a: 1.0 }; // #33BCAC
pub const TEAL_DIM: Color   = Color { r: 0.110, g: 0.420, b: 0.380, a: 1.0 }; // #1C6B61 darker

// ── Background layers (near-black, teal-tinted) ───────────────────────────
pub const BG_BASE: Color    = Color { r: 0.035, g: 0.047, b: 0.047, a: 1.0 }; // #091212
pub const BG_SURFACE: Color = Color { r: 0.060, g: 0.082, b: 0.082, a: 1.0 }; // #0F1515
pub const BG_CARD: Color    = Color { r: 0.086, g: 0.114, b: 0.114, a: 1.0 }; // #161D1D
pub const BG_HOVER: Color   = Color { r: 0.110, g: 0.149, b: 0.149, a: 1.0 }; // #1C2626

// ── Borders ───────────────────────────────────────────────────────────────
pub const BORDER_SUBTLE: Color = Color { r: 0.110, g: 0.200, b: 0.196, a: 1.0 }; // #1C3332
pub const BORDER_MED: Color    = Color { r: 0.160, g: 0.310, b: 0.302, a: 1.0 }; // #294F4D

// ── Text ──────────────────────────────────────────────────────────────────
// High contrast on dark bg — slightly warm white with teal hint
pub const TEXT_PRIMARY: Color   = Color { r: 0.918, g: 0.965, b: 0.957, a: 1.0 }; // #EAF6F4
pub const TEXT_SECONDARY: Color = Color { r: 0.620, g: 0.737, b: 0.718, a: 1.0 }; // #9EBBB7
pub const TEXT_MUTED: Color     = Color { r: 0.380, g: 0.490, b: 0.475, a: 1.0 }; // #617D79

// ── Accent (brand teal used for highlights) ───────────────────────────────
pub const ACCENT: Color     = TEAL;
pub const ACCENT_DIM: Color = TEAL_DIM;

// ── Status ────────────────────────────────────────────────────────────────
// Green stays close to teal; yellow/red are high-contrast for readability
pub const GREEN: Color  = Color { r: 0.200, g: 0.870, b: 0.600, a: 1.0 }; // #33DE99
pub const YELLOW: Color = Color { r: 1.000, g: 0.847, b: 0.200, a: 1.0 }; // #FFD833
pub const RED: Color    = Color { r: 1.000, g: 0.380, b: 0.340, a: 1.0 }; // #FF6157
pub const BLUE: Color   = Color { r: 0.380, g: 0.780, b: 0.870, a: 1.0 }; // #61C7DE
pub const PURPLE: Color = Color { r: 0.620, g: 0.490, b: 0.980, a: 1.0 }; // #9E7DFA

// ── Button colours ────────────────────────────────────────────────────────
pub const BTN_SUCCESS: Color = GREEN;
pub const BTN_DANGER: Color  = RED;
pub const BTN_WARN: Color    = YELLOW;
