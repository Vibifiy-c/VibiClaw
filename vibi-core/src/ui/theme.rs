use egui::Color32;

#[derive(Clone)]
pub struct Theme {
    pub dark: bool,
}

// Light mode colors
pub mod light {
    use egui::Color32;
    pub const BG: Color32              = Color32::from_rgb(0xfa, 0xf9, 0xff);
    pub const BG_SECONDARY: Color32    = Color32::from_rgb(0xf2, 0xe8, 0xf7);
    pub const SIDEBAR_BG: Color32      = Color32::from_rgb(0xed, 0xe0, 0xf5);
    pub const SURFACE: Color32         = Color32::from_rgb(0xff, 0xff, 0xff);
    pub const SURFACE2: Color32        = Color32::from_rgb(0xf2, 0xe8, 0xf7);
    pub const BORDER: Color32          = Color32::from_rgb(0xdd, 0xc8, 0xeb);
    pub const TEXT: Color32            = Color32::from_rgb(0x1a, 0x15, 0x23);
    pub const TEXT_SECONDARY: Color32  = Color32::from_rgb(0x6b, 0x65, 0x80);
    pub const TEXT_MUTED: Color32      = Color32::from_rgb(0xa0, 0x9b, 0xb8);
    pub const ACCENT: Color32          = Color32::from_rgb(0x8B, 0x00, 0xFF);
    pub const ACCENT_LIGHT: Color32    = Color32::from_rgb(0xf0, 0xe0, 0xff);
    pub const ACCENT_HOVER: Color32    = Color32::from_rgb(0x74, 0x00, 0xcc);
    pub const USER_BUBBLE: Color32     = Color32::from_rgb(0xf2, 0xe8, 0xf7);
    pub const USER_TEXT: Color32       = Color32::from_rgb(0x1a, 0x15, 0x23);
    pub const AI_BUBBLE: Color32       = Color32::TRANSPARENT;
    pub const AI_TEXT: Color32         = Color32::from_rgb(0x1a, 0x15, 0x23);
    pub const CODE_BG: Color32         = Color32::from_rgb(0xf5, 0xf3, 0xff);
    pub const SUCCESS: Color32         = Color32::from_rgb(0x1a, 0x7a, 0x1a);
    pub const SUCCESS_BG: Color32      = Color32::from_rgb(0xe6, 0xff, 0xe6);
    pub const SUCCESS_BORDER: Color32  = Color32::from_rgb(0x4c, 0xaf, 0x50);
}

// Dark mode colors
pub mod dark {
    use egui::Color32;
    pub const BG: Color32              = Color32::from_rgb(0x21, 0x21, 0x21);
    pub const BG_SECONDARY: Color32    = Color32::from_rgb(0x2f, 0x2f, 0x2f);
    pub const SIDEBAR_BG: Color32      = Color32::from_rgb(0x17, 0x17, 0x17);
    pub const SURFACE: Color32         = Color32::from_rgb(0x2f, 0x2f, 0x2f);
    pub const SURFACE2: Color32        = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    pub const BORDER: Color32          = Color32::from_rgb(0x3f, 0x3f, 0x3f);
    pub const TEXT: Color32            = Color32::from_rgb(0xf0, 0xee, 0xff);
    pub const TEXT_SECONDARY: Color32  = Color32::from_rgb(0x9d, 0x94, 0xc4);
    pub const TEXT_MUTED: Color32      = Color32::from_rgb(0x5c, 0x54, 0x80);
    pub const ACCENT: Color32          = Color32::from_rgb(0xa0, 0x20, 0xf0);
    pub const ACCENT_LIGHT: Color32    = Color32::from_rgb(0x3d, 0x2b, 0x45);
    pub const ACCENT_HOVER: Color32    = Color32::from_rgb(0xe8, 0xd5, 0xf0);
    pub const USER_BUBBLE: Color32     = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    pub const USER_TEXT: Color32       = Color32::from_rgb(0xf0, 0xee, 0xff);
    pub const AI_BUBBLE: Color32       = Color32::TRANSPARENT;
    pub const AI_TEXT: Color32         = Color32::from_rgb(0xf0, 0xee, 0xff);
    pub const CODE_BG: Color32         = Color32::from_rgb(0x1a, 0x17, 0x2a);
    pub const SUCCESS: Color32         = Color32::from_rgb(0x4c, 0xaf, 0x50);
    pub const SUCCESS_BG: Color32      = Color32::from_rgb(0x1a, 0x2e, 0x1a);
    pub const SUCCESS_BORDER: Color32  = Color32::from_rgb(0x2e, 0x5e, 0x2e);
}

impl Theme {
    pub fn new(dark: bool) -> Self {
        Self { dark }
    }

    pub fn bg(&self) -> Color32 {
        if self.dark { dark::BG } else { light::BG }
    }
    pub fn bg_secondary(&self) -> Color32 {
        if self.dark { dark::BG_SECONDARY } else { light::BG_SECONDARY }
    }
    pub fn sidebar_bg(&self) -> Color32 {
        if self.dark { dark::SIDEBAR_BG } else { light::SIDEBAR_BG }
    }
    pub fn surface(&self) -> Color32 {
        if self.dark { dark::SURFACE } else { light::SURFACE }
    }
    pub fn surface2(&self) -> Color32 {
        if self.dark { dark::SURFACE2 } else { light::SURFACE2 }
    }
    pub fn border(&self) -> Color32 {
        if self.dark { dark::BORDER } else { light::BORDER }
    }
    pub fn text(&self) -> Color32 {
        if self.dark { dark::TEXT } else { light::TEXT }
    }
    pub fn text_secondary(&self) -> Color32 {
        if self.dark { dark::TEXT_SECONDARY } else { light::TEXT_SECONDARY }
    }
    pub fn text_muted(&self) -> Color32 {
        if self.dark { dark::TEXT_MUTED } else { light::TEXT_MUTED }
    }
    pub fn accent(&self) -> Color32 {
        if self.dark { dark::ACCENT } else { light::ACCENT }
    }
    pub fn accent_light(&self) -> Color32 {
        if self.dark { dark::ACCENT_LIGHT } else { light::ACCENT_LIGHT }
    }
    pub fn accent_hover(&self) -> Color32 {
        if self.dark { dark::ACCENT_HOVER } else { light::ACCENT_HOVER }
    }
    pub fn user_bubble(&self) -> Color32 {
        if self.dark { dark::USER_BUBBLE } else { light::USER_BUBBLE }
    }
    pub fn user_text(&self) -> Color32 {
        if self.dark { dark::USER_TEXT } else { light::USER_TEXT }
    }
    pub fn ai_text(&self) -> Color32 {
        if self.dark { dark::AI_TEXT } else { light::AI_TEXT }
    }
    pub fn code_bg(&self) -> Color32 {
        if self.dark { dark::CODE_BG } else { light::CODE_BG }
    }
    pub fn success(&self) -> Color32 {
        if self.dark { dark::SUCCESS } else { light::SUCCESS }
    }
    pub fn success_bg(&self) -> Color32 {
        if self.dark { dark::SUCCESS_BG } else { light::SUCCESS_BG }
    }
    pub fn success_border(&self) -> Color32 {
        if self.dark { dark::SUCCESS_BORDER } else { light::SUCCESS_BORDER }
    }
}

// Custom cursor painter — draws the "Vi" cursor shape
// Call this every frame with the current mouse position
pub fn draw_cursor(painter: &egui::Painter, pos: egui::Pos2, color: Color32) {
    let stroke = egui::Stroke::new(1.5, color);

    // The V shape — tall slanted arrow-like V
    let v_top   = egui::pos2(pos.x + 4.0,  pos.y);
    let v_left  = egui::pos2(pos.x,        pos.y + 18.0);
    let v_mid   = egui::pos2(pos.x + 5.0,  pos.y + 13.0);
    let v_right = egui::pos2(pos.x + 10.0, pos.y + 18.0);

    painter.line_segment([v_top, v_left], stroke);
    painter.line_segment([v_top, v_right], stroke);
    painter.line_segment([v_left, v_mid], stroke);
    painter.line_segment([v_right, v_mid], stroke);

    // The i shape — sits to the right and slightly below
    let i_x       = pos.x + 14.0;
    let i_top_y   = pos.y + 7.0;
    let i_bot_y   = pos.y + 17.0;
    let i_dot_y   = pos.y + 4.0;

    // i stem
    painter.line_segment(
        [egui::pos2(i_x, i_top_y), egui::pos2(i_x, i_bot_y)],
        stroke,
    );
    // i dot
    painter.circle_filled(egui::pos2(i_x, i_dot_y), 1.5, color);
}