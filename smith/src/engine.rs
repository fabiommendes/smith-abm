use crate::{
    population::Population,
    prelude::Time,
    rng_context::RngContext,
    scheduler::Scheduler,
    simulation::{Stepper, UpdateListener},
    utils::Table,
};
use getset::{Getters, MutGetters};
use log;
use rand::{
    prelude::{SeedableRng, SmallRng},
    Rng,
};
use rayon::prelude::*;
use std::fmt::Debug;

/// The Engine struct is responsible to holding a Simulation field and can
/// schedule events, trigger messages and do other types of tracking and
/// bookeeping activities.
///
/// The choice of making two classes was done in order to keep the bare bones
/// simulation logic separated from secondary goals of tracking variables and
/// results, scheduling tasks, etc.
#[derive(Getters, MutGetters, Clone)]
pub struct Engine<Sim, L>
where
    Sim: Stepper<L>,
    L: UpdateListener,
{
    #[getset(get = "pub", get_mut = "pub")]
    simulation: Sim,

    #[getset(get = "pub", get_mut = "pub")]
    listener: L,

    #[getset(get = "pub", get_mut = "pub")]
    scheduler: Scheduler<Sim>,

    #[getset(get = "pub", get_mut = "pub")]
    tables: Table<usize>,
}

pub struct EngineBuilder<Sim, L>
where
    Sim: Stepper<L>,
    L: UpdateListener,
{
    simulation: Option<Sim>,
    listener: Option<L>,
    scheduler: Option<Scheduler<Sim>>,
}

impl<Sim, L> Engine<Sim, L>
where
    Sim: Stepper<L>,
    L: UpdateListener,
{
    /// Create new simulation from population and sampler.
    pub fn new(simulation: Sim, listener: L) -> Self {
        Engine {
            simulation,
            listener,
            scheduler: Scheduler::new(),
            tables: Table::new(vec![]),
        }
    }

    /// Return the current simulation time
    pub fn time(&self) -> Time {
        return self.simulation.time();
    }

    /*
    /// Initialize simulation and calibrate sampler from a curve of cases.
    ///
    /// This is a somewhat simplistic view on model calibration. We just run
    /// the simulation normally but at each step we recalibrate the sampler
    /// to produce the same number of infections as expected from the epidemic
    /// curve.
    pub fn calibrate_sampler_from_cases(&'a mut self, cases: &[Real]) -> &mut Self
    where
        ST::Clinical: Default,
    {
        // TODO: create calibrator struct
        let alpha = 0.5;
        let min_contacts = 0.0;
        let max_contacts = 10.0;
        let min_scale = 1.0 / 1.5;
        let max_scale = 1.33;
        let e_ratio = 0.25;
        let r = 0.85;

        let mut n_iter = 0;
        let mut excess = 0.0;
        let mut acc_cases = 0.0;
        let mut acc_target = 0.0;
        let mut c_mean = self.sampler.contacts();

        for raw_target in cases {
            n_iter += 1;
            acc_target += raw_target;

            let target = (raw_target + e_ratio * excess).max(0.0);
            let estimate = self
                .sampler
                .expected_infection_pairs(&self.state.population);
            let grow = ((target + alpha) / (estimate + alpha)).clamp(min_scale, max_scale);

            // Calibrate contacts. Other implementations might calibrate different
            // coefficients, but we do not have any way to generalize it yet.
            let c1 = self.sampler.contacts();
            let c2 = (c1 * grow).clamp(min_contacts, max_contacts);
            c_mean = c_mean * r + c2 * (1.0 - r);
            self.sampler.set_contacts(c2);

            // Run and register the number of cases
            let n_cases = self.steps(1);
            acc_cases += n_cases as Real;
            excess = acc_target - acc_cases as Real;

            // If excess is very large (very negative), we might want to create
            // artificial infections to quickstart an infection
            if excess > 0.25 * (acc_target + alpha) {
                let n = (excess * 0.25) as usize;
                self.state
                    .population
                    .contaminate_at_random(n, true, &mut self.state.rng);
                acc_cases += n as Real;
                excess = acc_target - acc_cases as Real;
            }

            trace!(target: "calibrate_sample_cases", "iter {}, coeff: {:.2} ({:.2})\n  - target: {} ({}); cases: {} (~ {:.1}); excess: {}", n_iter, c2, c_mean, raw_target, target, n_cases, estimate, excess);
        }
        self.sampler.set_contacts(c_mean);
        debug!(target: "calibrate_sample_cases", "final contacts: {}, {} iterations", c_mean, n_iter);
        return self;
    }
    */

    /// Like steps, but return Self, rather then the number of cases. This is
    /// useful to use in builder-like APIs.
    #[inline]
    pub fn run(&mut self, n_steps: usize) -> &mut Self {
        for _ in 0..n_steps {
            self.step();
        }
        return self;
    }

    /// Create n copies of simulation and run them for n_steps in parallel.
    pub fn run_parallel(&self, n: usize, n_steps: usize) -> Vec<Self>
    where
        Sim: Send + Sync + RngContext<SmallRng>,
        L: Send + Sync,
        Self: Clone,
    {
        let mut result = self.copies(n);
        result.par_iter_mut().for_each(|sim| {
            sim.run(n_steps);
        });
        return result;
    }

    /// Create n copies of self.
    ///
    /// RNG is initialized from entropy in each copy.
    pub fn copies(&self, n: usize) -> Vec<Self>
    where
        Sim: RngContext<SmallRng>,
        Self: Clone,
    {
        return (1..n)
            .map(|_| {
                let mut new = self.clone();
                new.simulation.seed_from(&mut SmallRng::from_entropy());
                new
            })
            .collect();
    }

    /// Run a single simulation step;
    pub fn step(&mut self) {
        let start_time = self.simulation.time();
        let scheduler = &mut self.scheduler;
        let listener = &mut self.listener;
        let simulation = &mut self.simulation;

        log::debug!("running step: {}", start_time + 1);
        scheduler.before_step(simulation);
        listener.on_step_start(start_time);
        simulation.step(listener);
        listener.on_step_end(start_time);
        scheduler.after_step(simulation);

        // let population = &simulation.population;
        // if let Some(table) = self.epicurves.as_mut() {
        //     table.count_epidemic_compartments(population, true);
        // }
    }

    /*
    /// Return the tip of the epicurve
    pub fn epistate(&self, normalize: bool) -> Vec<Real> {
        let factor = self._normalization_factor(normalize);
        if let Some(table) = self.epicurves.as_ref() {
            return table.tip().iter().map(|a| *a as Real * factor).collect();
        } else {
            return vec![0.0; ST::CARDINALITY];
        }
    }

    /// Return curve for the n-th component of epicurve.
    ///
    /// If normalized, results are divided by population size.
    pub fn get_epicurve(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let data = self.epicurves.as_ref()?.col(n)?;
        let mut vec = Vec::with_capacity(data.len());
        let factor = self._normalization_factor(normalize);
        for x in data {
            vec.push(x as Real * factor);
        }
        return Some(vec);
    }

    /// Get epistate at a given iteration
    pub fn get_epistate(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let factor = self._normalization_factor(normalize);
        let row = self.epicurves.as_ref()?.row(n)?;
        return Some(row.iter().map(|x| *x as Real * factor).collect());
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self) -> String {
        let mut infections = vec![0];
        if let Some(counts) = self.dispatcher.infection_counts() {
            infections.extend(counts.iter());
        }

        if let Some(table) = &self.epicurves {
            return table
                .clone()
                .add_column("cases", infections.iter().cloned(), true)
                .render_csv(',');
        } else {
            return "".to_string();
        }
    }

    /// Used internally to normalize (or not) results
    fn _normalization_factor(&self, normalize: bool) -> Real {
        if normalize {
            1.0 / self.count() as Real
        } else {
            1.0
        }
    }

    /// Get epidemiological params for given agent
    ///
    /// Return Some(FullSEIRParams<f64>) if agent exists.
    pub fn get_local_epiparams(&self, i: usize) -> Option<P::LocalParams>
    where
    ST: EpiModel,
    P::LocalParams: EpiParams,
    {
        let ag = self.state.population.get(i)?;
        let params = self.params.local_params(ag);
        return Some(params);
    }

    /// Work with mutable references to the internal population, parameters and RNG.
    pub fn with_parts<R>(&mut self, f: impl FnOnce(&mut Vec<ST>, &mut P, &mut SmallRng) -> R) -> R {
        return f(
            &mut self.simulation.population_mut(),
            &mut self.simulation.params_mut(),
            &mut self.simulation.rng(),
        );
    }

    /// Work with a mutable reference to the internal RNG, parameters and population.
    pub fn with_state_mut<R>(&mut self, f: impl FnOnce(&mut Simulation<P, ST>) -> R) -> R {
        return f(&mut self.simulation);
    }
    */
}

/*
impl<'a, P, ST> Engine<ST, P, SimpleSampler>
where
    P: ParamSet<ST>,
    ST: RandomUpdate<P::BoundParams> + EpiModel + Debug,
    P::BoundParams: EpiParams,
{
    /// Create a new simulation from a simple sampler
    pub fn new_simple(
        params: P,
        population: Vec<ST>,
        n_contacts: Real,
        prob_infection: Real,
    ) -> Self {
        let sampler = SimpleSampler::new(n_contacts, prob_infection);
        return Self::new(params, population, sampler);
    }
}
*/
