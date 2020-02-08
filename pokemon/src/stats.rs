pub struct Stats {
    hp: u8,
    atk: u8,
    def: u8,
    sp_atk: u8,
    sp_def: u8,
    speed: u8,
}

pub enum NatureModify {
    Positive,
    Neutral,
    Negative,
}

impl Default for NatureModify {
    fn default() -> Self {
        Self::Neutral
    }
}

/// ``(((2*Base + IV + EV/4) * Level) / 100 + 5) * Nature``
pub fn stat_cal(base: u8, iv: u8, ev: u8, level: u8, nature: NatureModify) -> u8 {
    let basic = (2 * base as u16) + iv as u16 + (ev / 4) as u16;
    let total = (basic * level as u16) / 100 + 5;
    match nature {
        NatureModify::Positive => (total as f32 * 1.1) as u8,
        NatureModify::Negative => (total as f32 * 0.9) as u8,
        NatureModify::Neutral => total as u8,
    }
}

/// The HP is calculating different from other stats
/// ``((2*Base + IV + EV/4 + 100) * Level) / 100 + 10``
pub fn hp_cal(base: u8, iv: u8, ev: u8, level: u8) -> u8 {
    let basic = (2 * base as u16) + iv as u16 + (ev / 4) as u16 + 100;
    let total = (basic * level as u16) / 100 + 10;
    total as u8
}
