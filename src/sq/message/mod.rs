use bytes::Buf;

mod id;
use id::ID;

/// Messages sent and received from the SQ object.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Message {
    // @Todo: this shouldn't need to have an ID, but it should be a higher-level object like Fader
    // or Channel or something
    // Level(ID, u16),
    Level(Source, Target, ValueState),
    Mute(Source, ButtonState),
    Pan(Source, Target, ValueState),
    Assign(Source, Target, ButtonState),
}

impl Message {
    pub(super) fn to_nrpn(&self) -> Nrpn {
        use Message::*;
        match self {
            Level(source, target, value_state) | Pan(source, target, value_state) => {
                let id = ID::from_source_target(*source, *target);
                match value_state {
                    ValueState::Set(val) => {
                        let (val_msb, val_lsb) = bit14!(val);
                        Nrpn::Absolute(id, val_msb, val_lsb)
                    }
                    ValueState::Increment => Nrpn::Increment(id),
                    ValueState::Decrement => Nrpn::Decrement(id),
                    ValueState::Get => Nrpn::Get(id),
                }
            }

            Mute(source, button_state) => {
                let id = ID::from_mute(*source);
                match button_state {
                    ButtonState::On => Nrpn::Absolute(id, 0x00, 0x01),
                    ButtonState::Off => Nrpn::Absolute(id, 0x00, 0x00),
                    ButtonState::Toggle => Nrpn::Increment(id),
                    ButtonState::Get => Nrpn::Get(id),
                }
            }

            Assign(source, target, button_state) => {
                let id = ID::from_source_target(*source, *target);
                match button_state {
                    ButtonState::On => Nrpn::Absolute(id, 0x00, 0x01),
                    ButtonState::Off => Nrpn::Absolute(id, 0x00, 0x00),
                    ButtonState::Toggle => Nrpn::Increment(id),
                    ButtonState::Get => Nrpn::Get(id),
                }
            }
        }
    }

    pub(super) fn from_nrpn(nrpn: Nrpn) -> Self {
        use Nrpn::*;
        match nrpn {
            Absolute(id, coarse, fine) => id.absolute(coarse, fine),
            Increment(id) => id.increment(),
            Decrement(id) => id.decrement(),
            Get(id) => id.get(),
        }
    }
}

/// A Source is anything that can be assigned, have its level or pan changed, or muted.
/// Most operations will require both a Source and a Target; mutes only require a Source, as they
/// are shared between Targets.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Source {
    /// Valid values are Input(1..=48)
    Input(u8),

    /// Valid values are Group(1..=12)
    Group(u8),

    /// Valid values are FXRet(1..=8)
    FXRet(u8),

    LR,

    /// Valid values are Aux(1..=12)
    Aux(u8),

    /// Valid values are FXSend(1..=4)
    FXSend(u8),

    /// Valid values are Mtx(1..=3)
    Mtx(u8),
    // @Todo: DCA
    // @Todo: Mute groups
}

/// A Target is anything that can have Sources assigned to it, or have their level or pan changed.
/// Most operations will require both a Source and a Target.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    LR,

    /// Valid values are Aux(1..=12)
    Aux(u8),

    /// Valid values are Group(1..=12)
    Group(u8),

    /// Valid values are FXSend(1..=4)
    FXSend(u8),

    /// Valid values are Mtx(1..=3)
    Mtx(u8),

    Output,
    // @Todo: Control (for DCA groups) (or maybe can reuse Output? only if there's no ambiguity)
}

/// Used for updating the state of either a mute or an assignment.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    On,
    Off,
    Toggle,
    Get,
}

/// Used for updating the state of either a level or a pan.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueState {
    Set(u16),
    Increment,
    Decrement,
    Get,
}

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
