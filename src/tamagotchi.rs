use crate::commands::PetCommand;
use circular_buffer::CircularBuffer;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub enum Health {
    Healthy(Needs, ActiveState),
    Dead,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ActiveState {
    Idle,
    Playing,
    Grooming,
    Sleeping,
    Eating,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Needs {
    pub energy: i32,
    pub happy: i32,
    pub hunger: i32,
    pub clean: i32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            energy: 100,
            happy: 100,
            hunger: 100,
            clean: 100,
        }
    }
}

#[derive(Serialize)]
pub struct Tamagotchi {
    pub name: String,
    pub health: Health,
    pub age: i32,
    #[serde(skip)]
    command_queue: CircularBuffer<20, PetCommand>,
    #[serde(skip)]
    action_timer: u8,
    #[serde(skip)]
    rng: SmallRng,
}

impl Display for Tamagotchi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: age: {}, stats: {:?}",
            self.name, self.age, self.health
        )
    }
}

impl Tamagotchi {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: Health::Healthy(Needs::default(), ActiveState::Idle),
            age: 0,
            command_queue: CircularBuffer::new(),
            action_timer: 0,
            rng: SmallRng::from_os_rng(),
        }
    }

    pub fn kill(&mut self) {
        self.health = Health::Dead;
    }

    pub fn tick(&mut self) {
        if let Health::Healthy(ref mut needs, ref mut state) = self.health {
            if self.action_timer > 0 {
                self.action_timer -= 1;
                if self.action_timer == 0 && *state != ActiveState::Sleeping {
                    *state = ActiveState::Idle;
                }
            }
            if let Some(cmd) = self.command_queue.pop_front() {
                match cmd {
                    PetCommand::Feed => {
                        needs.hunger = (needs.hunger + 50).min(100);
                        *state = ActiveState::Eating;
                        self.action_timer = 1;
                    }
                    PetCommand::Sleep => {
                        needs.energy = (needs.energy + 25).min(100);
                        *state = ActiveState::Sleeping;
                        self.action_timer = 2;
                    }
                    PetCommand::Clean => {
                        needs.clean = (needs.clean + 50).min(100);
                        *state = ActiveState::Grooming;
                        self.action_timer = 1;
                    }
                    PetCommand::Play => {
                        needs.happy = (needs.happy + 50).min(100);
                        *state = ActiveState::Playing;
                        self.action_timer = 1;
                    }
                    _ => unreachable!(),
                }
            }
            let age_factor = match self.age {
                x if x < 50 => 4,
                x if x < 100 => 3,
                x if x < 150 => 2,
                _ => 1,
            };
            if needs.energy == 0 || (state == &ActiveState::Sleeping && needs.energy < 100) {
                *state = ActiveState::Sleeping;
                needs.energy = (needs.energy + 25).min(100);
            } else if self.action_timer == 0 {
                *state = ActiveState::Idle;
                needs.energy = (needs.energy - self.rng.random_range(1..3)).max(0);
            };
            needs.hunger = (needs.hunger - (self.rng.random_range(1..3) * age_factor)).max(0);
            needs.happy = (needs.happy - (self.rng.random_range(1..3)) * age_factor).max(0);
            needs.clean = (needs.clean - self.rng.random_range(1..3)).max(0);
            self.age += 1;

            if needs.hunger == 0 || needs.happy == 0 {
                self.health = Health::Dead;
            }
        }
    }

    pub fn add_command(&mut self, command: PetCommand) {
        self.command_queue.push_back(command);
    }
}
