use rand::prelude::*;

/*
/// Compute R0 from iterator over agents and the number of secondary infections
/// produced by each agent.
pub fn r0<M: EpiModel>(it: impl IntoIterator<Item = (usize, M)>) -> Real {
    let mut total = 0;
    let mut acc = 0;
    for pair in it {
        match pair {
            (0, st) => {
                if st.is_recovered() {
                    total += 1;
                }
            }
            (n, _) => {
                total += 1;
                acc += n;
            }
        }
    }
    return (acc as Real) / (total as Real);
}
 */

/// Default random number generator
pub fn default_rng() -> SmallRng {
    SmallRng::from_entropy()
}

/// Default random number generator with numeric seed
pub fn seeded_rng(n: impl Into<u64>) -> SmallRng {
    SmallRng::seed_from_u64(n.into())
}
