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

use super::{GroupAffine, GroupProjective};

use snarkvm_utilities::{
    bytes::ToBytes,
    io::Cursor,
    rand::UniformRand,
    serialize::{CanonicalDeserialize, CanonicalSerialize},
    to_bytes,
};

use crate::traits::{
    pairing_engine::{AffineCurve, ProjectiveCurve},
    MontgomeryModelParameters,
    TEModelParameters,
};
use snarkvm_fields::{Field, One, PrimeField, Zero};

use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

pub const ITERATIONS: usize = 10;

pub fn montgomery_conversion_test<P>()
where
    P: TEModelParameters,
{
    // A = 2 * (a + d) / (a - d)
    let a = P::BaseField::one().double() * (P::COEFF_A + P::COEFF_D) * (P::COEFF_A - P::COEFF_D).inverse().unwrap();
    // B = 4 / (a - d)
    let b = P::BaseField::one().double().double() * (P::COEFF_A - P::COEFF_D).inverse().unwrap();

    assert_eq!(a, P::MontgomeryModelParameters::COEFF_A);
    assert_eq!(b, P::MontgomeryModelParameters::COEFF_B);
}

pub fn edwards_test<P: TEModelParameters>()
where
    P::BaseField: PrimeField,
{
    edwards_curve_serialization_test::<P>();
    edwards_from_random_bytes::<P>();
    edwards_from_x_and_y_coordinates::<P>();
}

pub fn edwards_curve_serialization_test<P: TEModelParameters>() {
    let buf_size = GroupAffine::<P>::zero().serialized_size();

    let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

    for _ in 0..10 {
        let a = GroupProjective::<P>::rand(&mut rng);
        let a = a.into_affine();
        {
            let mut serialized = vec![0; buf_size];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize(&mut cursor).unwrap();

            let mut cursor = Cursor::new(&serialized[..]);
            let b = GroupAffine::<P>::deserialize(&mut cursor).unwrap();
            assert_eq!(a, b);
        }

        {
            let a = GroupAffine::<P>::zero();
            let mut serialized = vec![0; buf_size];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize(&mut cursor).unwrap();
            let mut cursor = Cursor::new(&serialized[..]);
            let b = GroupAffine::<P>::deserialize(&mut cursor).unwrap();
            assert_eq!(a, b);
        }

        {
            let a = GroupAffine::<P>::zero();
            let mut serialized = vec![0; buf_size - 1];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize(&mut cursor).unwrap_err();
        }

        {
            let serialized = vec![0; buf_size - 1];
            let mut cursor = Cursor::new(&serialized[..]);
            GroupAffine::<P>::deserialize(&mut cursor).unwrap_err();
        }

        {
            let mut serialized = vec![0; a.uncompressed_size()];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize_uncompressed(&mut cursor).unwrap();

            let mut cursor = Cursor::new(&serialized[..]);
            let b = GroupAffine::<P>::deserialize_uncompressed(&mut cursor).unwrap();
            assert_eq!(a, b);
        }

        {
            let a = GroupAffine::<P>::zero();
            let mut serialized = vec![0; a.uncompressed_size()];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize_uncompressed(&mut cursor).unwrap();
            let mut cursor = Cursor::new(&serialized[..]);
            let b = GroupAffine::<P>::deserialize_uncompressed(&mut cursor).unwrap();
            assert_eq!(a, b);
        }
    }
}

pub fn edwards_from_random_bytes<P: TEModelParameters>()
where
    P::BaseField: PrimeField,
{
    let buf_size = GroupAffine::<P>::zero().serialized_size();

    let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

    for _ in 0..ITERATIONS {
        let a = GroupProjective::<P>::rand(&mut rng);
        let a = a.into_affine();
        {
            let mut serialized = vec![0; buf_size];
            let mut cursor = Cursor::new(&mut serialized[..]);
            a.serialize(&mut cursor).unwrap();

            let mut cursor = Cursor::new(&serialized[..]);
            let p1 = GroupAffine::<P>::deserialize(&mut cursor).unwrap();
            let p2 = GroupAffine::<P>::from_random_bytes(&serialized).unwrap();
            assert_eq!(p1, p2);
        }
    }

    for _ in 0..ITERATIONS {
        let biginteger = <<GroupAffine<P> as AffineCurve>::BaseField as PrimeField>::BigInteger::rand(&mut rng);
        let mut bytes = to_bytes![biginteger].unwrap();
        let mut g = GroupAffine::<P>::from_random_bytes(&bytes);
        while g.is_none() {
            bytes.iter_mut().for_each(|i| *i = i.wrapping_sub(1));
            g = GroupAffine::<P>::from_random_bytes(&bytes);
        }
        let _g = g.unwrap();
    }
}

pub fn edwards_from_x_and_y_coordinates<P: TEModelParameters>()
where
    P::BaseField: PrimeField,
{
    let mut rng = XorShiftRng::seed_from_u64(1231275789u64);

    for _ in 0..ITERATIONS {
        let a = GroupProjective::<P>::rand(&mut rng);
        let a = a.into_affine();
        {
            let x = a.x;

            let a1 = GroupAffine::<P>::from_x_coordinate(x, true).unwrap();
            let a2 = GroupAffine::<P>::from_x_coordinate(x, false).unwrap();

            assert!(a == a1 || a == a2);
        }
        {
            let y = a.y;

            let a1 = GroupAffine::<P>::from_y_coordinate(y, true).unwrap();
            let a2 = GroupAffine::<P>::from_y_coordinate(y, false).unwrap();

            assert!(a == a1 || a == a2);
        }
    }
}
