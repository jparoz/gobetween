macro_rules! bit14 {
    ($msb:expr, $lsb:expr) => {
        ($msb as u16) << 7 | $lsb as u16
    };
}
