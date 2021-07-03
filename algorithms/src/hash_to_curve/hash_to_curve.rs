// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::crh::sha256::sha256;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use snarkvm_curves::{
    bls12_377::{Fq, G1Affine},
    AffineCurve,
};
use snarkvm_fields::{One, Zero};
use snarkvm_utilities::UniformRand;
use std::ops::AddAssign;

pub fn hash_to_curve(input: &str) -> G1Affine {
    let mut rng = ChaCha8Rng::from_seed(sha256(input.as_bytes()));
    let mut x = Fq::rand(&mut rng);
    loop {
        for bit_y in [false, true] {
            if let Some(g1_proj) = G1Affine::from_x_coordinate(x.clone(), bit_y) {
                let g1_proj = g1_proj.scale_by_cofactor();
                if !g1_proj.is_zero() {
                    let g1 = G1Affine::from(g1_proj);
                    return g1;
                }
            }
        }
        x.add_assign(&Fq::one());
    }
}

#[test]
fn hash_bls12_377() {
    let g1 = hash_to_curve("Aleo BLS12-377 G1");
    assert!(g1.is_in_correct_subgroup_assuming_on_curve());
    assert_eq!(
        g1.x.to_string(),
        "55965310611925736182344266514489161724026037910766990800227254498080679430741845649557376683529002177041434636955"
    );
    assert_eq!(
        g1.y.to_string(),
        "47988536770352052135994920271989708481287122349204081233627486611634010967671014168936729210354715817984213346861761"
    );
}
