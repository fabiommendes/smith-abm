use rand::Rng;

use crate::{
    engine::Engine,
    population::{ContiguousPopulation, Population},
    prelude::{Real, Time},
    simulation::{ParamSet, Simulation, Stepper, UpdateListener},
};