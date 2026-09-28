use crate::{
    engine::Engine,
    population::Population,
    prelude::{Id, Time},
};
use getset::{CopyGetters, Getters, MutGetters};
use rand::{prelude::SmallRng, SeedableRng};
use std::fmt::Debug;

#[allow(unused_variables)]
pub trait UpdateListener {
    fn on_step_start(&mut self, t: Time) {}
    fn on_step_end(&mut self, t: Time) {}
    fn on_agent_updated(&mut self, id: Id) {}
    fn on_pair_updated(&mut self, id_a: Id, id_b: Id) {}
}

impl UpdateListener for () {}

/// The Stepper trait describes a structure that stores the bare bones simulation
/// state. The only action a simulation can do is to step() a single frame.
///
/// Simulation differs from Engine, which is also responsible to trigger events,
/// schedule tasks and track the overall execution state.
///
/// The simulation also stores an internal time counter.
pub trait Stepper<L: UpdateListener> {
    /// The current simulation time step.
    fn time(&self) -> Time;

    /// Execute a single step of simulation and dispatch messages to the given
    /// listener object.
    ///
    /// The listener object is usually specific to each simulation type and
    /// typically implement callback methods that should be called when certain
    /// events occur on the simulation.
    fn step(&mut self, listener: &mut L);
}

pub trait ParamSet<ST>: Clone {
    type BoundParams;

    // fn bound(&self, st: &ST) -> Box<Self::BoundParams>;

    /// Return the local set of parameters bound to the given state.
    fn bind(&self, st: &ST, f: impl FnOnce(&Self::BoundParams));

    /// Return the local set of parameters bound to the given state.
    fn bind_mut(&self, st: &mut ST, f: impl FnOnce(&Self::BoundParams, &mut ST));
}

pub trait Pairwise: Clone {
    /// Fill the list of pairs with new pairs.
    ///
    /// New pairs are appended to the input vector.
    fn fill_pairs(&mut self, pop: &impl Population, pairs: &mut Vec<(Id, Id)>);

    /// Return a list of pairs.
    fn pairs(&mut self, pop: &impl Population) -> Vec<(Id, Id)> {
        let mut pairs = vec![];
        self.fill_pairs(pop, &mut pairs);
        return pairs;
    }
}

impl Pairwise for () {
    fn fill_pairs(&mut self, _pop: &impl Population, _pairs: &mut Vec<(Id, Id)>) {}
}

/// Define several
#[derive(Clone)]
pub enum PairwiseByID {
    /// The product of all pairs, excluding the diagonal
    /// Each pair is emmited twice.
    FullProduct,

    /// All pairs, uses the upper triangular matrix;
    Upper,

    /// All pairs, uses the lower triangular matrix.
    Lower,

    /// Sequential neighboring pairs.
    Sequential,

    /// Sequential neighboring pairs. Wraps the last agent with the first.
    SequentialWrap,
}

impl Pairwise for PairwiseByID {
    fn fill_pairs(&mut self, pop: &impl Population, pairs: &mut Vec<(Id, Id)>) {
        if pop.count() <= 1 {
            return;
        }

        match self {
            Self::FullProduct => {
                // let ids = pop.each_agent(f)
            }
            Self::Upper => {}
            Self::Lower => {}
            Self::Sequential => {
                Self::SequentialWrap.fill_pairs(pop, pairs);
                pairs.pop();
            }
            Self::SequentialWrap => {
                let mut last = None;
                let mut first = None;

                pop.each_agent(move |id, _| {
                    if let Some(prev) = last {
                        pairs.push((prev, id));
                        last = Some(id);
                    } else {
                        first = Some(id);
                    }
                    match (last, first) {
                        (Some(i), Some(j)) => pairs.push((i, j)),
                        _ => {}
                    }
                })
            }
        }
    }
}

/// This struct stores all agents, global parameters necessary and an internal random
/// number generator necessary execute the simulation.
#[derive(Debug, Clone, Getters, CopyGetters, MutGetters)]
pub struct Simulation<Params, Pop, Pairs>
where
    Pop: Population,
    Params: ParamSet<Pop::State>,
    Pairs: Pairwise,
{
    #[getset(get_copy = "pub")]
    time: Time,

    #[getset(get = "pub", get_mut = "pub")]
    population: Pop,

    #[getset(get = "pub", get_mut = "pub")]
    params: Params,

    #[getset(get = "pub", get_mut = "pub")]
    pairwise: Pairs,

    #[getset(get = "pub", get_mut = "pub")]
    rng: SmallRng,
}

pub struct SimulationBuilder<Params, Pop, Pairs> {
    population: Option<Pop>,
    params: Option<Params>,
    pairwise: Option<Pairs>,
}

impl<Params, Pop, Pairs> Simulation<Params, Pop, Pairs>
where
    Pop: Population,
    Params: ParamSet<Pop::State>,
    Pairs: Pairwise,
{
    /// Create a Simulation builder.
    ///
    /// Builder is used to define population, parameters and the pairwise sampler
    /// function.
    ///
    /// Example
    ///
    /// ```rust
    /// Simulation::build()
    ///     .population(pop)
    ///     .params(params)
    ///     .pairwise(parwise)
    ///     .done();
    /// ```
    pub fn build() -> SimulationBuilder<Params, Pop, Pairs> {
        SimulationBuilder::default()
    }

    /// Create a new SimulationBuilder with default parameters
    pub fn build_default() -> SimulationBuilder<Params, Pop, Pairs>
    where
        Params: Default,
        Pairs: Default,
    {
        SimulationBuilder {
            population: None,
            params: Some(Params::default()),
            pairwise: Some(Pairs::default()),
        }
    }

    /// Create a new Simulation
    pub fn new(population: Pop, params: Params, pairwise: Pairs) -> Self {
        Simulation {
            time: 0,
            population,
            params,
            pairwise,
            rng: SmallRng::from_entropy(),
        }
    }

    /// Create engine from simulation
    pub fn engine<L: UpdateListener>(self, listener: L) -> Engine<Self, L>
    where
        Self: Stepper<L>,
    {
        Engine::new(self, listener)
    }
}

impl<Params, Pop, Pairs> Default for SimulationBuilder<Params, Pop, Pairs> {
    fn default() -> Self {
        SimulationBuilder {
            population: None,
            params: None,
            pairwise: None,
        }
    }
}

impl<Params, Pop, Pairs> SimulationBuilder<Params, Pop, Pairs>
where
    Pop: Population,
    Params: ParamSet<Pop::State>,
    Pairs: Pairwise,
{
    pub fn population(mut self, population: Pop) -> Self {
        self.population = Some(population);
        return self;
    }

    pub fn params(mut self, params: Params) -> Self {
        self.params = Some(params);
        return self;
    }

    pub fn pairwise(mut self, pairwise: Pairs) -> Self {
        self.pairwise = Some(pairwise);
        return self;
    }

    pub fn try_build(self) -> Option<Simulation<Params, Pop, Pairs>> {
        Some(Simulation::new(
            self.population
                .unwrap_or_else(|| Pop::new_population(vec![].iter())),
            self.params?,
            self.pairwise?,
        ))
    }

    pub fn done(self) -> Simulation<Params, Pop, Pairs> {
        self.try_build().unwrap()
    }
}

/*
/// A trait that implements a simulation centered around a population object.
pub trait PopulationSimulation<R, L>: Simulation<L>
where
    R: Rng,
    L: PopulationSimulationListener,
{
    /// Method used to update a single agent from a mutable reference.
    ///
    /// Return false if update was cancelled for some reason.
    ///
    /// Default implementation is a NO-OP.
    fn update_agent(
        &mut self,
        ag: &mut <<Self as PopulationRngContext<R>>::Population as Population>::State,
    ) -> bool {
        return false;
    }

    /// Method used to update a pair of interacting agents from mutable references.
    ///
    /// Rust borrowing rules forbid that a and b points to the same reference, hence a and b
    /// always represent different agents.
    ///
    /// Return false if update was cancelled for some reason.
    ///
    /// Default implementation is a NO-OP.
    fn update_pair(
        &mut self,
        a: &mut <<Self as PopulationRngContext<R>>::Population as Population>::State,
        b: &mut <<Self as PopulationRngContext<R>>::Population as Population>::State,
    ) -> bool {
        return false;
    }

    /// Method used to select pairs of interacting agents.
    ///
    /// It return a list of interacting id pairs.
    ///
    /// Default implementation returns an empty vector.
    fn sample_pairs(&self) -> Vec<(Id, Id)> {
        vec![]
    }

    /// Update each agent using the update_agent() method.
    fn step_agents(&mut self, listener: &mut L) {
        self.with_population_rng(|pop, rng| {
            pop.each_agent_mut(|id, ag| {
                if self.update_agent(ag) {
                    listener.updated_agent(id);
                }
            })
        });
    }

    /// Update each pair of agents using the update_pair() method to the list of sample pairs.
    fn step_pairs(&mut self, listener: &mut L) {
        for (i, j) in self.sample_pairs() {
            if let Some((src, dest)) = self.population_mut().get_pair_mut(i, j) {
                if self.update_pair(src, dest) {
                    listener.updated_pair(i, j);
                }
            }
        }
    }
}



impl<L, PARAM, POP, PW> Simulation<L> for SimpleSimulation<PARAM, POP, PW>
where
    PARAM: Clone,
    POP: Population,
    L: PopulationSimulationListener,
    Self: PopulationSimulation<SmallRng, L>,
{
    fn time(&self) -> Time {
        self.time
    }

    fn step(&mut self, listener: &mut L) {
        self.time += 1;
        self.step_agents(listener);
        self.step_pairs(listener);
    }
}

impl<P, ST> SimpleSimulation<P, ST>
where
// P: ParamSet<ST>,
// ST: RandomUpdate<P::BoundParams> + EpiModel + Clone,
// P::BoundParams: EpiParams,
{
    /// Advance a single simulation step.
    ///
    /// Infection pairs are produced by the sampler object.
    ///
    /// The callback function `cb` is executed for every new infection pair.
    pub fn step_old<S, F>(&mut self, sampler: &S, cb: F) -> usize
    where
    F: FnMut(usize, usize),
    {
        // Simulate agent interactions, allowing new infections to occur.
        let mut on_infection = cb;

        for (i, j) in sampler.sample_infection_pairs(&self.population, &mut self.rng) {
            if i == j {
                continue;
            }
            if let Some((src, dest)) = self.population.get_pair_mut(i, j) {
                if !self.rng.gen_bool(self.params.bind(src).prob_protect())
                && dest.contaminate_from(src)
                {
                    cases += 1;
                    on_infection(i, j);
                }
            }
        }
        return cases;
    }
}
*/
