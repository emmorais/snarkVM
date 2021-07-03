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
use snarkvm_curves::{bls12_377::{Fq, G1Affine}, AffineCurve};
use snarkvm_fields::{Field, Fp2};
use snarkvm_curves::bls12_377::{G2Affine, Fq2Parameters};
use snarkvm_utilities::UniformRand;

pub fn hash_to_curve<G: AffineCurve<BaseField = F>, F: Field>(input: &str) -> G {
    let mut rng = ChaCha8Rng::from_seed(sha256(input.as_bytes()));
    let mut x = G::BaseField::rand(&mut rng);
    loop {
        for bit_y in [false, true] {
            if let Some(g) = G::from_x_coordinate(x.clone(), bit_y) {
                let g = g.mul_by_cofactor();
                if !g.is_zero() {
                    return g;
                }
            }
        }
        x.add_assign(&F::one());
    }
}

#[test]
fn hash_bls12_377_g1() {
    let g1 = hash_to_curve::<G1Affine, Fq>("Aleo BLS12-377 G1");
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

#[test]
fn hash_bls12_377_g2() {
    let g2 = hash_to_curve::<G2Affine, Fp2<Fq2Parameters>>("Aleo BLS12-377 G2");
    assert!(g2.is_in_correct_subgroup_assuming_on_curve());
    assert_eq!(
        g2.x.to_string(),
        "Fp2(4992399534085505417963743334831158275796011776806012206913627929721152622217262086900539913823010591400808061223857 + 1036762444117213072465519435695624108541353692644436531612240142717350874959576322311161822617113219647751577789565 * u)"
    );
    assert_eq!(
        g2.y.to_string(),
        "Fp2(39593503541928207112672510593382719635487710939976849056149175312270166418672211148604934770065425690518397740619 + 89614077544256851417964173631152027483300723040827568531499279961411912919170578736548531738044390218650282162770 * u)"
    );
}
