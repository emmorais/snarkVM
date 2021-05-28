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

use crate::{traits::FiatShamirRng, Vec};
use snarkvm_fields::PrimeField;
use snarkvm_gadgets::{
    fields::FpGadget,
    utilities::{boolean::Boolean, uint::UInt8},
};
use snarkvm_nonnative::{params::OptimizationType, NonNativeFieldVar};
use snarkvm_r1cs::{ConstraintSystem, SynthesisError};

/// Constraints for a RNG for use in a Fiat-Shamir transform.
pub trait FiatShamirRngVar<TargetField: PrimeField, BaseField: PrimeField, PFS: FiatShamirRng<TargetField, BaseField>>:
    Clone
{
    /// Create a new RNG.
    fn new<CS: ConstraintSystem<BaseField>>(cs: CS) -> Self;

    /// Instantiate from a plaintext fs_rng.
    fn constant<CS: ConstraintSystem<BaseField>>(cs: CS, pfs: &PFS) -> Self;

    /// Take in field elements.
    fn absorb_nonnative_field_elements<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        elems: &[NonNativeFieldVar<TargetField, BaseField>],
        ty: OptimizationType,
    ) -> Result<(), SynthesisError>;

    /// Take in field elements.
    fn absorb_native_field_elements<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        elems: &[FpGadget<BaseField>],
    ) -> Result<(), SynthesisError>;

    /// Take in bytes.
    fn absorb_bytes<CS: ConstraintSystem<BaseField>>(&mut self, cs: CS, elems: &[UInt8]) -> Result<(), SynthesisError>;

    /// Output field elements.
    fn squeeze_native_field_elements<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<Vec<FpGadget<BaseField>>, SynthesisError>;

    /// Output field elements.
    fn squeeze_field_elements<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<Vec<NonNativeFieldVar<TargetField, BaseField>>, SynthesisError>;

    /// Output field elements and the corresponding bits (this can reduce repeated computation).
    #[allow(clippy::type_complexity)]
    fn squeeze_field_elements_and_bits<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<(Vec<NonNativeFieldVar<TargetField, BaseField>>, Vec<Vec<Boolean>>), SynthesisError>;

    /// Output field elements with only 128 bits.
    fn squeeze_128_bits_field_elements<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<Vec<NonNativeFieldVar<TargetField, BaseField>>, SynthesisError>;

    /// Output field elements with only 128 bits, and the corresponding bits (this can reduce
    /// repeated computation).
    #[allow(clippy::type_complexity)]
    fn squeeze_128_bits_field_elements_and_bits<CS: ConstraintSystem<BaseField>>(
        &mut self,
        cs: CS,
        num: usize,
    ) -> Result<(Vec<NonNativeFieldVar<TargetField, BaseField>>, Vec<Vec<Boolean>>), SynthesisError>;
}
