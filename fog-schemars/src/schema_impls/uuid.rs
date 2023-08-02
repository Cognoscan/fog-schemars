use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;
use uuid::Uuid;

impl FogValidate for Uuid {
    no_ref_validator!();
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("uuid", if opt { "OptUuid" } else { "Uuid" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        let mut v = BinValidator::new().min_len(16).max_len(16);
        if opt {
            // Exclude the default value
            v = v.nin_add(Uuid::default().as_bytes().to_owned());
        }
        v.build()
    }
}
