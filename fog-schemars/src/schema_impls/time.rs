use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::types::Value;
use fog_pack::validator::*;

use std::time::{Duration, SystemTime};

impl FogValidate for Duration {
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("std::time", if opt { "OptDuration" } else { "Duration" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            MapValidator::new()
                .nin_add([
                    (String::from("secs"), Value::from(0u64)),
                    (String::from("nanos"), Value::from(0u32)),
                ])
                .req_add("secs", IntValidator::new().min(0).max(u64::MAX).build())
                .req_add(
                    "nanos",
                    IntValidator::new().min(0).max(1_000_000_000 - 1).build(),
                )
                .build()
        } else {
            MapValidator::new()
                .req_add("secs", IntValidator::new().min(0).max(u64::MAX).build())
                .req_add(
                    "nanos",
                    IntValidator::new().min(0).max(1_000_000_000 - 1).build(),
                )
                .build()
        }
    }
}

/// SystemTime is not serialized as a fog-pack timestamp. Consider using fog-pack Timestamps instead.
impl FogValidate for SystemTime {
    has_opt!();

    fn validator_name(opt: bool) -> Name {
        Name::new("std::time", if opt { "OptDuration" } else { "Duration" })
    }

    fn validator(_: &mut SchemaGenerator, opt: bool) -> Validator {
        if opt {
            MapValidator::new()
                .nin_add([
                    (String::from("secs_since_epoch"), Value::from(0u64)),
                    (String::from("nanos_since_epoch"), Value::from(0u32)),
                ])
                .req_add(
                    "secs_since_epoch",
                    IntValidator::new().min(0).max(u64::MAX).build(),
                )
                .req_add(
                    "nanos_since_epoch",
                    IntValidator::new().min(0).max(1_000_000_000 - 1).build(),
                )
                .build()
        } else {
            MapValidator::new()
                .req_add(
                    "secs_since_epoch",
                    IntValidator::new().min(0).max(u64::MAX).build(),
                )
                .req_add(
                    "nanos_since_epoch",
                    IntValidator::new().min(0).max(1_000_000_000 - 1).build(),
                )
                .build()
        }
    }
}
