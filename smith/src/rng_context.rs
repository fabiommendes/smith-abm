use std::collections::HashSet;

use crate::{
    population::{Population},
    prelude::Id,
};
use rand::{Rng, SeedableRng};

/// Trait for objects that store an internal random number generator.
pub trait RngContext<R: Rng> {
    /// Mutable reference to the RNG.
    fn with_rng<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut R) -> T;

    /// Set seed for random number generator
    fn seed_from_u64(&mut self, seed: u64) -> &mut Self
    where
        R: SeedableRng + Clone,
    {
        self.with_rng(|rng| *rng = <R as SeedableRng>::seed_from_u64(seed));
        return self;
    }

    /// Set seed for random number generator
    fn seed_from_string(&mut self, data: &str) -> &mut Self
    where
        R: SeedableRng + Clone,
    {
        let mut seed = <R as SeedableRng>::Seed::default();
        let dest = seed.as_mut();
        let src = data.as_bytes();

        for i in 0..(dest.len().min(src.len())) {
            dest[i] += src[i];
        }

        self.with_rng(|rng| *rng = <R as SeedableRng>::from_seed(seed));
        return self;
    }

    /// Set seed for random number generator
    fn seed_from(&mut self, rng: &R) -> &mut Self
    where
        R: Clone,
    {
        self.with_rng(|rng_| *rng_ = rng.clone());
        return self;
    }

    /// Increment the RNG. Useful to avoid repeated runs for methods that do not affect the RNG.
    fn rng_next(&mut self) -> &mut Self {
        self.with_rng(|rng| rng.gen_bool(0.5));
        return self;
    }
}

/// A rich reference to a rng() and a population() object
pub struct PopulationRngRef<'a, R: Rng, P: Population> {
    pub rng: &'a mut R,
    pub population: &'a P,
}

/// A rich reference to a rng() and a population() object
pub struct PopulationRngRefMut<'a, R: Rng, P: Population> {
    pub rng: &'a mut R,
    pub population: &'a mut P,
}

impl<'a, R: Rng, P: Population> PopulationRngRef<'a, R, P> {
    /// Select a random id using random number generator.
    pub fn random_id(self) -> Id {
        let n = self.population.count();
        self.rng.gen_range(0..n)
    }

    /// Select a sample of random agent ids.
    pub fn random_ids(self, count: usize) -> Vec<Id> {
        let mut ids = HashSet::new();
        let mut missing = count;
        let n = self.population.count();

        while missing > 0 {
            let id = self.rng.gen_range(0..n);
            if !ids.contains(&id) {
                ids.insert(id);
                missing -= 1;
            }
        }
        return ids.into_iter().collect();
    }

    /// Select a random agent using random number generator.
    pub fn random_agent(self) -> (Id, &'a P::State) {
        let population = self.population;
        let id = self.random_id();
        let ag = population.get_agent(id).unwrap();
        return (id, ag);
    }

    /// Select a sample of distinct random agents.
    ///
    /// Return a vector of (Id, State) pairs.
    pub fn random_agents(self, count: usize) -> Vec<(Id, P::State)>
    where
        P::State: Clone,
    {
        let mut out = Vec::new();
        self.map_random_agents(count, |id, st| out.push((id, st.clone())));
        return out;
    }

    /// Select many distinct random agents.
    fn map_random_agents<F>(self, count: usize, f: F)
    where
        F: FnMut(usize, &P::State),
        P::State: Clone,
    {
        let population = self.population;
        let ids = self.random_ids(count);
        population.map_agents(&ids, f);
    }
}

/*
impl<'a, R: Rng, P: Population> PopulationRngRefMut<'a, R, P> {
    /// Select a random agent using random number generator.
    fn random_agent_mut(&'a mut self) -> (Id, &'a mut P::State) {
        let id = self.random_id();
        let ag = self.population.get_agent_mut(id).unwrap();
        return (id, ag);
    }

    /// A mutable version of map_random_agents()
    fn map_random_agents_mut<F>(&'a mut self, count: usize, f: F)
    where
        F: FnMut(usize, &mut P::State),
        P::State: Clone,
    {
        let ids = self.random_ids(count);
        let mut g = f;
        self.population.map_agents_mut(&ids, g);
    }
}
*/
