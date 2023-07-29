use crate::SchemaGenerator;
use fog_pack::validator::*;

use crate::FogValidate;

macro_rules! integer_impls {
    ($($type:ty)+) => {
        $(
            impl FogValidate for $type {
                no_ref_validator!();

                fn validator_name(_: bool) -> String {
                    (stringify!($type)).into()
                }

                fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
                    IntValidator::new().min(<$type>::MIN).max(<$type>::MAX).build()
                }
            }
        )+
    }
}

// Purposely excluding usize & isize, as their valid ranges can change depending
// on the architecture this code is run on.
integer_impls! { u8 u16 u32 u64 i8 i16 i32 i64 }

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: FogValidate> FogValidate for [T; $len] {
                fn validator_name(_: bool) -> String {
                    format!("Array{}_{}", $len, T::validator_name(false))
                }

                fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
                    ArrayValidator::new()
                        .min_len($len)
                        .max_len($len)
                        .items(gen.type_add::<T>())
                        .build()
                }
            }
        )+
    }
}

array_impls! {
     1  2  3  4  5  6  7  8  9 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}
