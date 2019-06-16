pub type Color = u32;

pub fn color_from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    return (r as u32) << 24
        | (g as u32) << 16
        | (b as u32) << 8
        | ((a as u32) & 0xFE);
}
