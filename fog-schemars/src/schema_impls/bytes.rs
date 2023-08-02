use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

use bytes::{Bytes, BytesMut};

impl FogValidate for Bytes {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("bytes", if opt { "OptBytes" } else { "Bytes" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        let mut v = BinValidator::new();
        if opt { v = v.min_len(1); }
        v.build()
    }
}

forward_impl!(BytesMut => Bytes);