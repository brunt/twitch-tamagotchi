use rand::rngs::{SmallRng, ThreadRng};
use rand::{Rng, SeedableRng};

pub enum Health {
    Healthy(Needs, ActiveState),
    #[allow(dead_code)]
    Dead,
}

#[derive(Eq, PartialEq, Debug)]
pub enum ActiveState {
    Idle,
    Playing,
    Grooming,
    Sleeping,
    Eating,
}

#[derive(Debug)]
pub struct Needs {
    pub energy: i32,
    pub happy: i32,
    pub hunger: i32,
    pub clean: i32,
    // pub hp: i32, ??
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            energy: 50,
            happy: 50,
            hunger: 50,
            clean: 50,
        }
    }
}

pub struct Tamagotchi {
    #[allow(dead_code)]
    pub name: String,
    pub health: Health,
    pub age: i32,
    rng: SmallRng,
}
impl Tamagotchi {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: Health::Healthy(Needs::default(), ActiveState::Idle),
            age: 0,
            rng: SmallRng::from_os_rng(),
        }
    }

    #[allow(dead_code)]
    fn get_needs(&self) -> Option<&Needs> {
        match &self.health {
            Health::Dead => None,
            Health::Healthy(needs, ..) => Some(needs),
        }
    }

    fn get_needs_mut(&mut self) -> Option<(&mut Needs, &mut ActiveState)> {
        match &mut self.health {
            Health::Dead => None,
            Health::Healthy(needs, active_state) => Some((needs, active_state)),
        }
    }

    //TODO: private after done printing
    pub fn get_needs_rng_mut(&mut self) -> Option<(&mut Needs, &mut ActiveState, &mut SmallRng)> {
        match &mut self.health {
            Health::Dead => None,
            Health::Healthy(needs, active_state) => Some((needs, active_state, &mut self.rng)),
        }
    }
    pub fn feed(&mut self, value: i32) {
        if let Some((needs, mut active_state)) = self.get_needs_mut() {
            if active_state == &mut ActiveState::Idle {
                needs.hunger = (needs.hunger + value).max(100);
                needs.happy = (needs.happy + (0.25 * value as f32) as i32).max(100);
                active_state = &mut ActiveState::Eating;
            }
        }
    }

    pub fn clean(&mut self, value: i32) {
        if let Some((needs, mut active_state)) = self.get_needs_mut() {
            if active_state == &mut ActiveState::Idle {
                needs.clean = 100;
                needs.happy = (needs.happy + (0.25 * value as f32) as i32).max(100);
                active_state = &mut ActiveState::Grooming;
                dbg!(needs);
            }
        }
    }

    pub fn play(&mut self) {
        if let Some((needs, mut active_state)) = self.get_needs_mut() {
            if active_state == &mut ActiveState::Idle {
                needs.happy = (needs.happy + (0.25 * needs.happy as f32) as i32).max(100);
                needs.energy = (needs.energy - (0.25 * needs.energy as f32) as i32).min(0);
                active_state = &mut ActiveState::Playing;
                dbg!(needs);
            }
        }
    }

    #[allow(dead_code)]
    pub fn sleep(&mut self) {
        if let Some((needs, mut active_state)) = self.get_needs_mut() {
            if active_state == &mut ActiveState::Idle {
                needs.energy = 100;
                //TODO: change active state for real
                active_state = &mut ActiveState::Sleeping;
            }
        }
    }

    //TODO: fix mut
    // pub fn idle(&mut self) {
    //     if let Some((needs, mut active_state)) = self.get_needs_mut() {
    //         // active_state = &mut ActiveState::Idle;
    //         self.health = Health::Healthy(, ActiveState::Idle);
    //     }
    // }

    #[allow(dead_code)]
    pub fn age(&mut self) {
        if let Some((needs, _, rng)) = self.get_needs_rng_mut() {
            needs.hunger = (needs.hunger - rng.random_range(10..20)).min(0);
            needs.happy = (needs.happy - rng.random_range(10..20)).min(0);
            needs.clean = (needs.clean - rng.random_range(10..20)).min(0);
            self.age += 1;
        }
    }
}
