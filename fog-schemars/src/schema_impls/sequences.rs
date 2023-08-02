use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where T: FogValidate,
        {
            no_ref_validator!();
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                let name = if opt { "NonEmptyVec" } else { "Vec" };
                Name::with_types("", name, vec![T::validator_name(false)])
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                let ty = gen.type_add::<T>();
                if opt {
                    ArrayValidator::new().items(ty).min_len(1).build()
                }
                else {
                    ArrayValidator::new().items(ty).build()
                }
            }
        }
    };
}

macro_rules! set_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where T: FogValidate,
        {
            no_ref_validator!();
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                let name = if opt { "NonEmptySet" } else { "Set" };
                Name::with_types("", name, vec![T::validator_name(false)])
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                let ty = gen.type_add::<T>();
                if opt {
                    ArrayValidator::new().unique(true).items(ty).min_len(1).build()
                }
                else {
                    ArrayValidator::new().unique(true).items(ty).build()
                }
            }
        }
    };
}

seq_impl!(<T> FogValidate for std::collections::BinaryHeap<T>);
seq_impl!(<T> FogValidate for std::collections::LinkedList<T>);
seq_impl!(<T> FogValidate for std::collections::VecDeque<T>);
seq_impl!(<T> FogValidate for Vec<T>);
seq_impl!(<T> FogValidate for [T]);

set_impl!(<T> FogValidate for std::collections::BTreeSet<T>);
set_impl!(<T, H> FogValidate for std::collections::HashSet<T, H>);