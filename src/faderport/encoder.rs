#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Encoder {
    Pan,
    Big,
}

impl Encoder {
    pub fn from_byte(id: u8) -> Self {
        match id {
            0x10 => Encoder::Pan,
            0x3C => Encoder::Big,
            _ => unimplemented!("invalid encoder ID: {}", id),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Encoder::Pan => 0x10,
            Encoder::Big => 0x3C,
            _ => unreachable!(),
        }
    }
}
