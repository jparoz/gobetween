#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Fader(u8);

impl Fader {
    pub fn from_byte(id: u8) -> Self {
        match id {
            0x68 => Fader(1),
            0x69 => Fader(2),
            0x6a => Fader(3),
            0x6b => Fader(4),

            0x6c => Fader(5),
            0x6d => Fader(6),
            0x6e => Fader(7),
            0x6f => Fader(8),

            0x70 => Fader(9),
            0x71 => Fader(10),
            0x72 => Fader(11),
            0x73 => Fader(12),

            0x74 => Fader(13),
            0x75 => Fader(14),
            0x76 => Fader(15),
            0x77 => Fader(16),

            _ => unimplemented!("invalid fader ID: {}", id),
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Fader(1) => 0x68,
            Fader(2) => 0x69,
            Fader(3) => 0x6a,
            Fader(4) => 0x6b,

            Fader(5) => 0x6c,
            Fader(6) => 0x6d,
            Fader(7) => 0x6e,
            Fader(8) => 0x6f,

            Fader(9) => 0x70,
            Fader(10) => 0x71,
            Fader(11) => 0x72,
            Fader(12) => 0x73,

            Fader(13) => 0x74,
            Fader(14) => 0x75,
            Fader(15) => 0x76,
            Fader(16) => 0x77,

            Fader(_) => unreachable!(),
        }
    }
}
