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

use snarkvm_algorithms::crh::sha256;
use snarkvm_dpc::errors::DPCError;
use snarkvm_posw::PoswMarlin;
use snarkvm_utilities::{bytes::ToBytes, to_bytes};

use rand::thread_rng;
use std::path::PathBuf;

mod utils;
use utils::store;

#[allow(clippy::type_complexity)]
pub fn setup() -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), DPCError> {
    let rng = &mut thread_rng();

    let srs = snarkvm_marlin::MarlinTestnet1::universal_setup(10000, 10000, 100000, rng).unwrap();

    let srs_bytes = to_bytes![srs]?;
    let posw_snark = PoswMarlin::index(srs).expect("could not setup params");

    let posw_snark_pk = to_bytes![posw_snark.pk.expect("posw_snark_pk should be populated")]?;
    let posw_snark_vk = posw_snark.vk;
    let posw_snark_vk = to_bytes![posw_snark_vk]?;

    println!("posw_snark_pk.params\n\tsize - {}", posw_snark_pk.len());
    println!("posw_snark_vk.params\n\tsize - {}", posw_snark_vk.len());
    println!("srs\n\tsize - {}", srs_bytes.len());
    Ok((posw_snark_pk, posw_snark_vk, srs_bytes))
}

fn versioned_filename(checksum: &str) -> String {
    match checksum.get(0..7) {
        Some(sum) => format!("posw_snark_pk-{}.params", sum),
        _ => "posw_snark_pk.params".to_string(),
    }
}

pub fn main() {
    let (posw_snark_pk, posw_snark_vk, _srs) = setup().unwrap();
    let posw_snark_pk_checksum = hex::encode(sha256(&posw_snark_pk));
    store(
        &PathBuf::from(&versioned_filename(&posw_snark_pk_checksum)),
        &PathBuf::from("posw_snark_pk.checksum"),
        &posw_snark_pk,
    )
    .unwrap();
    store(
        &PathBuf::from("posw_snark_vk.params"),
        &PathBuf::from("posw_snark_vk.checksum"),
        &posw_snark_vk,
    )
    .unwrap();
}
