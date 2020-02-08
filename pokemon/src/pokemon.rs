use super::{Nature, Abilitiy, Moves, Item, Stats};
/// A single pokemon data
pub struct Pokemon<'a> {
    iv: Stats,
    ev: Stats,
    level: u8,
    current_hp: u8,
    happiness: u8,
    nature: Nature,
    ability: Ability,
    item: Option<Item>,
    types: PokemonType,
    moveset: &'a [Option<Move>; 4],
}

pub enum PokemonType {
    Pure(Type),
    Dual(Type, Type),
}

