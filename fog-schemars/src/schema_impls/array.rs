use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

impl<T> FogValidate for [T; 0] {
    no_ref_validator!();

    fn validator_name(_: bool) -> Name {
        Name::new("", "Array0")
    }

    fn validator(_: &mut SchemaGenerator, _: bool) -> Validator {
        ArrayValidator::new().max_len(0).build()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: FogValidate> FogValidate for [T; $len] {
                fn validator_name(_: bool) -> Name {
                    let name = concat!("Array", $len);
                    Name::with_types("", &name, vec![T::validator_name(false)])
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
