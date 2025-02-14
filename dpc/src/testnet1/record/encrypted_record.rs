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

use crate::{testnet1::Testnet1Components, traits::DPCComponents};
use snarkvm_algorithms::traits::EncryptionScheme;
use snarkvm_curves::traits::{AffineCurve, ProjectiveCurve};
use snarkvm_utilities::{bits_to_bytes, bytes_to_bits, to_bytes, variable_length_integer::*, FromBytes, ToBytes};

use itertools::Itertools;
use std::io::{Error, ErrorKind, Read, Result as IoResult, Write};

#[derive(Derivative)]
#[derivative(
    Clone(bound = "C: Testnet1Components"),
    PartialEq(bound = "C: Testnet1Components"),
    Eq(bound = "C: Testnet1Components"),
    Debug(bound = "C: Testnet1Components")
)]
pub struct EncryptedRecord<C: Testnet1Components> {
    pub encrypted_record: Vec<<<C as DPCComponents>::AccountEncryption as EncryptionScheme>::Text>,
    pub final_fq_high_selector: bool,
}

impl<C: Testnet1Components> ToBytes for EncryptedRecord<C> {
    #[inline]
    fn write<W: Write>(&self, mut writer: W) -> IoResult<()> {
        let mut ciphertext_selectors = Vec::with_capacity(self.encrypted_record.len() + 1);

        // Write the encrypted record
        variable_length_integer(self.encrypted_record.len() as u64).write(&mut writer)?;
        for ciphertext_element in &self.encrypted_record {
            // Compress the ciphertext representation to the affine x-coordinate and the selector bit
            let ciphertext_element_affine =
                <C as Testnet1Components>::EncryptionGroup::read(&to_bytes![ciphertext_element]?[..])?.into_affine();

            let x_coordinate = ciphertext_element_affine.to_x_coordinate();
            x_coordinate.write(&mut writer)?;

            let selector =
                match <<C as Testnet1Components>::EncryptionGroup as ProjectiveCurve>::Affine::from_x_coordinate(
                    x_coordinate,
                    true,
                ) {
                    Some(affine) => ciphertext_element_affine == affine,
                    None => false,
                };

            ciphertext_selectors.push(selector);
        }

        ciphertext_selectors.push(self.final_fq_high_selector);

        // Write the ciphertext and fq_high selector bits
        let selector_bytes = bits_to_bytes(&ciphertext_selectors);
        selector_bytes.write(&mut writer)?;

        Ok(())
    }
}

impl<C: Testnet1Components> FromBytes for EncryptedRecord<C> {
    #[inline]
    fn read<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the ciphertext x coordinates
        let num_ciphertext_elements = read_variable_length_integer(&mut reader)?;
        let mut ciphertext_x_coordinates = Vec::with_capacity(num_ciphertext_elements);
        for _ in 0..num_ciphertext_elements {
            let ciphertext_element_x_coordinate: <<<C as Testnet1Components>::EncryptionGroup as ProjectiveCurve>::Affine as AffineCurve>::BaseField =
                FromBytes::read(&mut reader)?;
            ciphertext_x_coordinates.push(ciphertext_element_x_coordinate);
        }

        // Read the selector bits

        let num_selector_bytes = num_ciphertext_elements / 8 + 1;
        let mut selector_bytes = vec![0u8; num_selector_bytes];
        reader.read_exact(&mut selector_bytes)?;

        let mut selector_bits = bytes_to_bits(&selector_bytes);
        let ciphertext_selectors = selector_bits.by_ref().take(num_ciphertext_elements);

        // Recover the ciphertext
        let mut ciphertext = Vec::with_capacity(ciphertext_x_coordinates.len());
        for (x_coordinate, ciphertext_selector_bit) in ciphertext_x_coordinates.iter().zip_eq(ciphertext_selectors) {
            let ciphertext_element_affine =
                match <<C as Testnet1Components>::EncryptionGroup as ProjectiveCurve>::Affine::from_x_coordinate(
                    *x_coordinate,
                    ciphertext_selector_bit,
                ) {
                    Some(affine) => affine,
                    None => return Err(Error::new(ErrorKind::Other, "Could not read ciphertext")),
                };

            let ciphertext_element: <C::AccountEncryption as EncryptionScheme>::Text =
                FromBytes::read(&to_bytes![ciphertext_element_affine.into_projective()]?[..])?;

            ciphertext.push(ciphertext_element);
        }

        let final_fq_high_selector = selector_bits.next().unwrap();

        Ok(Self {
            encrypted_record: ciphertext,
            final_fq_high_selector,
        })
    }
}
