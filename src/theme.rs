// src/theme.rs — DevPanel design tokens
// Matches the index.php web welcome page palette exactly.
//
// index.php CSS variables:
//   --bg:         #0a0a0a   BG_BASE
//   --surface:    #111111   BG_SURFACE
//   --surface-2:  #161616   BG_CARD
//   --surface-3:  #1c1c1c   BG_HOVER / BG_ELEVATED
//   --border:     rgba(255,255,255,.07)  → opaque approx ~#181818
//   --border-mid: rgba(255,255,255,.11)  → opaque approx ~#1e1e1e
//   --text:       #f0f0f0   TEXT_PRIMARY
//   --text-2:     #a0a0a0   TEXT_SECONDARY
//   --text-3:     #555555   TEXT_MUTED
//   --green:      #30d158   GREEN  (Apple system green)
//   --red:        #ff453a   RED    (Apple system red)
//   --yellow:     #ffd60a   YELLOW (Apple system yellow)
//   --blue:       #0a84ff   BLUE   (Apple system blue)

#![allow(dead_code)]

use iced::Color;

// ── Brand / Accent ────────────────────────────────────────────────────────
// Primary accent = Apple system green (#30d158) — matches index.php --green
pub const GREEN: Color = Color { r: 0.188, g: 0.820, b: 0.345, a: 1.0 };

// Aliases so existing TEAL / ACCENT references keep compiling.
pub const TEAL: Color = GREEN;
pub const ACCENT:     Color = GREEN;
pub const ACCENT_DIM: Color = Color { r: 0.080, g: 0.250, b: 0.120, a: 1.0 };

// ── Backgrounds ───────────────────────────────────────────────────────────
pub const BG_BASE: Color     = Color { r: 0.039, g: 0.039, b: 0.039, a: 1.0 }; // #0a0a0a
pub const BG_SURFACE: Color  = Color { r: 0.067, g: 0.067, b: 0.067, a: 1.0 }; // #111111
pub const BG_CARD: Color     = Color { r: 0.086, g: 0.086, b: 0.086, a: 1.0 }; // #161616
pub const BG_HOVER: Color    = Color { r: 0.110, g: 0.110, b: 0.110, a: 1.0 }; // #1c1c1c
pub const BG_ELEVATED: Color = Color { r: 0.133, g: 0.133, b: 0.133, a: 1.0 }; // #222222

// ── Borders ───────────────────────────────────────────────────────────────
pub const BORDER_SUBTLE: Color = Color { r: 0.125, g: 0.125, b: 0.125, a: 1.0 }; // rgba(255,255,255,.07) opaque equiv
pub const BORDER_MED: Color    = Color { r: 0.165, g: 0.165, b: 0.165, a: 1.0 }; // rgba(255,255,255,.11) opaque equiv

// ── Text ──────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY:   Color = Color { r: 0.941, g: 0.941, b: 0.941, a: 1.0 }; // #f0f0f0
pub const TEXT_SECONDARY: Color = Color { r: 0.627, g: 0.627, b: 0.627, a: 1.0 }; // #a0a0a0
pub const TEXT_MUTED:     Color = Color { r: 0.333, g: 0.333, b: 0.333, a: 1.0 }; // #555555

// ── Status colors ─────────────────────────────────────────────────────────
pub const RED:    Color = Color { r: 1.000, g: 0.271, b: 0.227, a: 1.0 }; // #ff453a
pub const YELLOW: Color = Color { r: 1.000, g: 0.839, b: 0.039, a: 1.0 }; // #ffd60a
pub const BLUE:   Color = Color { r: 0.039, g: 0.518, b: 1.000, a: 1.0 }; // #0a84ff
pub const PURPLE: Color = Color { r: 0.749, g: 0.353, b: 0.949, a: 1.0 }; // kept for compat
pub const ORANGE: Color = Color { r: 1.000, g: 0.620, b: 0.039, a: 1.0 }; // kept for BTN_WARN

// ── Dim background tints (match index.php *-dim vars) ─────────────────────
// rgba(48,209,88,.12) on #0a0a0a
pub const GREEN_DIM:  Color = Color { r: 0.071, g: 0.122, b: 0.082, a: 1.0 };
// rgba(255,69,58,.10) on #0a0a0a
pub const RED_DIM:    Color = Color { r: 0.137, g: 0.071, b: 0.067, a: 1.0 };
// rgba(255,214,10,.10) on #0a0a0a
pub const YELLOW_DIM: Color = Color { r: 0.137, g: 0.122, b: 0.043, a: 1.0 };
// rgba(10,132,255,.10) on #0a0a0a
pub const BLUE_DIM:   Color = Color { r: 0.047, g: 0.090, b: 0.157, a: 1.0 };

// Legacy alias
pub const TEAL_DIM: Color = GREEN_DIM;

// ── Buttons ───────────────────────────────────────────────────────────────
pub const BTN_SUCCESS: Color = GREEN;
pub const BTN_DANGER:  Color = RED;
pub const BTN_WARN:    Color = ORANGE;
