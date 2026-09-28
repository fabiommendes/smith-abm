/** SCHEDULED INFECTIONS */
pub struct InfectionPlan {
    plan: Vec<usize>,
}

impl InfectionPlan {
    pub fn new() -> Self {
        InfectionPlan { plan: vec![] }
    }
}

impl<P, ST> Task<Simulation<P, ST>> for InfectionPlan
where
    P: ParamSet<ST>,
    ST: HasEpiModel + Clone + Default,
    <<ST as HasEpiModel>::Model as EpiModel>::Clinical: Default,
{
    fn run(&mut self, ctx: &mut Simulation<P, ST>) {
        if let Some(n) = self.plan.pop() {
            ctx.population.contaminate_at_random(n, true, &mut ctx.rng);
        }
    }
}

/** SCHEDULED VACCINATIONS */
pub struct VaccinationPlan<V> {
    plan: Vec<(Age, Age, Vec<(V, usize)>)>,
}

impl<V> VaccinationPlan<V> {
    pub fn new() -> Self {
        VaccinationPlan { plan: vec![] }
    }
}

impl<P, M, V> Task<Simulation<P, SimpleAgent<M, V>>> for VaccinationPlan<V>
where
    M: EpiModel + HasAge,
    V: Clone,
    P: ParamSet<SimpleAgent<M, V>>,
{
    fn run(&mut self, ctx: &mut Simulation<P, SimpleAgent<M, V>>) {
        if let Some((start, end, groups)) = self.plan.pop() {
            let mut target_pop = vec![];

            for (id, ag) in ctx.population.iter().enumerate() {
                let age = ag.age();
                if age >= start && age <= end && ag.is_vaccinated() {
                    target_pop.push(id);
                }
            }
            target_pop.shuffle(&mut ctx.rng);

            for (vac, n) in groups {
                for _ in 1..n {
                    if let Some(id) = target_pop.pop() {
                        ctx.population
                            .get_agent_mut(id)
                            .map(|ag| ag.vaccinate(&vac));
                    } else {
                        return;
                    }
                }
            }
        }
    }
}
