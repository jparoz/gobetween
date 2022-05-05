#[derive(Debug, Clone, Copy)]
pub enum Button {
    // Channel strip controls
    Solo(u8),
    Mute(u8),
    Select(u8),
    FaderTouch(u8),

    // General controls (left side)
    PanEncoder,
    Arm,
    SoloClear,
    MuteClear,
    Bypass,
    Macro,
    Link,
    LeftShift,

    // Fader mode buttons
    Track,
    EditPlugins,
    Sends,
    Pan,

    // Session navigator
    Prev,
    BigEncoder,
    Next,
    Channel,
    Zoom,
    Scroll,
    Bank,
    Master,
    Click,
    Section,
    Marker,

    // Mix management
    Audio,
    VI,
    Bus,
    Vca,
    All,
    RightShift,

    // Automation
    Read,
    Write,
    Trim,
    Touch,
    Latch,
    Off,

    // Transport
    Loop,
    Rewind,
    FastForward,
    Stop,
    Play,
    Record,
    Footswitch,
}

impl Button {
    pub fn new(id: u8) -> Self {
        use Button::*;

        match id {
            // Channel strip controls
            // Solo
            0x08 => Solo(1),
            0x09 => Solo(2),
            0x0a => Solo(3),
            0x0b => Solo(4),

            0x0c => Solo(5),
            0x0d => Solo(6),
            0x0e => Solo(7),
            0x0f => Solo(8),

            0x50 => Solo(9),
            0x51 => Solo(10),
            0x52 => Solo(11),
            0x58 => Solo(12),

            0x54 => Solo(13),
            0x55 => Solo(14),
            0x59 => Solo(15),
            0x57 => Solo(16),

            // Mute
            0x10 => Mute(1),
            0x11 => Mute(2),
            0x12 => Mute(3),
            0x13 => Mute(4),

            0x14 => Mute(5),
            0x15 => Mute(6),
            0x16 => Mute(7),
            0x17 => Mute(8),

            0x78 => Mute(9),
            0x79 => Mute(10),
            0x7a => Mute(11),
            0x7b => Mute(12),

            0x7c => Mute(13),
            0x7d => Mute(14),
            0x7e => Mute(15),
            0x7f => Mute(16),

            // Select
            0x18 => Select(1),
            0x19 => Select(2),
            0x1a => Select(3),
            0x1b => Select(4),

            0x1c => Select(5),
            0x1d => Select(6),
            0x1e => Select(7),
            0x1f => Select(8),

            0x07 => Select(9),
            0x21 => Select(10),
            0x22 => Select(11),
            0x23 => Select(12),

            0x24 => Select(13),
            0x25 => Select(14),
            0x26 => Select(15),
            0x27 => Select(16),

            // Fader touch
            0x68 => FaderTouch(1),
            0x69 => FaderTouch(2),
            0x6a => FaderTouch(3),
            0x6b => FaderTouch(4),

            0x6c => FaderTouch(5),
            0x6d => FaderTouch(6),
            0x6e => FaderTouch(7),
            0x6f => FaderTouch(8),

            0x70 => FaderTouch(9),
            0x71 => FaderTouch(10),
            0x72 => FaderTouch(11),
            0x73 => FaderTouch(12),

            0x74 => FaderTouch(13),
            0x75 => FaderTouch(14),
            0x76 => FaderTouch(15),
            0x77 => FaderTouch(16),

            // General controls (left side)
            0x20 => PanEncoder,
            0x00 => Arm,
            0x01 => SoloClear,
            0x02 => MuteClear,
            0x03 => Bypass,
            0x04 => Macro,
            0x05 => Link,
            0x06 => LeftShift,

            // Fader mode buttons
            0x28 => Track,
            0x2b => EditPlugins,
            0x29 => Sends,
            0x2a => Pan,

            // Session navigator
            0x2e => Prev,
            0x53 => BigEncoder,
            0x2f => Next,
            0x36 => Channel,
            0x37 => Zoom,
            0x38 => Scroll,
            0x39 => Bank,
            0x3a => Master,
            0x3b => Click,
            0x3c => Section,
            0x3d => Marker,

            // Mix management
            0x3e => Audio,
            0x3f => VI,
            0x40 => Bus,
            0x41 => Vca,
            0x42 => All,
            0x46 => RightShift,

            // Automation
            0x4a => Read,
            0x4b => Write,
            0x4c => Trim,
            0x4d => Touch,
            0x4e => Latch,
            0x4f => Off,

            // Transport
            0x56 => Loop,
            0x5b => Rewind,
            0x5c => FastForward,
            0x5d => Stop,
            0x5e => Play,
            0x5f => Record,
            0x66 => Footswitch,

            _ => unimplemented!("invalid button ID: {}", id),
        }
    }
}
