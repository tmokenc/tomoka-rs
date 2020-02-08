use super::Pokemon;
use super::Trainer;

pub struct Field {
    terrain: Option<Terrain>,
    weather: Option<Weather>,
    weather_count: u8,
    terrain_count: u8,
    p1: PlayerField,
    p2: PlayerField,
}

pub struct PlayerField {
    pub trap: Option<Trap>,
    pub trainer: Trainer,
    pub pokemon: PlayerPokemon,

}

pub enum PokemonOnField {
    Single(Pokemon),
    Double(Pokemom, Pokemon),
}

pub enum Weather {
    Rain,
    HashSunlight,
    Haze,
    DeltaStream,
}

pub enum Terrain {
    Grass,
    Misty,
    Electric,
    Psychic
}

/// Trap with the level
pub enum Trap {
    Spike(u8),
    ToxicSpike(u8),
}
