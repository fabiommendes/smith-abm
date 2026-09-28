use rand::Rng;
use std::error::Error;

extern crate clap;
use clap::{App, Arg, ArgMatches};
use simple_logger::SimpleLogger;
use smith_abm::{
    engine::Engine,
    population::{ContiguousPopulation, Population},
    prelude::{Id, Real, Time},
    simulation::{Pairwise, ParamSet, Simulation, Stepper, UpdateListener},
};

type Param_t = ();
type Pairwise_t = ();
type Population_t = ();
type Listener_t = ();
type Simulation_t = Simulation<Param_t, Population_t, Pairwise_t>;
type Engine_t = Engine<Simulation_t, Listener_t>;
type Fallible<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
enum SIRState {
    Susceptible,
    Infectious,
    Recovered,
}

#[derive(Clone, Debug)]
struct SIRParams {
    n_infections: Real,
    prob_infection: Real,
    infectious_period: Real,
}

#[derive(Clone, Debug)]
struct GlobalParams<T> {
    //#[getset(get = "pub", set = "pub")]
    params: T,
}

impl<T: Clone, ST> ParamSet<ST> for GlobalParams<T> {
    type BoundParams = Self;

    fn bind(&self, st: &ST, f: impl FnOnce(&Self::BoundParams)) {
        f(self);
    }

    fn bind_mut(&self, st: &mut ST, f: impl FnOnce(&Self::BoundParams, &mut ST)) {
        f(self, st);
    }
}

/*
impl<ST: Clone> ParamSet<ST> for () {
    type BoundParams = Self;

    fn bind(&self, st: &ST, f: impl FnOnce(&Self::BoundParams)) {
        f(self);
    }

    fn bind_mut(&self, st: &mut ST, f: impl FnOnce(&Self::BoundParams, &mut ST)) {
        f(self, st);
    }
}

impl Stepper<()> for Simulation_t {
    fn time(&self) -> Time {
        self.time()
    }

    fn step(&mut self, listener: &mut ()) {
        todo!()
    }
}
*/

impl<T> ParamSet<T> for SIRParams {
    type BoundParams = Self;

    fn bind(&self, st: &T, f: impl FnOnce(&Self::BoundParams)) {
        f(self);
    }

    fn bind_mut(&self, st: &mut T, f: impl FnOnce(&Self::BoundParams, &mut T)) {
        f(self, st);
    }
}

fn standard_app<'a, 'b>(name: &'b str, author: &'b str, about: &'b str) -> App<'a, 'b> {
    let st = name.to_string();
    let (name, version) = st.split_once("-").unwrap();
    return App::new(name)
        .version(version)
        .author(author)
        .about(about)
        .arg(
            Arg::with_name("name")
                .long("name")
                .value_name("NAME")
                .help("Sets a custom simulation name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_agents")
                .short("n")
                .long("n-agents")
                .help("Population size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_iter")
                .short("i")
                .long("n-iter")
                .help("Number of iterations")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("n_infections")
                .short("i")
                .long("n-infections")
                .help("Start simulation infecting this given number of individuals")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("seed")
                .long("seed")
                .help("Random string that initializes the RNG")
                .takes_value(true),
        );
}

fn main() -> Fallible<()> {
    SimpleLogger::new().init().unwrap();

    let matches = standard_app(
        "SIR-0.1",
        "Fábio Mendes <fabiomacedomendes@gmail.com>",
        "Simulate a simple SIR model epidemics.",
    )
    .arg(
        Arg::with_name("prob_infections")
            .long("prob-infections")
            .help("Probability of a single infection")
            .takes_value(true),
    )
    .arg(
        Arg::with_name("n_contacts")
            .long("n-contacts")
            .help("Average number of contacts for each agent")
            .takes_value(true),
    )
    .get_matches();

    let listener = ();
    let pop = ();
    let mut engine = Simulation::build_default()
        .population(pop)
        .done()
        .engine(listener);
    engine.run(100);

    let csv = engine.tables().render_csv(',');
    println!("{}", csv);

    return Ok(());
}
