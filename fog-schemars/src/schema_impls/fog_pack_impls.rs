use std::collections::BTreeMap;

use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::types::*;
use fog_pack::validator::*;

struct ByteBuf(Vec<u8>);
impl FogValidate for ByteBuf {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new(module_path!(), if opt { "OptBytes" } else { "Bytes" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        let mut v = BinValidator::new();
        if opt { v = v.min_len(1); }
        v.build()
    }
}

impl FogValidate for Integer {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new(
            "fog_pack::types",
            if opt { "OptInteger" } else { "Integer" },
        )
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            IntValidator::new().nin_add(0).build()
        } else {
            IntValidator::new().build()
        }
    }
}

macro_rules! fogpack_simple {
    ($type:ty, $validator:ty) => {
        impl FogValidate for $type {
            no_ref_validator!();

            fn validator_name(_: bool) -> Name {
                Name::new("fog_pack::types", stringify!($type))
            }

            fn validator(_: &mut SchemaGenerator, _: bool) -> Validator {
                <$validator>::new().build()
            }
        }
    };
}

impl FogValidate for Value {
    no_ref_validator!();

    fn validator_name(_: bool) -> Name {
        Name::new("fog_pack::types", "Value")
    }

    fn validator(_: &mut SchemaGenerator, _: bool) -> Validator {
        Validator::Any
    }
}
forward_impl!((<'a> FogValidate for ValueRef<'a>) => Value);

fogpack_simple!(Hash, HashValidator);
fogpack_simple!(Identity, IdentityValidator);
fogpack_simple!(StreamId, StreamIdValidator);
fogpack_simple!(LockId, LockIdValidator);
fogpack_simple!(Timestamp, TimeValidator);
fogpack_simple!(DataLockbox, DataLockboxValidator);
fogpack_simple!(LockLockbox, LockLockboxValidator);
fogpack_simple!(IdentityLockbox, IdentityLockboxValidator);
fogpack_simple!(StreamLockbox, StreamLockboxValidator);
fogpack_simple!(DataLockboxRef, DataLockboxValidator);
fogpack_simple!(LockLockboxRef, LockLockboxValidator);
fogpack_simple!(IdentityLockboxRef, IdentityLockboxValidator);
fogpack_simple!(StreamLockboxRef, StreamLockboxValidator);

macro_rules! validator_impl {
    ($validator:ty => { $($key:literal : $val:ty),+ }) => {
        impl FogValidate for $validator {
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                Name::new(
                    "fog_pack::validator",
                    if opt {
                        concat!("Opt", stringify!($validator))
                    }
                    else {
                        stringify!($validator)
                    }
                )
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                let mut validator = MapValidator::new()
                    $(.opt_add($key, gen.type_add_opt::<$val>()))+;
                if opt {
                    validator = validator.nin_add([]);
                }
                validator.build()
            }

        }
    }
}

macro_rules! float_nan_default {
    ($name:ident => ($float:ty, $validator:ty)) => {
        /// Same as floating point, but defaults to NaN instead of 0.
        #[repr(transparent)]
        struct $name(pub $float);

        impl Default for $name {
            fn default() -> Self {
                Self(<$float>::NAN)
            }
        }

        impl From<$float> for $name {
            fn from(value: $float) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $float {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FogValidate for $name {
            no_ref_validator!();
            has_opt!();
            fn validator_name(opt: bool) -> Name {
                if opt {
                    Name::new(module_path!(), stringify!($name))
                } else {
                    <$float>::validator_name(false)
                }
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                if opt {
                    <$validator>::new()
                        .min(<$float>::NEG_INFINITY)
                        .max(<$float>::INFINITY)
                        .build()
                } else {
                    <$float>::validator(gen, false)
                }
            }
        }
    };
}

macro_rules! type_skip_default {
    ($name:ident, $type:ty, $v:ty, $skip:expr) => {
        #[repr(transparent)]
        struct $name(pub $type);

        impl Default for $name {
            fn default() -> Self {
                Self(<$type>::from($skip))
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $type {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FogValidate for $name {
            fn should_reference(opt: bool) -> bool {
                if opt {
                    false
                } else {
                    <$type>::should_reference(false)
                }
            }
            has_opt!();
            fn validator_name(opt: bool) -> Name {
                if opt {
                    Name::new(module_path!(), stringify!($name))
                } else {
                    <$type>::validator_name(false)
                }
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                if opt {
                    <$v>::new().nin_add($skip).build()
                } else {
                    <$type>::validator(gen, false)
                }
            }
        }
    };
}

float_nan_default!(F32NanDefault => (f32, F32Validator));
float_nan_default!(F64NanDefault => (f64, F64Validator));

type_skip_default!(IntMaxDefault, Integer, IntValidator, u64::MAX);
type_skip_default!(IntMinDefault, Integer, IntValidator, i64::MIN);
type_skip_default!(IntMaxU32Default, Integer, IntValidator, u32::MAX);
type_skip_default!(
    TimeMaxDefault,
    Timestamp,
    TimeValidator,
    Timestamp::max_value()
);
type_skip_default!(
    TimeMinDefault,
    Timestamp,
    TimeValidator,
    Timestamp::min_value()
);

validator_impl!(BoolValidator => {
    "comment": String,
    "in": Vec<bool>,
    "nin": Vec<bool>,
    "query": bool
});

validator_impl!(IntValidator => {
    "comment": String,
    "bits_clr": u64,
    "bits_set": u64,
    "max": IntMaxDefault,
    "min": IntMinDefault,
    "ex_max": bool,
    "ex_min": bool,
    "in": Vec<Integer>,
    "nin": Vec<Integer>,
    "query": bool,
    "bit": bool,
    "ord": bool
});

validator_impl!(F32Validator => {
    "comment": String,
    "max": F32NanDefault,
    "min": F32NanDefault,
    "ex_max": bool,
    "ex_min": bool,
    "in": Vec<f32>,
    "nin": Vec<f32>,
    "query": bool,
    "ord": bool
});

validator_impl!(F64Validator => {
    "comment": String,
    "max": F64NanDefault,
    "min": F64NanDefault,
    "ex_max": bool,
    "ex_min": bool,
    "in": Vec<f64>,
    "nin": Vec<f64>,
    "query": bool,
    "ord": bool
});

validator_impl!(BinValidator => {
    "comment": String,
    "bits_clr": ByteBuf,
    "bits_set": ByteBuf,
    "max": ByteBuf,
    "min": ByteBuf,
    "ex_max": bool,
    "ex_min": bool,
    "max_len": IntMaxU32Default,
    "min_len": u32,
    "in": Vec<ByteBuf>,
    "nin": Vec<ByteBuf>,
    "query": bool,
    "bit": bool,
    "ord": bool,
    "size": bool
});

impl FogValidate for Normalize {
    has_opt!();
    fn validator_name(opt: bool) -> Name {
        Name::new(
            "fog_pack::validator",
            if opt { "OptNormalize" } else { "Normalize" },
        )
    }
    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        let mut v = EnumValidator::new()
            .insert("NFC", None)
            .insert("NFKC", None);
        if !opt {
            v = v.insert("None", None);
        }
        v.build()
    }
}

validator_impl!(StrValidator => {
    "comment": String,
    "in": Vec<String>,
    "nin": Vec<String>,
    "matches": Option<String>,
    "max_len": IntMaxU32Default,
    "min_len": u32,
    "max_char": IntMaxU32Default,
    "min_char": u32,
    "normalize": Normalize,
    "ban_prefix": Vec<String>,
    "ban_suffix": Vec<String>,
    "ban_char": String,
    "query": bool,
    "regex": bool,
    "ban": bool,
    "size": bool
});

validator_impl!(ArrayValidator => {
    "comment": String,
    "contains": Vec<Validator>,
    "items": Option<Validator>,
    "prefix": Vec<Validator>,
    "max_len": IntMaxU32Default,
    "min_len": u32,
    "in": Vec<Vec<Value>>,
    "nin": Vec<Vec<Value>>,
    "unique": bool,
    "query": bool,
    "array": bool,
    "contains_ok": bool,
    "unique_ok": bool,
    "size": bool
});

validator_impl!(MapValidator => {
    "comment": String,
    "max_len": IntMaxU32Default,
    "min_len": u32,
    "keys": Option<StrValidator>,
    "values": Option<Validator>,
    "req": BTreeMap<String, Validator>,
    "opt": BTreeMap<String, Validator>,
    "in": Vec<BTreeMap<String, Value>>,
    "nin": Vec<BTreeMap<String, Value>>,
    "same_len": bool,
    "query": bool,
    "size": bool,
    "map_ok": bool,
    "same_len_ok": bool
});

validator_impl!(TimeValidator => {
    "comment": String,
    "max": TimeMaxDefault,
    "min": TimeMinDefault,
    "ex_max": bool,
    "ex_min": bool,
    "in": Vec<Timestamp>,
    "nin": Vec<Timestamp>,
    "query": bool,
    "ord": bool
});

validator_impl!(HashValidator => {
    "comment": String,
    "link": Option<Validator>,
    "schema": Vec<Option<Hash>>,
    "in": Vec<Hash>,
    "nin": Vec<Hash>,
    "query": bool,
    "link_ok": bool,
    "schema_ok": bool
});

macro_rules! id_validator {
    ($v:ty, $type:ty) => {
        validator_impl!($v => {
            "comment": String,
            "in": Vec<$type>,
            "nin": Vec<$type>,
            "query": bool
        });
    };
}

id_validator!(IdentityValidator, Identity);
id_validator!(StreamIdValidator, StreamId);
id_validator!(LockIdValidator, LockId);

macro_rules! lockbox_validators{
    ($($v:ty),+) => {
        $(
            validator_impl!($v => {
                "comment": String,
                "max_len": IntMaxU32Default,
                "min_len": u32,
                "size": bool
            });
        )+
    };
}

lockbox_validators!(
    DataLockboxValidator,
    IdentityLockboxValidator,
    LockLockboxValidator,
    StreamLockboxValidator
);

forward_impl!(EnumValidator => BTreeMap<String, Option<Validator>>);
forward_impl!(MultiValidator => Vec<Validator>);


impl FogValidate for Validator {
    fn validator_name(_: bool) -> Name {
        Name::new("fog_pack::validator", "Validator")
    }

    #[rustfmt::skip]
    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        EnumValidator::new()
            .insert("Null"           , None)
            .insert("Bool"           , Some(gen.type_add::<BoolValidator>()))
            .insert("Int"            , Some(gen.type_add::<IntValidator>()))
            .insert("F32"            , Some(gen.type_add::<F32Validator>()))
            .insert("F64"            , Some(gen.type_add::<F64Validator>()))
            .insert("Bin"            , Some(gen.type_add::<BinValidator>()))
            .insert("Str"            , Some(gen.type_add::<StrValidator>()))
            .insert("Array"          , Some(gen.type_add::<ArrayValidator>()))
            .insert("Map"            , Some(gen.type_add::<MapValidator>()))
            .insert("Time"           , Some(gen.type_add::<TimeValidator>()))
            .insert("Hash"           , Some(gen.type_add::<HashValidator>()))
            .insert("Identity"       , Some(gen.type_add::<IdentityValidator>()))
            .insert("StreamId"       , Some(gen.type_add::<StreamIdValidator>()))
            .insert("LockId"         , Some(gen.type_add::<LockIdValidator>()))
            .insert("DataLockbox"    , Some(gen.type_add::<DataLockboxValidator>()))
            .insert("IdentityLockbox", Some(gen.type_add::<IdentityLockboxValidator>()))
            .insert("StreamLockbox"  , Some(gen.type_add::<StreamLockboxValidator>()))
            .insert("LockLockbox"    , Some(gen.type_add::<LockLockboxValidator>()))
            .insert("Ref"            , Some(gen.type_add::<String>()))
            .insert("Multi"          , Some(gen.type_add::<MultiValidator>()))
            .insert("Enum"           , Some(gen.type_add::<EnumValidator>()))
            .build()
    }
}
