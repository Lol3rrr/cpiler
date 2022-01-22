use std::fmt::Display;

pub struct Imm9(u16);
#[derive(Debug, PartialEq, Clone)]
pub struct Imm9Signed(i16);

impl Imm9Signed {
    pub fn new(val: i16) -> Self {
        if val > 255 || val < -256 {
            panic!("Attempted to construct Signed 9-Bit immediate: {}", val)
        }

        Self(val)
    }

    pub fn fits(value: i16) -> bool {
        value <= 255 && value >= -256
    }
}
impl Display for Imm9Signed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FloatImm8(f32);

impl FloatImm8 {
    pub fn new(val: f32) -> Self {
        Self(val)
    }
}
impl Display for FloatImm8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
