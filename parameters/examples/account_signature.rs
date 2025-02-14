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

use snarkvm_algorithms::{errors::SignatureError, traits::SignatureScheme};
use snarkvm_dpc::{testnet1::instantiated::Components, traits::DPCComponents};
use snarkvm_utilities::{bytes::ToBytes, to_bytes};

use rand::thread_rng;
use std::path::PathBuf;

mod utils;
use utils::store;

pub fn setup<C: DPCComponents>() -> Result<Vec<u8>, SignatureError> {
    let rng = &mut thread_rng();
    let account_signature = <C::AccountSignature as SignatureScheme>::setup(rng)?;
    let account_signature_parameters = account_signature.parameters();
    let account_signature_parameters_bytes = to_bytes![account_signature_parameters]?;

    let size = account_signature_parameters_bytes.len();
    println!("account_signature.params\n\tsize - {}", size);
    Ok(account_signature_parameters_bytes)
}

pub fn main() {
    let bytes = setup::<Components>().unwrap();
    let filename = PathBuf::from("account_signature.params");
    let sumname = PathBuf::from("account_signature.checksum");
    store(&filename, &sumname, &bytes).unwrap();
}
