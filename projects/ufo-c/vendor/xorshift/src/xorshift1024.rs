// Written by Alexander Stocko <as@coder.gg>
//
// To the extent possible under law, the author has dedicated all copyright
// and related and neighboring rights to this software to the public domain
// worldwide. This software is distributed without any warranty.
//
// See <LICENSE or http://creativecommons.org/publicdomain/zero/1.0/>

//! The Xorshift1024* random number generator.

#![cfg_attr(feature = "cargo-clippy", allow(needless_range_loop))]

use std::num::Wrapping as w;

use rand::{Rand, Rng, SeedableRng};

use RngJump;

const STATE_SIZE: usize = 16;

/// A random number generator that uses the xorshift1024* algorithm [1].
///
/// # Description
/// Quoted from [1].
///
/// This is a fast, top-quality generator. If 1024 bits of state are too
/// much, try a xoroshiro128+ generator.
///
/// Note that the three lowest bits of this generator are LSFRs, and thus
/// they are slightly less random than the other bits. We suggest to use a
/// sign test to extract a random Boolean value.
///
/// The state must be seeded so that it is not everywhere zero. If you have
/// a 64-bit seed, we suggest to seed a splitmix64 generator and use its
/// output to fill s.
///
/// [1]: Sebastiano Vigna, [xorshift1024*]
/// (http://xoroshiro.di.unimi.it/xorshift1024star.c)
///
/// # Parallelism
/// The `RngJump` implementation is equivalent to 2^512 calls to `next_u64`().
/// Used to generate 2^512 non-overlapping subsequences for parallel
/// computations.
#[derive(Clone, Copy)]
pub struct Xorshift1024 {
    state: [u64; 16],
    p: usize,
}

static EMPTY: Xorshift1024 = Xorshift1024 {
    state: [0; 16],
    p: 0,
};
static JUMP: [u64; 16] = [0x84242f96eca9c41d,
                          0xa3c65b8776f96855,
                          0x5b34a39f070b5837,
                          0x4489affce4f31a1e,
                          0x2ffeeb0a48316f40,
                          0xdc2d9891fe68c022,
                          0x3659132bb12fea70,
                          0xaac17d8efa43cab8,
                          0xc4cb815590989b13,
                          0x5ee975283d71c93b,
                          0x691548c86c1bd540,
                          0x7910c41d10a1e6a5,
                          0x0b5fc64563b3e2a8,
                          0x047f7684e9fc949d,
                          0xb99181f2d8f685ca,
                          0x284600e3f30e38c3];



impl Rng for Xorshift1024 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let s0 = self.state[self.p];
        self.p = (self.p + 1) & 15;
        let mut s1 = self.state[self.p];

        s1 ^= s1 << 31;
        self.state[self.p] = s1 ^ s0 ^ (s1 >> 11) ^ (s0 >> 30);

        (w(self.state[self.p]) * w(1181783497276652981_u64)).0
    }
}

impl<'a> SeedableRng<&'a [u64]> for Xorshift1024 {
    fn reseed(&mut self, seed: &'a [u64]) {
        if seed.len() < 16 {
            panic!("Xorshift1024 seed needs at least 16 u64s for seeding.");
        }

        for (index, element) in seed.iter().enumerate() {
            self.state[index] = *element;
        }
    }

    fn from_seed(seed: &'a [u64]) -> Xorshift1024 {
        let mut rng = EMPTY;
        rng.reseed(seed);
        rng
    }
}

impl Rand for Xorshift1024 {
    fn rand<R: Rng>(other: &mut R) -> Xorshift1024 {
        let mut key: [u64; STATE_SIZE] = [0; STATE_SIZE];
        for word in &mut key {
            *word = other.gen();
        }
        SeedableRng::from_seed(&key[..])
    }
}

impl RngJump for Xorshift1024 {
    fn jump(&mut self, count: usize) {
        for _ in 0..count {
            let mut t: [u64; 16] = [0; 16];
            for i in &JUMP {
                for b in 0..64 {
                    if (i & 1 << b) != 0 {
                        for (j, t_elem) in t.iter_mut().enumerate().take(16) {
                            *t_elem ^= self.state[(j + self.p) & 15];
                        }
                    }
                    self.next_u64();
                }
            }

            for j in 0..16 {
                self.state[(j + self.p) & 15] = t[j];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use super::Xorshift1024;
    #[test]
    fn test() {
        // Calculated from reference implementation
        // https://github.com/astocko/xorshift-cpp
        let seed: u64 = 1477777179826044140;
        let t_vals: Vec<u64> = vec![14360464905097655832,
                                    10515520027797512354,
                                    12277485841648819968,
                                    5975068082386226908,
                                    14360464905097655832,
                                    10515520027797512354,
                                    12277485841648819968,
                                    5975068082386226908,
                                    14360464905097655832,
                                    10515520027797512354,
                                    12277485841648819968,
                                    5975068082386226908,
                                    14360464905097655832,
                                    10515520027797512354,
                                    12277485841648819968,
                                    5975068082386226908,
                                    16155457212423715006,
                                    16973689320641693688,
                                    11981506001797128964,
                                    13241400995114197981,
                                    2158488016667357978,
                                    3377935610872016481,
                                    12277485841648819968,
                                    5975068082386226908,
                                    16155457212423715006,
                                    16973689320641693688,
                                    11981506001797128964,
                                    13241400995114197981,
                                    2158488016667357978,
                                    3377935610872016481,
                                    12277485841648819968,
                                    5975068082386226908,
                                    3862476215600981850,
                                    666405138486472370,
                                    2467704680056122713,
                                    18070567468833369740,
                                    14135306694933672725,
                                    3377935610872016481,
                                    12277485841648819968,
                                    5975068082386226908,
                                    3862476215600981850,
                                    666405138486472370,
                                    2467704680056122713,
                                    18070567468833369740,
                                    14135306694933672725,
                                    3377935610872016481,
                                    12277485841648819968,
                                    5975068082386226908,
                                    812945179660782235,
                                    14943324017293890156];

        let states = [seed; 16];
        let mut rng: Xorshift1024 = SeedableRng::from_seed(&states[..]);
        let vals = rng.gen_iter::<u64>().take(t_vals.len()).collect::<Vec<u64>>();
        assert!(::test::iter_eq(t_vals, vals));
    }
}
