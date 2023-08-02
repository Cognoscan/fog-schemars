#![forbid(unsafe_code)]

mod gen;
mod schema_impls;

use std::collections::HashMap;

pub use gen::SchemaGenerator;

use fog_pack::validator::*;

// Export fog-pack so fog-schemars-derive can always use it
#[doc(hidden)]
pub use fog_pack as _fog_pack;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    id: Identifier,
    type_params: Vec<Name>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Identifier {
    mods: Vec<&'static str>,
    name: &'static str,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for m in self.mods.iter() {
            write!(f, "{}::", m)?;
        }
        f.write_str(self.name)
    }
}

impl Name {
    pub fn new(path: &'static str, name: &'static str) -> Self {
        Self::with_types(path, name, Vec::new())
    }

    pub fn with_types(path: &'static str, name: &'static str, type_params: Vec<Name>) -> Self {
        assert!(
            !path.contains([',', '<', '>']),
            "Path shouldn't contain any of \",<>\""
        );
        assert!(
            !name.contains([',', '<', '>', ':']),
            "Name shouldn't contain any of \":,<>\""
        );
        let mods: Vec<&'static str> = path.split("::").collect();
        Self {
            id: Identifier { mods, name },
            type_params,
        }
    }

    pub(crate) fn try_shorten(&self, map: &HashMap<Identifier, Identifier>) -> Name {
        let id = if let Some(id) = map.get(&self.id) {
            id.clone()
        } else {
            self.id.clone()
        };
        let type_params = self
            .type_params
            .iter()
            .map(|p| p.try_shorten(map))
            .collect();
        Name { id, type_params }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        if let Some((first, rest)) = self.type_params.split_first() {
            write!(f, "<{}", first)?;
            for r in rest {
                write!(f, ",{}", r)?;
            }
            f.write_str(">")?;
        }
        Ok(())
    }
}

#[cfg(feature = "fog-schemars-derive")]
pub use fog_schemars_derive::*;

/// The core trait for automatic generation of fog-pack validators.
///
/// When implementing this trait, there may be two different types of validators
/// generated: one for most cases, and one when generating for a field marked as
/// optional in a parent [`MapValidator`] - which is detected in derived
/// implementations by checking for the serde attributes `default` and
/// `skip_serializing_if`. In the latter case, the `opt` flag is set on all
/// calls into this trait. This can be thought of as each trait implementation
/// returning up to two separate validators.
#[allow(unused_variables)]
pub trait FogValidate {
    /// Whether or not the fog-pack Validator for this type should be reused where possible through
    /// a Schema's type list.
    ///
    /// For extremely simple types (eg. primitives), this should return `false`. For anything
    /// remotely complex, it should return `true`. For types that may recurse on themselves
    /// (indirectly or otherwise), it **must** return `true` or the generator logic will cycle
    /// infinitely until running out of memory and crashing.
    fn should_reference(opt: bool) -> bool {
        true
    }

    /// Whether or not this has a different validator when the `opt` flag is
    /// true. This function defaults to returning `false`.
    fn has_opt() -> bool {
        false
    }

    /// A name to associate with the generated Validator.
    ///
    /// If deriving a schema automatically, this defaults to the module path
    /// plus the type name, or the one specified by the serde `rename`
    /// attribute. Implementors of this trait should prefer the format style of
    /// `module_path::Type<TypeParam1,TypeParam2>`, where the module path is
    /// obtained from the [`std::module_path!`] macro.
    ///
    /// When completing the schema, module paths are shortened if there are no
    /// conflicts. Examples, for a type named "Config":
    /// - `crate::foo::Config` and `crate::bar::Config` will shorten to
    ///   `foo::Config` and `bar::Config`.
    /// - `foo::thing::Config` and `bar::thing::Config` will not change.
    ///
    fn validator_name(opt: bool) -> Name;

    /// Generates a fog-pack validator for this type.
    ///
    /// If the returned validator depends on any sub-validators, then it must
    /// get them by calling [`SchemaGenerator::type_add`], with the exception of
    /// setting the [`StrValidator`] for a [`MapValidator`].
    ///
    /// This shouldn't ever return a [`Validator::Ref`].
    fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator;
}
