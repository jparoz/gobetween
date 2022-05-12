macro_rules! bit14 {
    ($lsb:expr, $msb:expr) => {
        ($msb as u16) << 7 | $lsb as u16
    };
}
