use std::collections::HashMap;

use crate::prelude::Id;

/// The population trait describes a collection of agents.
pub trait Population {
    type State: Clone;

    /** Initialization *******************************************************/

    /// Creates population from a sequence of agents.
    fn new_population<'a, I>(states: I) -> Self
    where
        Self: 'a,
        Self::State: Clone,
        I: Iterator<Item = &'a Self::State>;

    /// Creates population with n copies of the given state.
    fn new_clones(n: usize, state: Self::State) -> Self
    where
        Self: Sized,
        Self::State: Clone,
    {
        return Population::new_population([state].iter().cycle().take(n));
    }

    /// Creates population with n copies of the default state.
    fn new_defaults(n: usize) -> Self
    where
        Self: Sized,
        Self::State: Default + Clone,
    {
        return Population::new_clones(n, Self::State::default());
    }

    /** Conversions and information ******************************************/

    /// Enumerate all individual states in population.
    fn to_states(&self) -> Vec<Self::State>
    where
        Self::State: Clone,
    {
        let mut vec = vec![];
        self.each_agent(&mut |_, a: &Self::State| vec.push(a.clone()));
        return vec;
    }

    /// Count the population size.
    fn count(&self) -> usize;

    /** Extract/modify individual agents *************************************/

    /// Get an agent by id.
    fn get_agent(&self, id: Id) -> Option<&Self::State>;

    /// Get mutable reference to agent by id.
    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State>;

    /// Get a pair of agents by id.
    fn get_pair(&self, i: Id, j: Id) -> Option<(&Self::State, &Self::State)> {
        match (self.get_agent(i), self.get_agent(j)) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }
    }

    /// Get a pair of agents by id.
    ///
    /// The option returns a reference to both agents. If the ids are the same, always
    /// return None.
    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)>;

    /// Get agents by ids. If you need to fetch multiple agents, it can be more
    /// convenient to use this.
    fn get_agents(&self, ids: impl IntoIterator<Item = Id>) -> Vec<(Id, &Self::State)> {
        let mut out = Vec::new();
        for id in ids.into_iter() {
            self.get_agent(id).map(|a| out.push((id, a)));
        }
        return out;
    }

    /// Set an agent state by id.
    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self
    where
        Self::State: Clone,
    {
        self.get_agent_mut(id).map(|st| *st = state.clone());
        return self;
    }

    /// Map function to agent.
    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.get_agent(id).map(f)
    }

    /// Map function to agent in slice.
    fn map_agents<B, F>(&self, ids: &[Id], f: F) -> HashMap<Id, B>
    where
        F: FnMut(Id, &Self::State) -> B,
    {
        let mut hash = HashMap::new();
        let mut g = f;
        ids.iter().for_each(|&id| {
            if let Some(ag) = self.get_agent(id) {
                hash.insert(id, g(id, ag));
            }
        });
        return hash;
    }

    /// Map function to agent in slice.
    fn map_agents_mut<B, F>(&mut self, ids: &[Id], f: F) -> HashMap<Id, B>
    where
        F: FnMut(Id, &mut Self::State) -> B,
    {
        let mut hash = HashMap::new();
        let mut g = f;
        ids.iter().for_each(|&id| {
            if let Some(ag) = self.get_agent_mut(id) {
                hash.insert(id, g(id, ag));
            }
        });
        return hash;
    }

    /// Map function to agent, mutating it.
    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.get_agent_mut(id).map(f)
    }

    /// Set multiple agent states by ids. If you need to update multiple
    /// agents, it can be more convenient to use this.
    fn set_agents(&mut self, updates: &[(Id, &Self::State)]) -> &mut Self
    where
        Self::State: Clone,
    {
        for &(id, state) in updates {
            self.set_agent(id, state);
        }
        return self;
    }

    /// Apply function to all agents of population. Function receives the Id
    /// and reference to State.
    fn each_agent_fail<E>(&self, f: impl FnMut(Id, &Self::State) -> Result<(), E>)
        -> Result<(), E>;

    /// Apply function to all agents of population. Function receives the Id
    /// and reference to State.
    fn each_agent(&self, f: impl FnMut(Id, &Self::State)) {
        let mut g = f;
        let _: Result<(), ()> = self.each_agent_fail(|id, st| {
            g(id, st);
            Ok(())
        });
    }

    /// Apply function to all agents of population. Function receives the Id
    /// and mutable reference to State.
    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State));
}

/// Simple trait for implementations that store states in a Vec of states.
///
/// It automatically provides Population implementations for instances of this
/// trait.
pub trait ContiguousPopulation {
    type Elem;

    /// Create from an owned vector of agents
    fn from_agent_vec(agents: Vec<Self::Elem>) -> Self;

    /// Return an immutable slice of agents
    fn as_state_slice(&self) -> &[Self::Elem];

    /// Return a mutable slice of agents
    fn as_state_mut_slice(&mut self) -> &mut [Self::Elem];
}

/////////////////////////////////////////////////////////////////////////////
// Implementations
/////////////////////////////////////////////////////////////////////////////
impl<P> Population for P
where
    P: ContiguousPopulation + Sized,
    P::Elem: Sized + Clone,
{
    type State = P::Elem;

    fn new_population<'a, I>(states: I) -> Self
    where
        Self: 'a,
        I: Iterator<Item = &'a Self::State>,
        Self::State: Clone,
    {
        let mut agents = vec![];
        for st in states {
            agents.push(st.clone());
        }
        Self::from_agent_vec(agents)
    }

    fn count(&self) -> usize {
        self.as_state_slice().len()
    }

    fn get_agent(&self, id: Id) -> Option<&Self::State> {
        self.as_state_slice().get(id)
    }

    fn get_agent_mut(&mut self, id: Id) -> Option<&mut Self::State> {
        self.as_state_mut_slice().get_mut(id)
    }

    fn each_agent(&self, f: impl FnMut(Id, &Self::State)) {
        let mut g = f;

        for (id, st) in self.as_state_slice().iter().enumerate() {
            g(id, st);
        }
    }

    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State)) {
        let mut g = f;

        for (id, st) in self.as_state_mut_slice().iter_mut().enumerate() {
            g(id, st);
        }
    }

    fn get_pair_mut(&mut self, i: Id, j: Id) -> Option<(&mut Self::State, &mut Self::State)> {
        let slice = self.as_state_mut_slice();
        let n = slice.len();
        if i == j || i >= n || j >= n {
            return None;
        } else {
            // Safety: we can have two mutable borrows to elements of the slice
            // since the previous line guarantees that elements are not the same
            unsafe {
                let a = &mut *(slice.get_unchecked_mut(i) as *mut _);
                let b = &mut *(slice.get_unchecked_mut(j) as *mut _);
                return Some((a, b));
            }
        }
    }

    fn each_agent_fail<E>(
        &self,
        f: impl FnMut(Id, &Self::State) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut g = f;

        for (id, st) in self.as_state_slice().iter().enumerate() {
            g(id, st)?;
        }
        return Ok(());
    }
}

impl<S> ContiguousPopulation for Vec<S>
where
    S: Clone,
{
    type Elem = S;

    fn from_agent_vec(states: Vec<S>) -> Self {
        return states;
    }

    fn as_state_slice(&self) -> &[S] {
        self.as_slice()
    }

    fn as_state_mut_slice(&mut self) -> &mut [S] {
        self.as_mut_slice()
    }
}

impl Population for () {
    type State = ();

    fn new_population<'a, I>(_states: I) -> Self
    where
        Self: 'a,
        Self::State: Clone,
        I: Iterator<Item = &'a Self::State>,
    {
        return ();
    }

    fn count(&self) -> usize {
        return 0;
    }

    fn get_agent(&self, _id: Id) -> Option<&Self::State> {
        return None;
    }

    fn get_agent_mut(&mut self, _id: Id) -> Option<&mut Self::State> {
        return None;
    }

    fn get_pair_mut(&mut self, _i: Id, _j: Id) -> Option<(&mut Self::State, &mut Self::State)> {
        return None;
    }

    fn each_agent_mut(&mut self, _f: impl FnMut(Id, &mut Self::State)) {
        return;
    }

    fn each_agent_fail<E>(
        &self,
        _f: impl FnMut(Id, &Self::State) -> Result<(), E>,
    ) -> Result<(), E> {
        return Ok(());
    }
}
