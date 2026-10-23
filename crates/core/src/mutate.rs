//! The RND button: mutate every unlocked parameter.

use crate::params::{NUM_PARAMS, PARAMS};

/// Small deterministic PRNG (xorshift64*), good enough for mutations.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Approximately standard normal (Irwin-Hall, n = 4).
    pub fn gauss(&mut self) -> f32 {
        let s: f32 = (0..4).map(|_| self.f32()).sum();
        (s - 2.0) * 1.732
    }
}

/// Mutate `values` in place. `amount` is 0..1: 0 does nothing, 1 is close
/// to a full randomisation. Locked parameters are never touched.
pub fn mutate(values: &mut [f32; NUM_PARAMS], locked: &[bool; NUM_PARAMS], amount: f32, rng: &mut Rng) {
    let amount = amount.clamp(0.0, 1.0);
    if amount <= 0.0 {
        return;
    }
    for (i, info) in PARAMS.iter().enumerate() {
        if locked[i] {
            continue;
        }
        let n = info.to_norm(values[i]);
        let new_n = if info.is_discrete() {
            if rng.f32() < amount * 0.6 { rng.f32() } else { n }
        } else if rng.f32() < amount * amount * 0.25 {
            // Occasional wild jump so RND can escape the current neighbourhood.
            rng.f32()
        } else {
            n + rng.gauss() * amount * 0.35
        };
        values[i] = info.from_norm(reflect01(new_n));
    }
}

/// Fold a value back into 0..1 rather than clamping, so mutations don't pile
/// up at the extremes.
fn reflect01(mut n: f32) -> f32 {
    for _ in 0..4 {
        if n < 0.0 {
            n = -n;
        } else if n > 1.0 {
            n = 2.0 - n;
        } else {
            break;
        }
    }
    n.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{default_values, P};

    #[test]
    fn locked_params_never_change() {
        let mut rng = Rng::new(42);
        let mut locked = [false; NUM_PARAMS];
        locked[P::Metal.idx()] = true;
        locked[P::FilterType.idx()] = true;
        let base = default_values();
        let mut v = base;
        for _ in 0..200 {
            mutate(&mut v, &locked, 1.0, &mut rng);
            assert_eq!(v[P::Metal.idx()], base[P::Metal.idx()]);
            assert_eq!(v[P::FilterType.idx()], base[P::FilterType.idx()]);
        }
    }

    #[test]
    fn unlocked_params_change_and_stay_in_range() {
        let mut rng = Rng::new(7);
        let locked = [false; NUM_PARAMS];
        let base = default_values();
        let mut v = base;
        mutate(&mut v, &locked, 0.5, &mut rng);
        let changed = (0..NUM_PARAMS).filter(|&i| v[i] != base[i]).count();
        assert!(changed > NUM_PARAMS / 2, "only {changed} changed");
        for _ in 0..500 {
            mutate(&mut v, &locked, 0.8, &mut rng);
            for (i, p) in PARAMS.iter().enumerate() {
                assert!(v[i] >= p.min && v[i] <= p.max, "{} = {}", p.key, v[i]);
            }
        }
    }

    #[test]
    fn zero_amount_is_identity() {
        let mut rng = Rng::new(1);
        let mut v = default_values();
        mutate(&mut v, &[false; NUM_PARAMS], 0.0, &mut rng);
        assert_eq!(v, default_values());
    }
}
