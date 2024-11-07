use core::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct RGB(u32);
impl RGB {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r as u32 | (g as u32) << 8 | (b as u32) << 16)
    }

    pub const fn r(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    pub const fn b(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    pub const fn bytes(self) -> [u8; 3] {
        [self.r(), self.g(), self.b()]
    }

    pub const fn tuple(self) -> (u8, u8, u8) {
        (self.r(), self.g(), self.b())
    }
}

impl Display for RGB {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r(), self.g(), self.b())
    }
}
impl From<RGB> for u32 {
    fn from(rgb: RGB) -> Self {
        rgb.0
    }
}

impl Into<RGB> for u32 {
    fn into(self) -> RGB {
        RGB(self)
    }
}

impl Into<RGB> for [u8; 3] {
    fn into(self) -> RGB {
        RGB::new(self[0], self[1], self[2])
    }
}

impl From<RGB> for [u8; 3] {
    fn from(rgb: RGB) -> Self {
        rgb.bytes()
    }
}

impl Into<RGB> for (u8, u8, u8) {
    fn into(self) -> RGB {
        RGB::new(self.0, self.1, self.2)
    }
}

impl From<RGB> for (u8, u8, u8) {
    fn from(rgb: RGB) -> Self {
        rgb.tuple()
    }
}
