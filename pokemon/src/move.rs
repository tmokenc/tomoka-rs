use super::Type;

pub struct SerMove {
    pub name: String,
    pub accuracy: Option<u8>,
    pub power: Option<u8>,
    pub pp: u8,
    pub priority: u8,
    pub stat_changes: Vec<SerStatChange>,
}

pub struct SerInfo {
    pub name: String,
}

pub struct SerMeta {
    pub category: SerInfo,
}

pub struct SerStatChange {
    pub change: i8,
    pub stat: SerInfo,
}

pub struct Move {
    pub name: String,
    pub move_type: Type,
    pub power: u8,
    pub target: MoveTarget,
    pub accuracy: MoveAccuracy,
    pub category: MoveCategory,
}

pub enum MoveTarget {
    User,
    Teammate,
    Opponent,
    OppenentField,
    AllField,
}

pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

