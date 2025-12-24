pub const RECOMMENDED: u8 = 0b10000;
pub const ADVANCED: u8 = 0b01000;
pub const EXPERT: u8 = 0b00100;
pub const UNSAFE: u8 = 0b00010;
pub const UNIDENTIFIED: u8 = 0b00001;

pub const VALUES: [u8; 5] = [RECOMMENDED, ADVANCED, EXPERT, UNSAFE, UNIDENTIFIED];
pub const NAMES: [&str; 5] = [
    "Recommended",
    "Advanced",
    "Expert",
    "Unsafe",
    "Unidentified",
];

pub fn value_to_name(value: u8) -> &'static str {
    match value {
        RECOMMENDED => "Recommended",
        ADVANCED => "Advanced",
        EXPERT => "Expert",
        UNSAFE => "Unsafe",
        _ => "Unidentified",
    }
}
