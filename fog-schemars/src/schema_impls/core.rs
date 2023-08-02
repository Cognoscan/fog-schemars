use std::ops::Bound;

use crate::{SchemaGenerator, FogValidate, Name};
use fog_pack::types::Value;
use fog_pack::validator::*;

impl FogValidate for () {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "!" } else { "unit" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            // A boolean that will never succeed, because this value should
            // never be generated
            BoolValidator::new().nin_add(false).nin_add(true).build()
        } else {
            Validator::Null
        }
    }
}

forward_impl!((<T: ?Sized> FogValidate for std::marker::PhantomData<T>) => ());

impl<T: FogValidate> FogValidate for Option<T> {
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        if opt {
            T::validator_name(false)
        } else {
            Name::with_types("std::option", "Option", vec![T::validator_name(false)])
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
    fn validator_name(_: bool) -> Name {
        Name::with_types(
            "std::result",
            "Result",
            vec![T::validator_name(false), E::validator_name(false)],
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
    fn validator_name(_: bool) -> Name {
        Name::with_types("std::ops", "Bound", vec![T::validator_name(false)])
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
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        if opt {
            Name::with_types("std::ops", "OptRange", vec![Idx::validator_name(false)])
        } else {
            Name::with_types("std::ops", "Range", vec![Idx::validator_name(false)])
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

impl<Idx: FogValidate> FogValidate for std::ops::RangeInclusive<Idx>
where
    Idx: Default,
    Value: From<Idx>,
{
    fn validator_name(_: bool) -> Name {
        Name::with_types(
            "std::ops",
            "RangeInclusive",
            vec![Idx::validator_name(false)],
        )
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        MapValidator::new()
            .req_add("start", Idx::validator(gen, false))
            .req_add("end", Idx::validator(gen, false))
            .build()
    }
}

impl<Idx: FogValidate> FogValidate for std::ops::RangeFrom<Idx>
where
    Idx: Default,
    Value: From<Idx>,
{
    fn validator_name(_: bool) -> Name {
        Name::with_types("std::ops", "RangeFrom", vec![Idx::validator_name(false)])
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        MapValidator::new()
            .req_add("start", Idx::validator(gen, false))
            .build()
    }
}

impl<Idx: FogValidate> FogValidate for std::ops::RangeTo<Idx>
where
    Idx: Default,
    Value: From<Idx>,
{
    fn validator_name(_: bool) -> Name {
        Name::with_types("std::ops", "RangeTo", vec![Idx::validator_name(false)])
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        MapValidator::new()
            .req_add("end", Idx::validator(gen, false))
            .build()
    }
}
