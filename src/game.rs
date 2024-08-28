use rustc_hash::FxHashSet;

#[derive(Debug)]
struct Boss {
    hitpoints: i32,
    damage: i32,
    poisoned: Option<i32>,
}

impl Boss {
    fn new() -> Self {
        Self {
            hitpoints: 55,
            damage: 8,
            poisoned: None,
        }
    }

    fn new_custom(hitpoints: i32, damage: i32) -> Self {
        Self {
            hitpoints,
            damage,
            poisoned: None,
        }
    }

    fn attack(&mut self, enemy: &mut Wizard) {
        let adjusted_damage = if self.damage - enemy.armor <= 0 {
            1
        } else {
            self.damage - enemy.armor
        };
        enemy.hitpoints -= adjusted_damage;
    }

    fn apply_effect(&mut self) {
        if let Some(poison_timer) = self.poisoned.as_mut() {
            self.hitpoints -= 3;
            *poison_timer -= 1;
            if *poison_timer == 0 {
                self.poisoned = None;
            }
        }
    }
}

#[derive(Debug)]
pub struct Wizard {
    pub hitpoints: i32,
    pub armor: i32,
    pub mana: i32,
    pub shielded: Option<i32>,
    pub recharging: Option<i32>,
    pub possible_spells: FxHashSet<Spell>,
}

impl Wizard {
    fn new() -> Self {
        let mut wizard = Self {
            hitpoints: 50,
            armor: 0,
            mana: 500,
            shielded: None,
            recharging: None,
            possible_spells: FxHashSet::default(),
        };
        wizard.update_possible_spells(&Boss::new());
        wizard
    }

    fn new_custom(hitpoints: i32, mana: i32) -> Self {
        let mut wizard = Self {
            hitpoints,
            armor: 0,
            mana,
            shielded: None,
            recharging: None,
            possible_spells: FxHashSet::default(),
        };
        wizard.update_possible_spells(&Boss::new());
        wizard
    }

    fn magic_missile(&mut self, enemy: &mut Boss) {
        self.mana -= Spell::MagicMissile.get_mana();
        enemy.hitpoints -= 4;
    }

    fn drain(&mut self, enemy: &mut Boss) {
        self.mana -= Spell::Drain.get_mana();
        enemy.hitpoints -= 2;
        self.hitpoints += 2;
    }

    fn shield(&mut self) {
        self.mana -= Spell::Shield.get_mana();
        if self.shielded.is_some() {
            panic!("Can not shield with existing shield");
        } else {
            self.shielded = Some(6);
            self.armor = 7;
        }
    }

    fn poison(&mut self, enemy: &mut Boss) {
        self.mana -= Spell::Poison.get_mana();
        if enemy.poisoned.is_some() {
            panic!("Cannot poison with existing poison");
        } else {
            enemy.poisoned = Some(6);
        }
    }

    fn recharge(&mut self) {
        self.mana -= Spell::Recharge.get_mana();
        if self.recharging.is_some() {
            panic!("Can not recharge with existing recharging");
        } else {
            self.recharging = Some(5);
        }
    }

    fn apply_effect(&mut self) {
        if let Some(shield_timer) = self.shielded.as_mut() {
            *shield_timer -= 1;
            if *shield_timer == 0 {
                self.armor = 0;
                self.shielded = None;
            }
        }
        if let Some(recharge_timer) = self.recharging.as_mut() {
            self.mana += 101;
            *recharge_timer -= 1;
            if *recharge_timer == 0 {
                self.recharging = None;
            }
        }
    }

    fn update_possible_spells(&mut self, enemy: &Boss) {
        if self.mana >= Spell::MagicMissile.get_mana() {
            self.possible_spells.insert(Spell::MagicMissile);
        } else {
            self.possible_spells.remove(&Spell::MagicMissile);
        }
        if self.mana >= Spell::Drain.get_mana() {
            self.possible_spells.insert(Spell::Drain);
        } else {
            self.possible_spells.remove(&Spell::Drain);
        }
        if self.mana >= Spell::Shield.get_mana()
            && (self.shielded.is_none() || self.shielded == Some(1))
        {
            self.possible_spells.insert(Spell::Shield);
        } else {
            self.possible_spells.remove(&Spell::Shield);
        }
        if self.mana >= Spell::Poison.get_mana()
            && (enemy.poisoned.is_none() || enemy.poisoned == Some(1))
        {
            self.possible_spells.insert(Spell::Poison);
        } else {
            self.possible_spells.remove(&Spell::Poison);
        }
        if self.mana >= Spell::Recharge.get_mana()
            && (self.recharging.is_none() || self.recharging == Some(1))
        {
            self.possible_spells.insert(Spell::Recharge);
        } else {
            self.possible_spells.remove(&Spell::Recharge);
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Spell {
    MagicMissile,
    Drain,
    Shield,
    Poison,
    Recharge,
}

impl Spell {
    fn get_mana(&self) -> i32 {
        match self {
            Spell::MagicMissile => 53,
            Spell::Drain => 73,
            Spell::Shield => 113,
            Spell::Poison => 173,
            Spell::Recharge => 229,
        }
    }
}

#[derive(Debug)]
pub struct Battle {
    wizard: Wizard,
    boss: Boss,
    hard_mode: bool,
    mana_used: i32,
    spells_used: Vec<Spell>,
    /// Did the the wizard win
    outcome: Option<bool>,
}

pub struct EffectOngoingError();

impl Default for Battle {
    fn default() -> Self {
        Battle {
            wizard: Wizard::new(),
            boss: Boss::new(),
            hard_mode: false,
            mana_used: 0,
            spells_used: Vec::new(),
            outcome: None,
        }
    }
}

impl Battle {
    fn new(wizard: Wizard, boss: Boss, hard_mode: bool) -> Self {
        Self {
            wizard,
            boss,
            hard_mode,
            mana_used: 0,
            spells_used: Vec::new(),
            outcome: None,
        }
    }

    /// Returns Some(true) if the wizard won, Some(false) if the boss won or None
    /// if neither has won.
    pub fn wizard_turn(&mut self, spell: &Spell) -> Result<Option<bool>, EffectOngoingError> {
        // Check chosen spell is possible
        if !self.wizard.possible_spells.contains(spell) {
            return Err(EffectOngoingError());
        }

        // Wizard's turn
        if self.hard_mode {
            self.wizard.hitpoints -= 1;
            if self.wizard.hitpoints <= 0 {
                self.outcome = Some(false);
                return Ok(Some(false));
            }
        }
        self.wizard.apply_effect();
        self.boss.apply_effect();

        match spell {
            Spell::MagicMissile => self.wizard.magic_missile(&mut self.boss),
            Spell::Drain => self.wizard.drain(&mut self.boss),
            Spell::Shield => self.wizard.shield(),
            Spell::Poison => self.wizard.poison(&mut self.boss),
            Spell::Recharge => self.wizard.recharge(),
        }
        self.mana_used += spell.get_mana();
        self.spells_used.push(spell.clone());

        if self.boss.hitpoints <= 0 {
            self.outcome = Some(true);
            return Ok(Some(true));
        }
        Ok(None)
    }

    /// Returns Some(true) if the wizard won, Some(false) if the boss won or None
    /// if neither has won.
    pub fn boss_turn(&mut self) -> Option<bool> {
        // Boss' turn
        self.wizard.apply_effect();
        self.boss.apply_effect();
        if self.boss.hitpoints <= 0 {
            self.outcome = Some(true);
            return Some(true);
        }

        self.boss.attack(&mut self.wizard);
        if self.wizard.hitpoints <= 0 {
            self.outcome = Some(false);
            return Some(false);
        }

        self.wizard.update_possible_spells(&self.boss);

        None
    }

    pub fn get_wizard(&self) -> &Wizard {
        &self.wizard
    }
}

#[cfg(test)]
mod solution {
    use super::*;

    #[test]
    fn battle_simple() {
        let wizard = Wizard::new_custom(10, 250);
        let boss = Boss::new_custom(14, 8);
        let mut battle = Battle::new(wizard, boss, false);
        for spell in [
            Spell::Recharge,
            Spell::Shield,
            Spell::Drain,
            Spell::Poison,
            Spell::MagicMissile,
        ] {
            let outcome = battle.step(&spell);
            assert!(outcome.is_ok());
        }
        assert_eq!(battle.outcome, Some(true));
    }
}
