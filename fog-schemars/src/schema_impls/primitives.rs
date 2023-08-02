use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

use std::sync::atomic::*;

impl FogValidate for bool {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "true" } else { "bool" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            BoolValidator::new().nin_add(false).build()
        } else {
            BoolValidator::new().build()
        }
    }
}

impl FogValidate for str {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "OptStr" } else { "str" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            StrValidator::new().min_len(1).build()
        } else {
            StrValidator::new().build()
        }
    }
}

forward_impl!(String => str);

impl FogValidate for f32 {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "OptF32" } else { "f32" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            F32Validator::new().nin_add(f32::default()).build()
        } else {
            F32Validator::new().build()
        }
    }
}

impl FogValidate for f64 {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "OptF64" } else { "f64" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            F64Validator::new().nin_add(f64::default()).build()
        } else {
            F64Validator::new().build()
        }
    }
}

impl FogValidate for char {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("", if opt { "OptChar" } else { "char" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            StrValidator::new()
                .nin_add(char::default())
                .min_char(1)
                .max_char(1)
                .build()
        } else {
            StrValidator::new().min_char(1).max_char(1).build()
        }
    }
}

macro_rules! unsigned_impl {
    ($type:ty, $non_zero:ty) => {
        impl FogValidate for $type {
            no_ref_validator!();
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                let name: &str = if opt {
                    stringify!($non_zero)
                } else {
                    stringify!($type)
                };
                Name::new("", name)
            }

            fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
                if opt {
                    IntValidator::new()
                        .min(<$type>::MIN + 1)
                        .max(<$type>::MAX)
                        .build()
                } else {
                    IntValidator::new()
                        .min(<$type>::MIN)
                        .max(<$type>::MAX)
                        .build()
                }
            }
        }
    };
}

macro_rules! signed_impl {
    ($type:ty, $non_zero:ty) => {
        impl FogValidate for $type {
            no_ref_validator!();
            has_opt!();

            fn validator_name(opt: bool) -> Name {
                let name: &str = if opt {
                    stringify!($non_zero)
                } else {
                    stringify!($type)
                };
                Name::new("", name)
            }

            fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
                if opt {
                    IntValidator::new()
                        .nin_add(0)
                        .min(<$type>::MIN)
                        .max(<$type>::MAX)
                        .build()
                } else {
                    IntValidator::new()
                        .min(<$type>::MIN)
                        .max(<$type>::MAX)
                        .build()
                }
            }
        }
    };
}

// Purposely excluding usize & isize, as their valid ranges can change depending
// on the architecture this code is run on.
unsigned_impl!(u8, NonZeroU8);
unsigned_impl!(u16, NonZeroU16);
unsigned_impl!(u32, NonZeroU32);
unsigned_impl!(u64, NonZeroU64);
signed_impl!(i8, NonZeroI8);
signed_impl!(i16, NonZeroI16);
signed_impl!(i32, NonZeroI32);
signed_impl!(i64, NonZeroI64);

forward_impl!(AtomicBool => bool);
forward_impl!(AtomicI8 => i8);
forward_impl!(AtomicI16 => i16);
forward_impl!(AtomicI32 => i32);
forward_impl!(AtomicI64 => i64);
forward_impl!(AtomicU8 => u8);
forward_impl!(AtomicU16 => u16);
forward_impl!(AtomicU32 => u32);
forward_impl!(AtomicU64 => u64);