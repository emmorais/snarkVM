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

use crate::{
    impl_edwards_curve_serializer,
    traits::{
        AffineCurve,
        Group,
        MontgomeryModelParameters as MontgomeryParameters,
        ProjectiveCurve,
        TEModelParameters as Parameters,
    },
};
use snarkvm_fields::{impl_additive_ops_from_ref, Field, One, PrimeField, SquareRootField, Zero};
use snarkvm_utilities::{
    bititerator::BitIteratorBE,
    bytes::{FromBytes, ToBytes},
    rand::UniformRand,
    serialize::*,
};

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::{Read, Result as IoResult, Write},
    marker::PhantomData,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub mod tests;
use serde::{Deserialize, Serialize};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Copy(bound = "P: Parameters"),
    Clone(bound = "P: Parameters"),
    PartialEq(bound = "P: Parameters"),
    Eq(bound = "P: Parameters"),
    Debug(bound = "P: Parameters"),
    Hash(bound = "P: Parameters")
)]
pub struct GroupAffine<P: Parameters> {
    pub x: P::BaseField,
    pub y: P::BaseField,
    #[derivative(Debug = "ignore")]
    _params: PhantomData<P>,
}

impl<P: Parameters> Display for GroupAffine<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "GroupAffine(x={}, y={})", self.x, self.y)
    }
}

impl<P: Parameters> GroupAffine<P> {
    pub fn new(x: P::BaseField, y: P::BaseField) -> Self {
        Self {
            x,
            y,
            _params: PhantomData,
        }
    }

    #[must_use]
    pub fn scale_by_cofactor(&self) -> <Self as AffineCurve>::Projective {
        self.mul_bits(BitIteratorBE::new(P::COFACTOR))
    }

    /// Checks that the current point is on the elliptic curve.
    pub fn is_on_curve(&self) -> bool {
        let x2 = self.x.square();
        let y2 = self.y.square();

        let lhs = y2 + P::mul_by_a(&x2);
        let rhs = P::BaseField::one() + (P::COEFF_D * (x2 * y2));

        lhs == rhs
    }
}

impl<P: Parameters> Zero for GroupAffine<P> {
    fn zero() -> Self {
        Self::new(P::BaseField::zero(), P::BaseField::one())
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() & self.y.is_one()
    }
}

impl<P: Parameters> AffineCurve for GroupAffine<P> {
    type BaseField = P::BaseField;
    type Projective = GroupProjective<P>;

    fn prime_subgroup_generator() -> Self {
        Self::new(P::AFFINE_GENERATOR_COEFFS.0, P::AFFINE_GENERATOR_COEFFS.1)
    }

    /// Attempts to construct an affine point given an x-coordinate. The
    /// point is not guaranteed to be in the prime order subgroup.
    ///
    /// If and only if `greatest` is set will the lexicographically
    /// largest y-coordinate be selected.
    fn from_x_coordinate(x: Self::BaseField, greatest: bool) -> Option<Self> {
        // y = sqrt( (a * x^2 - 1)  / (d * x^2 - 1) )
        let x2 = x.square();
        let one = Self::BaseField::one();
        let numerator = P::mul_by_a(&x2) - one;
        let denominator = P::COEFF_D * x2 - one;
        let y2 = denominator.inverse().map(|denom| denom * numerator);
        y2.and_then(|y2| y2.sqrt()).map(|y| {
            let negy = -y;
            let y = if (y < negy) ^ greatest { y } else { negy };
            Self::new(x, y)
        })
    }

    /// Attempts to construct an affine point given a y-coordinate. The
    /// point is not guaranteed to be in the prime order subgroup.
    ///
    /// If and only if `greatest` is set will the lexicographically
    /// largest y-coordinate be selected.
    fn from_y_coordinate(y: Self::BaseField, greatest: bool) -> Option<Self> {
        // x = sqrt( (1 - y^2) / (a - d * y^2) )
        let y2 = y.square();
        let one = Self::BaseField::one();
        let numerator = one - y2;
        let denominator = P::mul_by_a(&one) - (P::COEFF_D * y2);
        let x2 = denominator.inverse().map(|denom| denom * numerator);
        x2.and_then(|x2| x2.sqrt()).map(|x| {
            let negx = -x;
            let x = if (x < negx) ^ greatest { x } else { negx };
            Self::new(x, y)
        })
    }

    // Copied from https://github.com/scipr-lab/zexe/blob/4b3f08c6c0a08c5392ed8aa3fd3c32f28da402c4/algebra-core/src/curves/models/twisted_edwards_extended.rs#L144-L156.
    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        let x = P::BaseField::from_random_bytes_with_flags(bytes);
        if let Some((x, flags)) = x {
            let parsed_flags = EdwardsFlags::from_u8(flags);
            if x.is_zero() {
                Some(Self::zero())
            } else {
                Self::from_x_coordinate(x, parsed_flags.is_positive())
            }
        } else {
            None
        }
    }

    fn mul_bits<S: AsRef<[u64]>>(&self, bits: BitIteratorBE<S>) -> <Self as AffineCurve>::Projective {
        let mut res = GroupProjective::zero();
        for i in bits {
            res.double_in_place();
            if i {
                res.add_assign_mixed(self)
            }
        }
        res
    }

    fn mul_by_cofactor_to_projective(&self) -> Self::Projective {
        self.scale_by_cofactor()
    }

    fn mul_by_cofactor_inv(&self) -> Self {
        self.mul(P::COFACTOR_INV).into()
    }

    fn into_projective(&self) -> GroupProjective<P> {
        (*self).into()
    }

    fn is_in_correct_subgroup_assuming_on_curve(&self) -> bool {
        self.mul_bits(BitIteratorBE::new(P::ScalarField::characteristic()))
            .is_zero()
    }

    fn to_x_coordinate(&self) -> Self::BaseField {
        self.x
    }

    fn to_y_coordinate(&self) -> Self::BaseField {
        self.y
    }

    /// Checks that the current point is on the elliptic curve.
    fn is_on_curve(&self) -> bool {
        let x2 = self.x.square();
        let y2 = self.y.square();

        let lhs = y2 + P::mul_by_a(&x2);
        let rhs = P::BaseField::one() + (P::COEFF_D * (x2 * y2));

        lhs == rhs
    }
}

impl<P: Parameters> Group for GroupAffine<P> {
    type ScalarField = P::ScalarField;

    #[inline]
    #[must_use]
    fn double(&self) -> Self {
        let mut tmp = *self;
        tmp += self;
        tmp
    }

    #[inline]
    fn double_in_place(&mut self) {
        let tmp = *self;
        *self = tmp.double();
    }
}

impl<P: Parameters> Neg for GroupAffine<P> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, self.y)
    }
}

impl_additive_ops_from_ref!(GroupAffine, Parameters);

impl<'a, P: Parameters> Add<&'a Self> for GroupAffine<P> {
    type Output = Self;

    fn add(self, other: &'a Self) -> Self {
        let mut copy = self;
        copy += other;
        copy
    }
}

impl<'a, P: Parameters> AddAssign<&'a Self> for GroupAffine<P> {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: &'a Self) {
        let y1y2 = self.y * other.y;
        let x1x2 = self.x * other.x;
        let dx1x2y1y2 = P::COEFF_D * y1y2 * x1x2;

        let d1 = P::BaseField::one() + dx1x2y1y2;
        let d2 = P::BaseField::one() - dx1x2y1y2;

        let x1y2 = self.x * other.y;
        let y1x2 = self.y * other.x;

        self.x = (x1y2 + y1x2) / &d1;
        self.y = (y1y2 - P::mul_by_a(&x1x2)) / &d2;
    }
}

impl<'a, P: Parameters> Sub<&'a Self> for GroupAffine<P> {
    type Output = Self;

    fn sub(self, other: &'a Self) -> Self {
        let mut copy = self;
        copy -= other;
        copy
    }
}

impl<'a, P: Parameters> SubAssign<&'a Self> for GroupAffine<P> {
    fn sub_assign(&mut self, other: &'a Self) {
        *self += &(-(*other));
    }
}

impl<P: Parameters> Mul<P::ScalarField> for GroupAffine<P> {
    type Output = Self;

    fn mul(self, other: P::ScalarField) -> Self {
        self.mul_bits(BitIteratorBE::new(other.into_repr())).into()
    }
}

impl<P: Parameters> MulAssign<P::ScalarField> for GroupAffine<P> {
    fn mul_assign(&mut self, other: P::ScalarField) {
        *self = self.mul(other).into()
    }
}

impl<P: Parameters> ToBytes for GroupAffine<P> {
    #[inline]
    fn write<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.x.write(&mut writer)?;
        self.y.write(&mut writer)
    }
}

impl<P: Parameters> FromBytes for GroupAffine<P> {
    #[inline]
    fn read<R: Read>(mut reader: R) -> IoResult<Self> {
        let x = P::BaseField::read(&mut reader)?;
        let y = P::BaseField::read(&mut reader)?;
        Ok(Self::new(x, y))
    }
}

impl<P: Parameters> Default for GroupAffine<P> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<P: Parameters> Distribution<GroupAffine<P>> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GroupAffine<P> {
        loop {
            let x = P::BaseField::rand(rng);
            let greatest = rng.gen();

            if let Some(p) = GroupAffine::from_x_coordinate(x, greatest) {
                return p.scale_by_cofactor().into();
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Derivative)]
#[derivative(
    Copy(bound = "P: Parameters"),
    Clone(bound = "P: Parameters"),
    Eq(bound = "P: Parameters"),
    Debug(bound = "P: Parameters"),
    Hash(bound = "P: Parameters")
)]
pub struct GroupProjective<P: Parameters> {
    pub x: P::BaseField,
    pub y: P::BaseField,
    pub t: P::BaseField,
    pub z: P::BaseField,
    #[derivative(Debug = "ignore")]
    _params: PhantomData<P>,
}

impl<P: Parameters> Display for GroupProjective<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.into_affine())
    }
}

impl<P: Parameters> PartialEq for GroupProjective<P> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_zero() {
            return other.is_zero();
        }

        if other.is_zero() {
            return false;
        }

        // x1/z1 == x2/z2  <==> x1 * z2 == x2 * z1
        (self.x * other.z) == (other.x * self.z) && (self.y * other.z) == (other.y * self.z)
    }
}

impl<P: Parameters> Distribution<GroupProjective<P>> for Standard {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GroupProjective<P> {
        loop {
            let x = P::BaseField::rand(rng);
            let greatest = rng.gen();

            if let Some(p) = GroupAffine::from_x_coordinate(x, greatest) {
                return p.scale_by_cofactor();
            }
        }
    }
}

impl<P: Parameters> ToBytes for GroupProjective<P> {
    #[inline]
    fn write<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.x.write(&mut writer)?;
        self.y.write(&mut writer)?;
        self.t.write(&mut writer)?;
        self.z.write(writer)
    }
}

impl<P: Parameters> FromBytes for GroupProjective<P> {
    #[inline]
    fn read<R: Read>(mut reader: R) -> IoResult<Self> {
        let x = P::BaseField::read(&mut reader)?;
        let y = P::BaseField::read(&mut reader)?;
        let t = P::BaseField::read(&mut reader)?;
        let z = P::BaseField::read(reader)?;
        Ok(Self::new(x, y, t, z))
    }
}

impl<P: Parameters> Default for GroupProjective<P> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<P: Parameters> GroupProjective<P> {
    pub fn new(x: P::BaseField, y: P::BaseField, t: P::BaseField, z: P::BaseField) -> Self {
        Self {
            x,
            y,
            t,
            z,
            _params: PhantomData,
        }
    }
}

impl<P: Parameters> Zero for GroupProjective<P> {
    fn zero() -> Self {
        Self::new(
            P::BaseField::zero(),
            P::BaseField::one(),
            P::BaseField::zero(),
            P::BaseField::one(),
        )
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y == self.z && !self.y.is_zero() && self.t.is_zero()
    }
}

impl<P: Parameters> ProjectiveCurve for GroupProjective<P> {
    type Affine = GroupAffine<P>;
    type BaseField = P::BaseField;

    fn prime_subgroup_generator() -> Self {
        GroupAffine::prime_subgroup_generator().into()
    }

    fn is_normalized(&self) -> bool {
        self.z.is_one()
    }

    fn batch_normalization(v: &mut [Self]) {
        // Montgomery’s Trick and Fast Implementation of Masked AES
        // Genelle, Prouff and Quisquater
        // Section 3.2

        // First pass: compute [a, ab, abc, ...]
        let mut prod = Vec::with_capacity(v.len());
        let mut tmp = P::BaseField::one();
        for g in v
            .iter_mut()
            // Ignore normalized elements
            .filter(|g| !g.is_normalized())
        {
            tmp.mul_assign(&g.z);
            prod.push(tmp);
        }

        // Invert `tmp`.
        tmp = tmp.inverse().unwrap(); // Guaranteed to be nonzero.

        // Second pass: iterate backwards to compute inverses
        for (g, s) in v
            .iter_mut()
            // Backwards
            .rev()
            // Ignore normalized elements
            .filter(|g| !g.is_normalized())
            // Backwards, skip last element, fill in one for last term.
            .zip(
                prod.into_iter()
                    .rev()
                    .skip(1)
                    .chain(Some(P::BaseField::one())),
            )
        {
            // tmp := tmp * g.z; g.z := tmp * s = 1/z
            let newtmp = tmp * g.z;
            g.z = tmp * s;
            tmp = newtmp;
        }

        // Perform affine transformations
        for g in v.iter_mut().filter(|g| !g.is_normalized()) {
            g.x *= &g.z; // x/z
            g.y *= &g.z;
            g.t *= &g.z;
            g.z = P::BaseField::one(); // z = 1
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn add_assign_mixed(&mut self, other: &Self::Affine) {
        // A = X1*X2
        let a = self.x * other.x;
        // B = Y1*Y2
        let b = self.y * other.y;
        // C = T1*d*T2
        let c = P::COEFF_D * self.t * other.x * other.y;
        // D = Z1
        let d = self.z;
        // E = (X1+Y1)*(X2+Y2)-A-B
        let e = (self.x + self.y) * (other.x + other.y) - a - b;
        // F = D-C
        let f = d - c;
        // G = D+C
        let g = d + c;
        // H = B-a*A
        let h = b - P::mul_by_a(&a);
        // X3 = E*F
        self.x = e * f;
        // Y3 = G*H
        self.y = g * h;
        // T3 = E*H
        self.t = e * h;
        // Z3 = F*G
        self.z = f * g;
    }

    fn into_affine(&self) -> GroupAffine<P> {
        (*self).into()
    }

    fn recommended_wnaf_for_scalar(scalar: <Self::ScalarField as PrimeField>::BigInteger) -> usize {
        P::empirical_recommended_wnaf_for_scalar(scalar)
    }

    fn recommended_wnaf_for_num_scalars(num_scalars: usize) -> usize {
        P::empirical_recommended_wnaf_for_num_scalars(num_scalars)
    }
}

impl<P: Parameters> Group for GroupProjective<P> {
    type ScalarField = P::ScalarField;

    #[inline]
    #[must_use]
    fn double(&self) -> Self {
        let mut tmp = *self;
        tmp += self;
        tmp
    }

    #[inline]
    fn double_in_place(&mut self) {
        let tmp = *self;
        *self = tmp.double();
    }
}

impl<P: Parameters> Neg for GroupProjective<P> {
    type Output = Self;

    fn neg(mut self) -> Self {
        self.x = -self.x;
        self.t = -self.t;
        self
    }
}

impl_additive_ops_from_ref!(GroupProjective, Parameters);

impl<'a, P: Parameters> Add<&'a Self> for GroupProjective<P> {
    type Output = Self;

    fn add(self, other: &'a Self) -> Self {
        let mut copy = self;
        copy += other;
        copy
    }
}

impl<'a, P: Parameters> AddAssign<&'a Self> for GroupProjective<P> {
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, other: &'a Self) {
        // See "Twisted Edwards Curves Revisited"
        // Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        // 3.1 Unified Addition in E^e

        // A = x1 * x2
        let a = self.x * other.x;

        // B = y1 * y2
        let b = self.y * other.y;

        // C = d * t1 * t2
        let c = P::COEFF_D * self.t * other.t;

        // D = z1 * z2
        let d = self.z * other.z;

        // H = B - aA
        let h = b - P::mul_by_a(&a);

        // E = (x1 + y1) * (x2 + y2) - A - B
        let e = (self.x + self.y) * (other.x + other.y) - a - b;

        // F = D - C
        let f = d - c;

        // G = D + C
        let g = d + c;

        // x3 = E * F
        self.x = e * f;

        // y3 = G * H
        self.y = g * h;

        // t3 = E * H
        self.t = e * h;

        // z3 = F * G
        self.z = f * g;
    }
}

impl<'a, P: Parameters> Sub<&'a Self> for GroupProjective<P> {
    type Output = Self;

    fn sub(self, other: &'a Self) -> Self {
        let mut copy = self;
        copy -= other;
        copy
    }
}

impl<'a, P: Parameters> SubAssign<&'a Self> for GroupProjective<P> {
    fn sub_assign(&mut self, other: &'a Self) {
        *self += &(-(*other));
    }
}

impl<P: Parameters> Mul<P::ScalarField> for GroupProjective<P> {
    type Output = Self;

    /// Performs scalar multiplication of this element.
    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline]
    fn mul(self, other: P::ScalarField) -> Self {
        let mut res = Self::zero();

        let mut found_one = false;

        for i in BitIteratorBE::new(other.into_repr()) {
            if found_one {
                res.double_in_place();
            } else {
                found_one = i;
            }

            if i {
                res += self;
            }
        }

        res
    }
}

impl<P: Parameters> MulAssign<P::ScalarField> for GroupProjective<P> {
    /// Performs scalar multiplication of this element.
    fn mul_assign(&mut self, other: P::ScalarField) {
        *self = *self * other
    }
}

// The affine point (X, Y) is represented in the Extended Projective coordinates
// with Z = 1.
impl<P: Parameters> From<GroupAffine<P>> for GroupProjective<P> {
    fn from(p: GroupAffine<P>) -> GroupProjective<P> {
        Self::new(p.x, p.y, p.x * p.y, P::BaseField::one())
    }
}

// The projective point X, Y, T, Z is represented in the affine
// coordinates as X/Z, Y/Z.
impl<P: Parameters> From<GroupProjective<P>> for GroupAffine<P> {
    fn from(p: GroupProjective<P>) -> GroupAffine<P> {
        if p.is_zero() {
            GroupAffine::zero()
        } else if p.z.is_one() {
            // If Z is one, the point is already normalized.
            GroupAffine::new(p.x, p.y)
        } else {
            // Z is nonzero, so it must have an inverse in a field.
            let z_inv = p.z.inverse().unwrap();
            let x = p.x * z_inv;
            let y = p.y * z_inv;
            GroupAffine::new(x, y)
        }
    }
}

#[derive(Derivative)]
#[derivative(
    Copy(bound = "P: MontgomeryParameters"),
    Clone(bound = "P: MontgomeryParameters"),
    PartialEq(bound = "P: MontgomeryParameters"),
    Eq(bound = "P: MontgomeryParameters"),
    Debug(bound = "P: MontgomeryParameters"),
    Hash(bound = "P: MontgomeryParameters")
)]
pub struct MontgomeryGroupAffine<P: MontgomeryParameters> {
    pub x: P::BaseField,
    pub y: P::BaseField,
    #[derivative(Debug = "ignore")]
    _params: PhantomData<P>,
}

impl<P: MontgomeryParameters> Display for MontgomeryGroupAffine<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "MontgomeryGroupAffine(x={}, y={})", self.x, self.y)
    }
}

impl<P: MontgomeryParameters> MontgomeryGroupAffine<P> {
    pub fn new(x: P::BaseField, y: P::BaseField) -> Self {
        Self {
            x,
            y,
            _params: PhantomData,
        }
    }
}

impl_edwards_curve_serializer!(Parameters);
