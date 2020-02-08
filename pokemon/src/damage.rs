type Damage = u16;
type MoveAccuracy = Option<u8>;

const MIN_DAMAGE_MODIFY: f32 = 0.85;

pub struct DamageStats {
    atk: u8,
    def: u8,
    level: u8,
    power: u8,
    modifier: DamageModifier,
}

#[derive(Default)]
pub struct DamageModifier {
    /// If it's STAB move
    stab: bool,
    /// If it's a critical hit
    critical: bool,
    /// If the Light Screen / Reflect is activating
    screen: bool,
    /// If using a Choice Specs/ Choice Band
    choiced: bool,
    /// A single target or a multiple target move
    target: MoveTarget,
    /// Type effectiveness
    effectiveness: TypeEffective,
}

#[derive(PartialEq)]
pub enum MoveTarget {
    Single,
    Multi,
}
pub enum TypeEffective {
    NotEffective,
    NotVeryEffective,
    Effective,
    SuperEffective,
    ExtremelyEffective,
}

impl DamageStats {
    pub fn new(power: u8, atk: u8, def: u8, level: impl Into<Option<u8>>) -> Self {
        Self {
            power,
            atk,
            def,
            level: level.into().unwrap_or(50),
            modifier: Default::default(),
        }
    }

    pub fn with_modifier(
        power: u8,
        atk: u8,
        def: u8,
        level: impl Into<Option<u8>>,
        modifier: DamageModifier,
    ) -> Self {
        Self {
            power,
            atk,
            def,
            modifier,
            level: level.into().unwrap_or(50),
        }
    }

    // (((level * 2 / 5) + 2) * power * atk/def) / 50 + 2
    fn cal_basic(&self) -> Damage {
        let level_dmg = (self.level as u16 * 2 / 5) + 2;
        let basic = level_dmg * self.power as u16 * (self.atk / self.def) as u16 / 50;
        basic + 2
    }

    fn cal(&self) -> Damage {
        let basic = self.cal_basic();
        let res = basic as f32 * self.modifier.calculate();
        res as Damage
    }

    pub fn calculate(&self) -> (Damage, Damage) {
        let max_dmg = self.cal();
        let min_dmg = (max_dmg as f32 * MIN_DAMAGE_MODIFY) as Damage;
        (min_dmg, max_dmg)
    }

    pub fn min_damage(&self) -> Damage {
        let min_dmg = self.cal() as f32 * MIN_DAMAGE_MODIFY;
        min_dmg as Damage
    }

    #[inline]
    pub fn max_damage(&self) -> Damage {
        self.cal()
    }
}

impl Default for MoveTarget {
    fn default() -> Self {
        Self::Single
    }
}

impl Default for TypeEffective {
    fn default() -> Self {
        Self::Effective
    }
}

impl DamageModifier {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    pub fn stab(&mut self, is_stab: bool) {
        self.stab = is_stab;
    }

    pub fn crit(&mut self, is_crit: bool) {
        self.critical = is_crit;
    }

    pub fn screen(&mut self, using_screen: bool) {
        self.screen = using_screen;
    }

    pub fn choice(&mut self, using_choice: bool) {
        self.choiced = using_choice;
    }

    pub fn target(&mut self, target: MoveTarget) {
        self.target = target;
    }

    pub fn effectiveness(&mut self, effectiveness: TypeEffective) {
        self.effectiveness = effectiveness;
    }

    pub fn calculate(&self) -> f32 {
        let mut res: f32 = 1.0;

        if self.stab {
            res *= 1.5
        }
        if self.critical {
            res *= 2.0
        }
        if self.screen {
            res *= 0.5
        }
        if self.choiced {
            res *= 1.5
        }
        if self.target == MoveTarget::Multi {
            res *= 0.75
        }

        match self.effectiveness {
            TypeEffective::NotEffective => res *= 0.25,
            TypeEffective::NotVeryEffective => res *= 0.5,
            TypeEffective::SuperEffective => res *= 2.0,
            TypeEffective::ExtremelyEffective => res *= 4.0,
            _ => {}
        }

        res
    }
}

