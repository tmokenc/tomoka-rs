use super::Damange::*;
// /// Calculate the hidden power
// pub fn hidden_power(iv_stats: Stats) -> Move {

// }

#[inline]
pub fn damage_calculate(
    power: u8,
    atk: u8,
    def: u8,
    level: impl Into<Option<u8>>,
    modifier: DamageModifier,
) -> (Damage, Damage) {
    DamageStats::with_modifier(power, atk, def, level, modifier).calculate()
}
