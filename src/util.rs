pub fn spaces(size: u8) -> &'static str {
    const SPACES: &str = "                                                                                                                                                                                                                                                                ";
    &SPACES[..size as usize]
}

pub fn num_digits(i: usize) -> u8 {
    f64::log10(i as f64) as u8 + 1
}
