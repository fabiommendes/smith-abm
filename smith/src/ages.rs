use crate::{population::Population, prelude::Real, rng_context::PopulationRngRefMut};
use rand::{distributions::WeightedIndex, prelude::*};

/// Age of an agent, in years.
pub type Age = u8;

/// An age distribution array in bins of 10 years.
pub type AgeDistribution10 = [Real; 9];

/// Count population in each bin of 10 years.
pub type AgeCount10 = [u32; 9];

/// Convert age counts to a normalized age distribution.
pub fn normalize_age_counts(ages: AgeCount10) -> AgeDistribution10 {
    let total = ages.iter().sum::<u32>() as Real;
    return ages.map(|x| x as Real / total);
}

/// A trait for agents that have an age field. Many epidemiological models are
/// age-sensitive and it is useful to isolate this property as trait to implement
/// generic functionality associated with processing ages.
pub trait HasAge {
    /// Agent's age.
    fn age(&self) -> Age;

    /// Set age with given value.
    fn set_age(&mut self, value: Age) -> &mut Self;
}

/// Sample n ages from a non-empty vector of probabilities given as pairs
/// (age_group, prob). Probabilities do not need to be normalized and age
/// groups are iterpreted the range from the given number to the next value.
///
/// Each age group is assumed to span 10 years.
pub fn random_ages(n: usize, rng: &mut impl Rng, probs: AgeDistribution10) -> Vec<Age> {
    let distrib = WeightedIndex::new(&probs).unwrap();
    return (0..n)
        .map(|_| (10 * distrib.sample(rng) + rng.gen_range(0..10)) as Age)
        .collect();
}

/// Return the age counts in population.
pub fn age_counts<P>(pop: &P) -> AgeCount10
where
    P: Population,
    P::State: HasAge,
{
    let mut counts = AgeCount10::default();
    let max_index = counts.len() - 1;

    pop.each_agent(|_, ag: &P::State| {
        let id = ag.age() / 10;
        counts[(id as usize).min(max_index)] += 1;
    });

    return counts;
}

impl<'a, R, P> PopulationRngRefMut<'a, R, P>
where
    R: Rng,
    P: Population,
    P::State: HasAge,
{
    /// Set ages of all agents acording to distribution.
    pub fn set_age_distribution(self, distrib: AgeDistribution10) {
        let population = self.population;
        let rng = self.rng;
        let ages = random_ages(population.count(), rng, distrib);
        population.each_agent_mut(|id, st| drop(st.set_age(ages[id])));
    }

    /// Set ages from age counts. This method fills the age bins sequentially and stops
    /// once all age counts were set.
    ///
    /// This usually means that the age counts should sum to the population size.
    pub fn set_age_counts(self, counts: AgeCount10) {
        let mut i = 0;
        let mut n = counts.get(i).map(|x| *x).unwrap_or(0);
        let population = self.population;
        let rng = self.rng;

        population.each_agent_mut(|_, st| {
            if n <= 0 {
                i += 1;
                n = counts.get(i).map(|x| *x).unwrap_or(0);
            }
            if n > 0 {
                let start = i * 10;
                st.set_age(rng.gen_range(start..start + 10) as Age);
                n -= 1;
            }
        });
    }
}
