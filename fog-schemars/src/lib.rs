#![forbid(unsafe_code)]

mod gen;
mod schema_impls;

use std::any::TypeId;

pub use gen::SchemaGenerator;

use fog_pack::validator::*;

// Export fog-pack so fog-schemars-derive can always use it
#[doc(hidden)]
pub use fog_pack as _fog_pack;

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
pub trait FogValidate: 'static {
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
    /// In the event of a name conflict that the generator is configured to
    /// resolve, an underscore and a number (starting with 0) are appended to
    /// this name.
    ///
    /// If deriving a schema automatically, this defaults to the type name, or
    /// the one specified by the serde `rename` attribute.
    fn validator_name(opt: bool) -> String;

    /// Generates a fog-pack validator for this type.
    ///
    /// If the returned validator depends on any sub-validators, then it must
    /// get them by calling [`SchemaGenerator::type_add`], with the exception of
    /// setting the [`StrValidator`] for a [`MapValidator`].
    ///
    /// This shouldn't ever return a [`Validator::Ref`].
    fn validator(gen: &mut SchemaGenerator, opt: bool) -> Validator;

    /// The unique TypeId of this type. You shouldn't be overriding this unless
    /// this validator is completely transparent to another type's validator, in
    /// which case this, along with every other one of this trait's functions,
    /// should forward to said type's [`FogValidate`] implementations. Doing so
    /// will ensure fewer redundant validator definitions.
    fn validator_type_id(opt: bool) -> TypeId {
        TypeId::of::<Self>()
    }

}
