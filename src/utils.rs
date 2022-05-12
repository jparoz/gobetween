macro_rules! bit14 {
    ($lsb:expr, $msb:expr) => {
        ($msb as u16) << 7 | $lsb as u16
    };

    ($num14:expr) => {
        (($num14 & 0x007F) as u8, (($num14 & 0x3F80) >> 7) as u8)
    };
}
