use std::any::TypeId;
use std::ops::Bound;

use crate::gen::SchemaGenerator;
use crate::FogValidate;
use fog_pack::types::Value;
use fog_pack::validator::*;

impl<T: FogValidate> FogValidate for Option<T> {
    fn has_opt() -> bool {
        true
    }

    fn validator_name(opt: bool) -> String {
        if opt {
            T::validator_name(false)
        } else {
            format!("Option_{}", T::validator_name(false))
        }
    }

    fn validator_type_id(opt: bool) -> TypeId {
        if opt {
            T::validator_type_id(false)
        } else {
            TypeId::of::<Self>()
        }
    }

    fn should_reference(opt: bool) -> bool {
        if opt {
            T::should_reference(false)
        } else {
            true
        }
    }

    fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            gen.type_add::<T>()
        } else {
            let v = gen.type_add::<T>();
            if let Validator::Null = v {
                panic!(
                    "Started generating an Option<()>, which cannot roundtrip \
                    correctly in fog-pack. Consider making a boolean instead. \
                    This happened in the type {}",
                    std::any::type_name::<T>()
                );
            }
            MultiValidator::new().push(Validator::Null).push(v).build()
        }
    }
}

impl<T: FogValidate, E: FogValidate> FogValidate for Result<T, E> {
    fn validator_name(_: bool) -> String {
        format!(
            "Result_{}_{}",
            T::validator_name(false),
            E::validator_name(false)
        )
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        EnumValidator::new()
            .insert("Ok", Some(gen.type_add::<T>()))
            .insert("Err", Some(gen.type_add::<E>()))
            .build()
    }
}

impl<T: FogValidate> FogValidate for Bound<T> {
    fn validator_name(_: bool) -> String {
        format!("Bound_{}", T::validator_name(false))
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        EnumValidator::new()
            .insert("Included", Some(gen.type_add::<T>()))
            .insert("Excluded", Some(gen.type_add::<T>()))
            .insert("Unbounded", None)
            .build()
    }
}

impl<Idx: FogValidate> FogValidate for std::ops::Range<Idx>
where
    Idx: Default,
    Value: From<Idx>,
{
    fn has_opt() -> bool {
        true
    }

    fn validator_name(opt: bool) -> String {
        if opt {
            format!("Opt_Range_{}", Idx::validator_name(false))
        } else {
            format!("Range_{}", Idx::validator_name(false))
        }
    }

    fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            let default = Value::from(Idx::default());
            MapValidator::new()
                .req_add("start", Idx::validator(gen, false))
                .req_add("end", Idx::validator(gen, false))
                .nin_add([
                    (String::from("start"), default.clone()),
                    (String::from("end"), default),
                ])
                .build()
        } else {
            MapValidator::new()
                .req_add("start", Idx::validator(gen, false))
                .req_add("end", Idx::validator(gen, false))
                .build()
        }
    }
}
