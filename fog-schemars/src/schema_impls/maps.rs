use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: FogValidate,
        {
            no_ref_validator!();
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                let name = if opt { "NonEmptyMap" } else { "Map" };
                Name::with_types("", name, vec![V::validator_name(false)])
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                let ty = gen.type_add::<V>();
                if opt {
                    MapValidator::new().nin_add([]).values(ty).build()
                }
                else {
                    MapValidator::new().values(ty).build()
                }
            }
        }
    };
}

map_impl!(<V> FogValidate for std::collections::BTreeMap<String, V>);
map_impl!(<V, H> FogValidate for std::collections::HashMap<String, V, H>);
