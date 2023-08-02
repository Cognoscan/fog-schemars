use crate::{SchemaGenerator, FogValidate, Name};
use fog_pack::validator::*;

macro_rules! tuple_impls {
    ($($len:expr => ($($name:ident)+))+) => {
        $(
            impl<$($name: FogValidate),+> FogValidate for ($($name,)+) {
                no_ref_validator!();

                fn validator_name(_: bool) -> Name {
                    Name::with_types("", concat!("Tuple", $len), vec![$($name::validator_name(false)),+])
                }

                fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
                    let prefix = vec![
                        $(gen.type_add::<$name>()),+
                    ];
                    let mut v = ArrayValidator::new().min_len($len).max_len($len);
                    for p in prefix {
                        v = v.prefix_add(p);
                    }
                    v.build()
                }

            }
        )+
    }
}

tuple_impls! {
    1 => (T0)
    2 => (T0 T1)
    3 => (T0 T1 T2)
    4 => (T0 T1 T2 T3)
    5 => (T0 T1 T2 T3 T4)
    6 => (T0 T1 T2 T3 T4 T5)
    7 => (T0 T1 T2 T3 T4 T5 T6)
    8 => (T0 T1 T2 T3 T4 T5 T6 T7)
    9 => (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    10 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    11 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    12 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    13 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    14 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    15 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    16 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}
