use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;
use std::num::*;

macro_rules! nonzero_signed_impl {
    ($non_zero:ty, $type:ty) => {
        impl FogValidate for $non_zero {
            no_ref_validator!();

            fn validator_name(_: bool) -> Name {
                Name::new("", stringify!($non_zero))
            }

            fn validator(_: &mut SchemaGenerator, _: bool) -> Validator {
                IntValidator::new()
                    .nin_add(0)
                    .min(<$type>::MIN)
                    .max(<$type>::MAX)
                    .build()
            }
        }
    };
}

macro_rules! nonzero_unsigned_impl {
    ($non_zero:ty, $type:ty) => {
        impl FogValidate for $non_zero {
            no_ref_validator!();

            fn validator_name(_: bool) -> Name {
                Name::new("", stringify!($non_zero))
            }

            fn validator(_: &mut SchemaGenerator, _: bool) -> Validator {
                IntValidator::new()
                    .min(<$type>::MIN + 1)
                    .max(<$type>::MAX)
                    .build()
            }
        }
    };
}

// Purposely excluding usize & isize, as their valid ranges can change depending
// on the architecture this code is run on.
nonzero_unsigned_impl!(NonZeroU8, u8);
nonzero_unsigned_impl!(NonZeroU16, u16);
nonzero_unsigned_impl!(NonZeroU32, u32);
nonzero_unsigned_impl!(NonZeroU64, u64);
nonzero_signed_impl!(NonZeroI8, i8);
nonzero_signed_impl!(NonZeroI16, i16);
nonzero_signed_impl!(NonZeroI32, i32);
nonzero_signed_impl!(NonZeroI64, i64);
