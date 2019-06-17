pub type Color = u32;

pub fn color_from_rgba(r: u8, g: u8, b: u8) -> Color {
    return (r as u32) << 16 | (g as u32) << 8 | (b as u32);
}
