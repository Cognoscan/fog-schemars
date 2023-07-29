macro_rules! no_ref_validator {
    () => {
        fn should_reference(_: bool) -> bool {
            false
        }
    };
}

macro_rules! forward_impl {
    (($($impl:tt)+) => $target:ty) => {
        impl $($impl)+ {
            fn should_reference(opt: bool) -> bool {
                <$target>::should_reference(opt)
            }

            fn has_opt() -> bool {
                <$target>::has_default_opt()
            }

            fn validator_name(opt: bool) -> String {
                <$target>::validator_name(opt)
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                <$target>::validator(gen)
            }

            fn validator_type_id() -> std::any::TypeId {
                <$target>::validator_type_id()
            }
        }
    };
    ($ty:ty => $target:ty) => {
        forward_impl!((FogValidate for $ty) => $target);
    };
}

mod core;
mod primitives;