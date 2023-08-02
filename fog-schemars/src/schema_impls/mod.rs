#[allow(unused_imports)]
use crate::{FogValidate, Name, SchemaGenerator};
#[allow(unused_imports)]
use fog_pack::validator::*;

macro_rules! no_ref_validator {
    () => {
        fn should_reference(_: bool) -> bool {
            false
        }
    };
}

macro_rules! has_opt {
    () => {
        fn has_opt() -> bool {
            true
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
                <$target>::has_opt()
            }

            fn validator_name(opt: bool) -> Name {
                <$target>::validator_name(opt)
            }

            fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator {
                <$target>::validator(gen, opt)
            }
        }
    };
    ($ty:ty => $target:ty) => {
        forward_impl!((FogValidate for $ty) => $target);
    };
}

mod array;
mod core;
mod primitives;
mod sequences;
mod maps;
mod tuple;
mod wrapper;
mod nonzero;
mod time;
mod ipaddr;
mod fog_pack_impls;

#[cfg(feature = "uuid")]
mod uuid;

#[cfg(feature = "serde_bytes")]
mod serde_bytes;

#[cfg(feature = "bytes")]
mod bytes;

#[cfg(feature = "smol_str")]
forward_impl!(smol_str::SmolStr => str);

#[cfg(feature = "smartstring")]
forward_impl!((<M: smartstring::SmartStringMode> FogValidate for smartstring::SmartString<M>) => str);

#[cfg(feature = "smallstr")]
forward_impl!((<A: smallvec::Array<Item=u8>> FogValidate for smallstr::SmallString<A>) => str);

#[cfg(feature = "smallvec")]
forward_impl!((<T: FogValidate + smallvec::Array> FogValidate for smallvec::SmallVec<T>) => Vec<T>);