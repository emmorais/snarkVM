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

use snarkvm_dpc::{
    errors::DPCError,
    testnet2::{instantiated::Components, Testnet2Components, DPC},
};
use snarkvm_fields::ToConstraintField;
use snarkvm_marlin::PolynomialCommitment;
use snarkvm_utilities::{bytes::ToBytes, to_bytes};

use rand::thread_rng;
use std::path::PathBuf;

mod utils;
use utils::store;

pub fn setup<C: Testnet2Components>() -> Result<Vec<u8>, DPCError>
where
    <C::PolynomialCommitment as PolynomialCommitment<C::InnerField>>::VerifierKey: ToConstraintField<C::OuterField>,
    <C::PolynomialCommitment as PolynomialCommitment<C::InnerField>>::Commitment: ToConstraintField<C::OuterField>,
{
    let rng = &mut thread_rng();

    let universal_srs = DPC::<C>::generate_program_snark_universal_srs(rng)?;
    let universal_srs_bytes = to_bytes![universal_srs.0]?;

    println!("universal_srs.params\n\tsize - {}", universal_srs_bytes.len());
    Ok(universal_srs_bytes)
}

pub fn main() {
    let universal_srs = setup::<Components>().unwrap();
    store(
        &PathBuf::from("universal_srs.params"),
        &PathBuf::from("universal_srs.checksum"),
        &universal_srs,
    )
    .unwrap();
}
