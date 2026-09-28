use rand::Rng;

use crate::{
    engine::Engine,
    population::{ContiguousPopulation, Population},
    prelude::{Real, Time},
    simulation::{ParamSet, Simulation, Stepper, UpdateListener},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Spin {
    Up,
    Down,
}

impl Spin {
    fn value(self) -> i8 {
        if self == Self::Up {
            return 1;
        } else {
            return -1;
        }
    }
}

struct Spins {
    spins: Vec<Spin>,
}

impl Spins {
    fn new_random(n: usize, rng: &mut impl Rng) -> Self {
        Spins {
            spins: (1..n)
                .into_iter()
                .map(|_| {
                    if rng.gen_bool(0.5) {
                        Spin::Up
                    } else {
                        Spin::Down
                    }
                })
                .collect(),
        }
    }

    fn energy(&self, params: &Params) -> Real {
        let J = params.interaction_J;
        let mu = params.moment_mu;
        let h = params.field_H;
        let mut energy = 0.0;

        energy -= mu * h * (self.spins[0].value() as Real);
        for i in 1..self.spins.len() {
            energy -= mu * h * self.spins[i].value() as Real;
            energy -= J * (self.spins[i].value() * self.spins[i - 1].value()) as Real;
        }

        return energy;
    }

    /// Show Ising chain as a string of +'s and -'s.
    fn show(&self) -> String {
        self.spins
            .iter()
            .map(|&x| if x == Spin::Up { '+' } else { '-' })
            .collect()
    }
}

#[derive(Debug, Clone, Copy)]
struct Params {
    interaction_J: Real,
    field_H: Real,
    moment_mu: Real,
    temperature: Real,
}

impl<ST> ParamSet<ST> for Params {
    type BoundParams = Params;

    fn bind(&self, st: &ST) -> Self::BoundParams {
        *self
    }
}

type Ising1DPopulation = Vec<Spin>;
type Ising1DSimulation = Simulation<Params, Ising1DPopulation, ()>;

/* 
impl<L: UpdateListener> Stepper<L> for Ising1DSimulation {
    fn time(&self) -> Time {
        self.time
    }
    
    fn step(&mut self, listener: &mut L) {
        self.step_agents();
        self.step_pairs();
        self.time += 1;
    }
}
impl Ising1DSimulation {
    fn step_agents(&mut self) {}
    
    fn step_pairs(&mut self) {}
}
*/