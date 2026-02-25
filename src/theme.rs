// src/theme.rs — Apple-inspired design tokens

use iced::Color;

// ── Brand ─────────────────────────────────────────────────────────────────
pub const TEAL: Color = Color {
    r: 0.196,
    g: 0.780,
    b: 0.706,
    a: 1.0,
};
pub const TEAL_DIM: Color = Color {
    r: 0.118,
    g: 0.447,
    b: 0.404,
    a: 1.0,
};

// ── Backgrounds ───────────────────────────────────────────────────────────
pub const BG_BASE: Color = Color {
    r: 0.067,
    g: 0.067,
    b: 0.071,
    a: 1.0,
};
pub const BG_SURFACE: Color = Color {
    r: 0.106,
    g: 0.106,
    b: 0.114,
    a: 1.0,
};
pub const BG_CARD: Color = Color {
    r: 0.141,
    g: 0.141,
    b: 0.149,
    a: 1.0,
};
pub const BG_HOVER: Color = Color {
    r: 0.196,
    g: 0.196,
    b: 0.208,
    a: 1.0,
};
pub const BG_ELEVATED: Color = Color {
    r: 0.212,
    g: 0.212,
    b: 0.224,
    a: 1.0,
};

// ── Borders — solid opaque colors, no alpha hacks ────────────────────────
pub const BORDER_SUBTLE: Color = Color {
    r: 0.220,
    g: 0.220,
    b: 0.235,
    a: 1.0,
};
pub const BORDER_MED: Color = Color {
    r: 0.290,
    g: 0.290,
    b: 0.310,
    a: 1.0,
};

//pub const BORDER_FOCUS: Color  = TEAL;

// ── Text ──────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color = Color {
    r: 0.980,
    g: 0.980,
    b: 0.980,
    a: 1.0,
};
pub const TEXT_SECONDARY: Color = Color {
    r: 0.620,
    g: 0.620,
    b: 0.640,
    a: 1.0,
};
pub const TEXT_MUTED: Color = Color {
    r: 0.420,
    g: 0.420,
    b: 0.440,
    a: 1.0,
};

//pub const TEXT_TEAL: Color      = TEAL;

// ── Accent ────────────────────────────────────────────────────────────────
pub const ACCENT: Color = TEAL;
pub const ACCENT_DIM: Color = TEAL_DIM;

// ── Status colors ─────────────────────────────────────────────────────────
pub const GREEN: Color = Color {
    r: 0.188,
    g: 0.820,
    b: 0.498,
    a: 1.0,
};
pub const YELLOW: Color = Color {
    r: 1.000,
    g: 0.800,
    b: 0.000,
    a: 1.0,
};
pub const RED: Color = Color {
    r: 1.000,
    g: 0.271,
    b: 0.227,
    a: 1.0,
};
pub const BLUE: Color = Color {
    r: 0.039,
    g: 0.518,
    b: 1.000,
    a: 1.0,
};
pub const PURPLE: Color = Color {
    r: 0.749,
    g: 0.353,
    b: 0.949,
    a: 1.0,
};
pub const ORANGE: Color = Color {
    r: 1.000,
    g: 0.620,
    b: 0.039,
    a: 1.0,
};

// ── Button colors ─────────────────────────────────────────────────────────
pub const BTN_SUCCESS: Color = GREEN;
pub const BTN_DANGER: Color = RED;
pub const BTN_WARN: Color = ORANGE;
