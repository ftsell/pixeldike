use std::fmt::{Formatter, UpperHex};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// Color data represented as red, green, and blue channels each having a depth of 8 bits
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<u32> for Color {
    fn from(src: u32) -> Self {
        let b = src.to_le_bytes();
        Self(b[0], b[1], b[2])
    }
}

impl From<[u8; 3]> for Color {
    fn from(data: [u8; 3]) -> Self {
        Self(data[0], data[1], data[2])
    }
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        0u32 | (self.0 as u32) | (self.1 as u32) << 8 | (self.2 as u32) << 16
    }
}

impl Into<u32> for &Color {
    fn into(self) -> u32 {
        0u32 | (self.0 as u32) | (self.1 as u32) << 8 | (self.2 as u32) << 16
    }
}

impl Into<Vec<u8>> for Color {
    fn into(self) -> Vec<u8> {
        vec![self.0, self.1, self.2]
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
