use bytes::Buf;

/// Messages sent and received from the SQ object.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Message {
    // @Todo: this shouldn't need to have an ID, but it should be a higher-level object like Fader
    // or Channel or something
    Level(ID, u16),
}

impl Message {
    pub(super) fn to_nrpn(&self) -> Nrpn {
        use Message::*;
        match self {
            Level(id, val) => {
                let (val_lsb, val_msb) = bit14!(val);
                Nrpn::Absolute(*id, val_msb, val_lsb)
            }
        }
    }

    pub(super) fn from_nrpn(nrpn: Nrpn) -> Self {
        use Message::*;
        use Nrpn::*;
        match nrpn {
            Absolute(id @ ID(msb, lsb), coarse, fine) => {
                // @Todo @Fixme @XXX: check that this is a level ID
                Level(id, bit14!(fine, coarse))
            }
            _ => todo!(),
        }
    }
}

/// ID(msb, lsb)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ID(u8, u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Nrpn {
    Absolute(ID, u8, u8),
    Increment(ID),
    Decrement(ID),
    Get(ID),
}

impl Nrpn {
    pub fn to_bytes(&self) -> Vec<u8> {
        use Nrpn::*;
        match self {
            Absolute(ID(msb, lsb), coarse, fine) => vec![
                0xB0, 0x63, *msb, 0xB0, 0x62, *lsb, 0xB0, 0x06, *coarse, 0xB0, 0x26, *fine,
            ],
            Increment(ID(msb, lsb)) => vec![0xB0, 0x63, *msb, 0xB0, 0x62, *lsb, 0xB0, 0x60, 0x00],
            Decrement(ID(msb, lsb)) => vec![0xB0, 0x63, *msb, 0xB0, 0x62, *lsb, 0xB0, 0x61, 0x00],
            Get(ID(msb, lsb)) => vec![0xB0, 0x63, *msb, 0xB0, 0x62, *lsb, 0xB0, 0x60, 0x7F],
        }
    }

    /// Tries to parse an NRPN message from the given buffer.
    /// If it succeeds, the buffer's cursor will be advanced past the parsed bytes.
    /// If it fails, and returns None, the buffer will be unchanged.
    pub fn from_buf<T: Buf>(buf: &mut T) -> Option<Self> {
        use Nrpn::*;
        match buf.chunk() {
            &[0xB0, 0x63, msb, 0xB0, 0x62, lsb, 0xB0, 0x06, coarse, 0xB0, 0x26, fine, ..] => {
                buf.advance(12);
                Some(Absolute(ID(msb, lsb), coarse, fine))
            }
            &[0xB0, 0x63, msb, 0xB0, 0x62, lsb, 0xB0, 0x60, 0x00, ..] => {
                buf.advance(9);
                Some(Increment(ID(msb, lsb)))
            }
            &[0xB0, 0x63, msb, 0xB0, 0x62, lsb, 0xB0, 0x61, 0x00, ..] => {
                buf.advance(9);
                Some(Decrement(ID(msb, lsb)))
            }
            &[0xB0, 0x63, msb, 0xB0, 0x62, lsb, 0xB0, 0x60, 0x7F, ..] => {
                buf.advance(9);
                Some(Get(ID(msb, lsb)))
            }
            _ => None,
        }
    }
}
