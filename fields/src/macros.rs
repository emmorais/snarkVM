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

#[macro_export]
macro_rules! field {
    ($name:ident, $c0:expr) => {
        $name {
            0: $c0,
            1: std::marker::PhantomData,
        }
    };
    ($name:ident, $c0:expr, $c1:expr $(,)?) => {
        $name {
            c0: $c0,
            c1: $c1,
            _parameters: std::marker::PhantomData,
        }
    };
    ($name:ident, $c0:expr, $c1:expr, $c2:expr $(,)?) => {
        $name {
            c0: $c0,
            c1: $c1,
            c2: $c2,
            _parameters: std::marker::PhantomData,
        }
    };
}

macro_rules! impl_field_into_bigint {
    ($field: ident, $bigint: ident, $params: ident) => {
        impl<P: $params> Into<$bigint> for $field<P> {
            fn into(self) -> $bigint {
                self.into_repr()
            }
        }
    };
}

macro_rules! impl_prime_field_standard_sample {
    ($field: ident, $params: ident) => {
        impl<P: $params> rand::distributions::Distribution<$field<P>> for rand::distributions::Standard {
            #[inline]
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $field<P> {
                loop {
                    let mut tmp = $field(rng.sample(rand::distributions::Standard), PhantomData);
                    // Mask away the unused bits at the beginning.
                    tmp.0
                        .as_mut()
                        .last_mut()
                        .map(|val| *val &= std::u64::MAX >> P::REPR_SHAVE_BITS);

                    if tmp.is_valid() {
                        return tmp;
                    }
                }
            }
        }
    };
}

macro_rules! impl_prime_field_from_int {
    ($field: ident, u128, $params: ident) => {
        impl<P: $params> From<u128> for $field<P> {
            /// Attempts to convert an integer into a field element.
            /// Panics if the provided integer is invalid (e.g. larger than the field modulus).
            fn from(other: u128) -> Self {
                let upper = (other >> 64) as u64;
                let lower = ((other << 64) >> 64) as u64;
                let mut default_int = P::BigInteger::default();
                default_int.0[0] = lower;
                default_int.0[1] = upper;
                Self::from_repr(default_int).unwrap()
            }
        }
    };
    ($field: ident, $int: ident, $params: ident) => {
        impl<P: $params> From<$int> for $field<P> {
            /// Attempts to convert an integer into a field element.
            /// Panics if the provided integer is invalid (e.g. larger than the field modulus).
            fn from(other: $int) -> Self {
                Self::from_repr(P::BigInteger::from(u64::from(other))).unwrap()
            }
        }
    };
}

macro_rules! sqrt_impl {
    ($Self:ident, $P:tt, $self:expr) => {{
        use crate::LegendreSymbol::*;
        // https://eprint.iacr.org/2012/685.pdf (page 12, algorithm 5)
        // Actually this is just normal Tonelli-Shanks; since `P::Generator`
        // is a quadratic non-residue, `P::ROOT_OF_UNITY = P::GENERATOR ^ t`
        // is also a quadratic non-residue (since `t` is odd).
        match $self.legendre() {
            Zero => Some(*$self),
            QuadraticNonResidue => None,
            QuadraticResidue => {
                let mut z = $Self::two_adic_root_of_unity();
                let mut w = $self.pow($P::T_MINUS_ONE_DIV_TWO);
                let mut x = w * $self;
                let mut b = x * w;

                let mut v = $P::TWO_ADICITY as usize;
                // t = self^t
                #[cfg(debug_assertions)]
                {
                    let mut check = b;
                    for _ in 0..(v - 1) {
                        check.square_in_place();
                    }
                    if !check.is_one() {
                        panic!("Input is not a square root, but it passed the QR test")
                    }
                }

                while !b.is_one() {
                    let mut k = 0usize;

                    let mut b2k = b;
                    while !b2k.is_one() {
                        // invariant: b2k = b^(2^k) after entering this loop
                        b2k.square_in_place();
                        k += 1;
                    }

                    let j = v - k - 1;
                    w = z;
                    for _ in 0..j {
                        w.square_in_place();
                    }

                    z = w.square();
                    b *= &z;
                    x *= &w;
                    v = k;
                }

                Some(x)
            }
        }
    }};
}

macro_rules! impl_prime_field_serializer {
    ($field: ident, $params: ident, $byte_size: expr) => {
        impl<P: $params> CanonicalSerializeWithFlags for $field<P> {
            #[allow(unused_qualifications)]
            fn serialize_with_flags<W: snarkvm_utilities::io::Write, F: snarkvm_utilities::serialize::Flags>(
                &self,
                writer: &mut W,
                flags: F,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                const BYTE_SIZE: usize = $byte_size;

                let (output_bit_size, output_byte_size) =
                    snarkvm_utilities::serialize::buffer_bit_byte_size($field::<P>::size_in_bits());
                if F::len() > (output_bit_size - P::MODULUS_BITS as usize) {
                    return Err(snarkvm_utilities::errors::SerializationError::NotEnoughSpace);
                }

                let mut bytes = [0u8; BYTE_SIZE];
                self.write(&mut bytes[..])?;

                bytes[output_byte_size - 1] |= flags.u8_bitmask();

                writer.write_all(&bytes[..output_byte_size])?;
                Ok(())
            }
        }

        impl<P: $params> ConstantSerializedSize for $field<P> {
            const SERIALIZED_SIZE: usize = snarkvm_utilities::serialize::buffer_byte_size(
                <$field<P> as crate::PrimeField>::Parameters::MODULUS_BITS as usize,
            );
            const UNCOMPRESSED_SIZE: usize = Self::SERIALIZED_SIZE;
        }

        impl<P: $params> CanonicalSerialize for $field<P> {
            #[allow(unused_qualifications)]
            #[inline]
            fn serialize<W: snarkvm_utilities::io::Write>(
                &self,
                writer: &mut W,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                self.serialize_with_flags(writer, snarkvm_utilities::serialize::EmptyFlags)
            }

            #[inline]
            fn serialized_size(&self) -> usize {
                Self::SERIALIZED_SIZE
            }
        }

        impl<P: $params> CanonicalDeserializeWithFlags for $field<P> {
            #[allow(unused_qualifications)]
            fn deserialize_with_flags<R: snarkvm_utilities::io::Read, F: snarkvm_utilities::serialize::Flags>(
                reader: &mut R,
            ) -> Result<(Self, F), snarkvm_utilities::errors::SerializationError> {
                const BYTE_SIZE: usize = $byte_size;

                let (output_bit_size, output_byte_size) =
                    snarkvm_utilities::serialize::buffer_bit_byte_size($field::<P>::size_in_bits());
                if F::len() > (output_bit_size - P::MODULUS_BITS as usize) {
                    return Err(snarkvm_utilities::errors::SerializationError::NotEnoughSpace);
                }

                let mut masked_bytes = [0; BYTE_SIZE];
                reader.read_exact(&mut masked_bytes[..output_byte_size])?;

                let flags = F::from_u8_remove_flags(&mut masked_bytes[output_byte_size - 1]);

                Ok((Self::read(&masked_bytes[..])?, flags))
            }
        }

        impl<P: $params> CanonicalDeserialize for $field<P> {
            #[allow(unused_qualifications)]
            fn deserialize<R: snarkvm_utilities::io::Read>(
                reader: &mut R,
            ) -> Result<Self, snarkvm_utilities::errors::SerializationError> {
                const BYTE_SIZE: usize = $byte_size;

                let (_, output_byte_size) =
                    snarkvm_utilities::serialize::buffer_bit_byte_size($field::<P>::size_in_bits());

                let mut masked_bytes = [0; BYTE_SIZE];
                reader.read_exact(&mut masked_bytes[..output_byte_size])?;
                Ok(Self::read(&masked_bytes[..])?)
            }
        }

        impl<P: $params> serde::Serialize for $field<P> {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                use serde::ser::SerializeTuple;

                let len = self.serialized_size();
                let mut bytes = Vec::with_capacity(len);
                CanonicalSerialize::serialize(self, &mut bytes).map_err(serde::ser::Error::custom)?;

                let mut tup = s.serialize_tuple(len)?;
                for byte in &bytes {
                    tup.serialize_element(byte)?;
                }
                tup.end()
            }
        }

        impl<'de, P: $params> serde::Deserialize<'de> for $field<P> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct SerVisitor<P>(std::marker::PhantomData<P>);

                impl<'de, P: $params> serde::de::Visitor<'de> for SerVisitor<P> {
                    type Value = $field<P>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a valid field element")
                    }

                    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                    where
                        S: serde::de::SeqAccess<'de>,
                    {
                        let len = <Self::Value as ConstantSerializedSize>::SERIALIZED_SIZE;
                        let bytes: Vec<u8> = (0..len)
                            .map(|_| {
                                seq.next_element()?
                                    .ok_or_else(|| serde::de::Error::custom("could not read bytes"))
                            })
                            .collect::<Result<Vec<_>, _>>()?;

                        let res =
                            CanonicalDeserialize::deserialize(&mut &bytes[..]).map_err(serde::de::Error::custom)?;
                        Ok(res)
                    }
                }

                let visitor = SerVisitor(std::marker::PhantomData);
                deserializer.deserialize_tuple(Self::SERIALIZED_SIZE, visitor)
            }
        }
    };
}

macro_rules! impl_field_from_random_bytes_with_flags {
    ($limbs: expr) => {
        #[inline]
        fn from_random_bytes_with_flags(bytes: &[u8]) -> Option<(Self, u8)> {
            let mut result_bytes = [0u8; $limbs * 8];
            for (result_byte, in_byte) in result_bytes.iter_mut().zip(bytes.iter()) {
                *result_byte = *in_byte;
            }

            let mask: u64 = 0xffffffffffffffff >> P::REPR_SHAVE_BITS;
            // the flags will be at the same byte with the lowest shaven bits or the one after
            let flags_byte_position: usize = 7 - P::REPR_SHAVE_BITS as usize / 8;
            let flags_mask: u8 = ((1 << P::REPR_SHAVE_BITS % 8) - 1) << (8 - P::REPR_SHAVE_BITS % 8);
            // take the last 8 bytes and pass the mask
            let last_bytes = &mut result_bytes[($limbs - 1) * 8..];
            let mut flags: u8 = 0;
            for (i, (b, m)) in last_bytes.iter_mut().zip(&mask.to_le_bytes()).enumerate() {
                if i == flags_byte_position {
                    flags = *b & flags_mask
                }
                *b &= m;
            }

            <Self as CanonicalDeserialize>::deserialize(&mut &result_bytes[..])
                .ok()
                .map(|f| (f, flags))
        }
    };
}

// Implements AddAssign on Self by deferring to an implementation on &Self
#[macro_export]
macro_rules! impl_additive_ops_from_ref {
    ($type: ident, $params: ident) => {
        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Add<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                let mut result = self;
                result.add_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Sub<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                let mut result = self;
                result.sub_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Add<&&Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn add(self, other: &&Self) -> Self {
                let mut result = self;
                result.add_assign(*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Sub<&&Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn sub(self, other: &&Self) -> Self {
                let mut result = self;
                result.sub_assign(*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Add<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn add(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.add_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Sub<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn sub(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.sub_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::AddAssign<Self> for $type<P> {
            fn add_assign(&mut self, other: Self) {
                self.add_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::SubAssign<Self> for $type<P> {
            fn sub_assign(&mut self, other: Self) {
                self.sub_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::AddAssign<&&Self> for $type<P> {
            fn add_assign(&mut self, other: &&Self) {
                self.add_assign(*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::SubAssign<&&Self> for $type<P> {
            fn sub_assign(&mut self, other: &&Self) {
                self.sub_assign(*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::AddAssign<&'a mut Self> for $type<P> {
            fn add_assign(&mut self, other: &'a mut Self) {
                self.add_assign(&*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::SubAssign<&'a mut Self> for $type<P> {
            fn sub_assign(&mut self, other: &'a mut Self) {
                self.sub_assign(&*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::iter::Sum<Self> for $type<P> {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::iter::Sum<&'a Self> for $type<P> {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }
    };
}

// Implements AddAssign on Self by deferring to an implementation on &Self
#[macro_export]
macro_rules! impl_multiplicative_ops_from_ref {
    ($type: ident, $params: ident) => {
        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Mul<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn mul(self, other: Self) -> Self {
                let mut result = self;
                result.mul_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Div<Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn div(self, other: Self) -> Self {
                let mut result = self;
                result.div_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Mul<&&Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn mul(self, other: &&Self) -> Self {
                let mut result = self;
                result.mul_assign(*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::Div<&&Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn div(self, other: &&Self) -> Self {
                let mut result = self;
                result.div_assign(*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Mul<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn mul(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.mul_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::Div<&'a mut Self> for $type<P> {
            type Output = Self;

            #[inline]
            fn div(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.div_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::MulAssign<Self> for $type<P> {
            fn mul_assign(&mut self, other: Self) {
                self.mul_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::DivAssign<Self> for $type<P> {
            fn div_assign(&mut self, other: Self) {
                self.div_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::MulAssign<&&Self> for $type<P> {
            fn mul_assign(&mut self, other: &&Self) {
                self.mul_assign(*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::ops::DivAssign<&&Self> for $type<P> {
            fn div_assign(&mut self, other: &&Self) {
                self.div_assign(*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::MulAssign<&'a mut Self> for $type<P> {
            fn mul_assign(&mut self, other: &'a mut Self) {
                self.mul_assign(&*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::ops::DivAssign<&'a mut Self> for $type<P> {
            fn div_assign(&mut self, other: &'a mut Self) {
                self.div_assign(&*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<P: $params> core::iter::Product<Self> for $type<P> {
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::one(), core::ops::Mul::mul)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, P: $params> core::iter::Product<&'a Self> for $type<P> {
            fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::one(), Mul::mul)
            }
        }
    };
}
