use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

use fog_pack::{
    document::Document,
    schema::{Compress, SchemaBuilder},
    types::Integer,
    validator::Validator,
};

use crate::FogValidate;

pub struct SchemaGenerator {
    doc: Option<(Validator, TypeId)>,
    max_regex: u8,
    description: Option<String>,
    name: Option<String>,
    doc_compress: Option<Compress>,
    version: Option<Integer>,
    types: BTreeMap<String, Validator>,
    entries: BTreeMap<String, EntryItem>,
    opt_name_tracker: HashMap<TypeId, String>,
    name_tracker: HashMap<TypeId, String>,
    types_source_names: BTreeMap<String, String>,
    name_collisions_ok: bool,
}

struct EntryItem {
    validator: Validator,
    compress: Option<Compress>,
    type_id: TypeId,
}

impl SchemaGenerator {
    /// Start generating a new schema. This is private because we're supposed to
    /// always start off with a validator for the document, but we can't
    /// actually do that so we're only making it *look* like that's so.
    fn init() -> Self {
        Self {
            doc: None,
            max_regex: 0,
            description: None,
            name: None,
            version: None,
            doc_compress: None,
            types: BTreeMap::new(),
            entries: BTreeMap::new(),
            name_tracker: HashMap::new(),
            opt_name_tracker: HashMap::new(),
            types_source_names: BTreeMap::new(),
            name_collisions_ok: false,
        }
    }

    /// Start generating a new schema for a document that passes the validator
    /// for the given type.
    ///
    /// When the schema has been generated, it's recommended practice to take
    /// the schema hash and compare the generate schema against it in a unit
    /// test, which can ensure the schema stays stable over time.
    pub fn new<T: FogValidate>() -> Self {
        let mut this = Self::init();
        this.doc = Some((T::validator(&mut this, false), T::validator_type_id(false)));
        this
    }

    /// Start generating a new schema for a document that passes the validator
    /// for the given type, deconflicting names as needed.
    ///
    /// Normally, when two [`FogValidate`] implementing types have conflicting
    /// names, a generator will panic. This function bypasses that and appends
    /// an underscore and number to the end to deconflict the names, starting
    /// with the number 0 and incrementing with each new conflict. This
    /// potentially introduces some big unexpected stability requirement: struct
    /// field ordering and enum variant ordering must remain the same when
    /// deriving the [`FogValidate`] trait, and direct implementation must add
    /// types in the same order. Additionally, if a new field or variant is
    /// added, the conflict ordering could also change. In all of these cases,
    /// the conflict resolution mechanism can change the generated schema,
    /// breaking consistency.
    ///
    /// Like with [`new`][SchemaGenerator::new],  it's recommended practice to
    /// take the schema hash and compare the generated schema against it in a
    /// unti test, ensuring the schema stays stable over time.
    pub fn new_allow_name_conflicts<T: FogValidate>() -> Self {
        let mut this = Self::init();
        this.name_collisions_ok = true;
        this.doc = Some((T::validator(&mut this, false), T::validator_type_id(false)));
        this
    }

    /// Add a description to the schema. This is only used for documentation
    /// purposes.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }

    /// Set the default compression to use for documents adhering to this schema.
    pub fn doc_compress(mut self, doc_compress: Compress) -> Self {
        self.doc_compress = Some(doc_compress);
        self
    }

    /// Add a new entry type to the schema, where `entry` is the key for the
    /// entry, the validator for type T will be used to validate each entry, and
    /// `compress` optionally overrides the default compression with a specific
    /// compression setting.
    ///
    /// Panics
    /// ------
    /// This function panics if there's already an entry validator with the
    /// provided name.
    pub fn entry_add<T: FogValidate>(mut self, entry: &str, compress: Option<Compress>) -> Self {
        let validator = T::validator(&mut self, false);
        if self
            .entries
            .insert(
                entry.to_owned(),
                EntryItem {
                    validator,
                    compress,
                    type_id: T::validator_type_id(false),
                },
            )
            .is_some()
        {
            panic!("An entry table item ({:?}) is being overwritten! You probably didn't mean to do this.", entry);
        }
        self
    }

    /// Set the schema name. This is only used for documentation purposes.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Create a validator for a type, returning either the validator itself or
    /// a reference to it. In general, this should only be called by an
    /// implementation of the [`FogValidate`] trait.
    ///
    /// Panics
    /// ------
    /// This function panics if type name conflicts aren't allowed and one occurs.
    pub fn type_add<T: FogValidate>(&mut self) -> Validator {
        self.type_add_inner::<T>(false)
    }

    /// Create a validator for a type, returning either the validator itself or
    /// a reference to it. In general, this should only be called by an
    /// implementation of the [`FogValidate`] trait.
    ///
    /// This is specifically for when a type is in a field that has both a
    /// default implementation and has a `skip_serializing_if` attribute.
    ///
    /// Panics
    /// ------
    /// This function panics if type name conflicts aren't allowed and one occurs.
    pub fn type_add_opt<T: FogValidate>(&mut self) -> Validator {
        self.type_add_inner::<T>(true)
    }

    fn type_add_inner<T: FogValidate>(&mut self, opt: bool) -> Validator {
        let opt = opt && T::has_opt();
        let type_id = T::validator_type_id(opt);
        // Immediately return a ref if it's already been loaded into the schema.
        if opt {
            if let Some(name) = self.opt_name_tracker.get(&type_id) {
                return Validator::new_ref(name);
            }
        } else if let Some(name) = self.name_tracker.get(&type_id) {
            return Validator::new_ref(name);
        }

        if T::should_reference(opt) {
            // Check for name conflict and de-conflict if we are allowed to do so.
            let name = T::validator_name(opt);
            let name = if self.types.contains_key(&name) {
                if !self.name_collisions_ok {
                    let this_type = std::any::type_name::<T>();
                    let other_type = self.types_source_names.get(&name).unwrap();
                    panic!(
                        "Schema experienced a name conflict for {}, between types {} and {}. \
                            Try deconflicting the names (ideally) or allow for name conflicts \
                            during generation",
                        name, this_type, other_type
                    );
                }
                let mut new_name = name.clone();
                let mut index = 0;
                new_name.push_str("_0");
                while self.types.contains_key(&name) {
                    new_name.truncate(name.len());
                    use std::fmt::Write;
                    index += 1;
                    write!(new_name, "_{}", index).unwrap();
                }
                new_name
            } else {
                name
            };

            // CAREFUL ORDERING: We insert our name into both maps immediately,
            // and only then do we generate the validator. In this way,
            // recursive types won't infinitely recurse - they'll stop once they
            // see their own TypeID in the name tracker. We also need to load a
            // placeholder validator into the type map, as otherwise another
            // validator might overwrite our chosen name during a name conflic.
            if opt {
                self.opt_name_tracker.insert(type_id, name.clone());
            } else {
                self.name_tracker.insert(type_id, name.clone());
            }
            self.types_source_names
                .insert(name.clone(), std::any::type_name::<T>().into());
            self.types.insert(name.clone(), Validator::Any);
            let validator = T::validator(self, opt);
            self.types.insert(name.clone(), validator);
            Validator::new_ref(name)
        } else {
            T::validator(self, opt)
        }
    }

    /// Set the schema version. This is only used for documentation purposes.
    pub fn version<T: Into<Integer>>(mut self, version: T) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the maximum number of regexes allowed in a query.
    pub fn regexes(mut self, max_regex: u8) -> Self {
        self.max_regex = max_regex;
        self
    }

    pub fn build(self) -> fog_pack::error::Result<Document> {
        // If the document's validator is in our type list, just use that instead of duplicating it.
        let (mut doc, doc_id) = self.doc.unwrap();
        if let Some(name) = self.name_tracker.get(&doc_id) {
            doc = Validator::new_ref(name);
        }
        let mut builder = SchemaBuilder::new(doc).regexes(self.max_regex);
        if let Some(v) = self.description {
            builder = builder.description(&v);
        }
        if let Some(v) = self.name {
            builder = builder.name(&v);
        }
        if let Some(v) = self.doc_compress {
            builder = builder.doc_compress(v);
        }
        if let Some(v) = self.version {
            builder = builder.version(v);
        }
        for (k, v) in self.types {
            builder = builder.type_add(&k, v);
        }
        for (k, v) in self.entries {
            // If the entry'e validator is in our type list, just use that instead of duplicating it.
            let validator = if let Some(name) = self.name_tracker.get(&v.type_id) {
                Validator::new_ref(name)
            } else {
                v.validator
            };
            builder = builder.entry_add(&k, validator, v.compress);
        }
        builder.build()
    }
}
