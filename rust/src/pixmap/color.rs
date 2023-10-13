use anyhow::anyhow;
use std::fmt::{Formatter, UpperHex};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// Color data represented as red, green, and blue channels each having a depth of 8 bits
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<[u8; 3]> for Color {
    fn from(data: [u8; 3]) -> Self {
        Self(data[0], data[1], data[2])
    }
}

impl From<Color> for [u8; 3] {
    fn from(value: Color) -> Self {
        [value.0, value.1, value.2]
    }
}

impl From<u32> for Color {
    fn from(src: u32) -> Self {
        let b = src.to_be_bytes();
        Self(b[0], b[1], b[2])
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        (value.0 as u32) << 24 | (value.1 as u32) << 16 | (value.2 as u32) << 8
    }
}

impl TryFrom<&[u8]> for Color {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.len() {
            3 => Ok(Self(value[0], value[1], value[2])),
            _ => Err(anyhow!(
                "cannot convert slices of more or less than three elements to color"
            )),
        }
    }
}

impl From<Color> for Vec<u8> {
    fn from(value: Color) -> Self {
        vec![value.0, value.1, value.2]
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

impl UpperHex for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // format each byte as hex string with at least two characters and leading zeroes
        f.write_fmt(format_args!("{:02X}{:02X}{:02X}", self.0, self.1, self.2))
    }
}

#[cfg(test)]
impl Arbitrary for Color {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        u32::arbitrary(g).into()
    }
}

#[cfg(test)]
quickcheck! {
    fn test_u32_conversion(color: Color) -> bool {
        let c_enc: u32 = color.into();
        let c_dec: Color = Color::from(c_enc);
        c_dec == color
    }
}
