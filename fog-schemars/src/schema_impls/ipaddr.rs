use crate::{FogValidate, Name, SchemaGenerator};
use fog_pack::validator::*;

forward_impl!((FogValidate for std::net::Ipv4Addr) => [u8;4]);
forward_impl!((FogValidate for std::net::Ipv6Addr) => [u8;16]);
forward_impl!((FogValidate for std::net::SocketAddrV4) => ([u8;4], u16));
forward_impl!((FogValidate for std::net::SocketAddrV6) => ([u8;16], u16));

impl FogValidate for std::net::IpAddr {
    fn validator_name(_: bool) -> Name {
        Name::new("std::net", "IpAddr")
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        EnumValidator::new()
            .insert("V4", Some(gen.type_add::<std::net::Ipv4Addr>()))
            .insert("V6", Some(gen.type_add::<std::net::Ipv6Addr>()))
            .build()
    }
}

impl FogValidate for std::net::SocketAddr {
    fn validator_name(_: bool) -> Name {
        Name::new("std::net", "IpAddr")
    }

    fn validator(gen: &mut SchemaGenerator, _: bool) -> Validator {
        EnumValidator::new()
            .insert("V4", Some(gen.type_add::<std::net::SocketAddrV4>()))
            .insert("V6", Some(gen.type_add::<std::net::SocketAddrV6>()))
            .build()
    }
}
