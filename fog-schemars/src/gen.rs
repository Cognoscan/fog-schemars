use std::collections::{BTreeMap, HashMap};

use fog_pack::{
    document::Document,
    schema::{Compress, SchemaBuilder},
    types::Integer,
    validator::Validator,
};

use crate::{FogValidate, Identifier, Name};

#[derive(Clone, Debug)]
pub struct SchemaGenerator {
    // Core Schema components
    doc: Option<(Validator, String)>,
    max_regex: u8,
    description: Option<String>,
    name: Option<String>,
    doc_compress: Option<Compress>,
    version: Option<Integer>,
    types: BTreeMap<String, Validator>,
    entries: BTreeMap<String, EntryItem>,

    // Name shortening stuff
    long_names: Vec<Name>,
    ident_tracker: BTreeMap<&'static str, IdentTracker>,
}

#[derive(Clone, Default, Debug)]
struct IdentTracker {
    exists: bool,
    map: BTreeMap<&'static str, IdentTracker>,
}

struct FixupState<'a> {
    map: &'a mut HashMap<Identifier, Identifier>,
    new_mods: Vec<&'static str>,
    full_mods: Vec<&'static str>,
    name: &'static str,
}

impl<'a> FixupState<'a> {
    // Insert an identifier shortener at the current state (provided the
    // identifier is different)
    fn shorten(&mut self) {
        if self.new_mods == self.full_mods {
            return;
        }
        assert!(
            self.map
                .insert(
                    Identifier {
                        mods: self.full_mods.clone(),
                        name: self.name
                    },
                    Identifier {
                        mods: self.new_mods.clone(),
                        name: self.name
                    },
                )
                .is_none(),
            "Shouldn't be creating more than one shortener for a unique name"
        );
    }
}

impl IdentTracker {
    // Update the name tracker based on this incoming token set
    fn modify<T>(&mut self, mut mods: T)
    where
        T: Iterator<Item = &'static str>,
    {
        match mods.next() {
            None => self.exists = true,
            Some(s) => self.map.entry(s).or_default().modify(mods),
        }
    }

    // Try to shorten the names in this name tracker.
    fn shorten(&self, state: &mut FixupState) {
        if self.exists {
            state.shorten();
        }
        if !self.exists && self.map.len() == 1 {
            let (k, v) = self.map.first_key_value().unwrap();
            state.full_mods.push(k);
            v.shorten(state);
            state.full_mods.pop();
        } else {
            for (k, v) in self.map.iter() {
                state.full_mods.push(k);
                state.new_mods.push(k);
                v.shorten(state);
                state.full_mods.pop();
                state.new_mods.pop();
            }
        }
    }
}

#[derive(Clone, Debug)]
struct EntryItem {
    validator: Validator,
    compress: Option<Compress>,
    type_name: String,
}

impl SchemaGenerator {
    /// Start generating a new schema for a document that passes the validator
    /// for the given type.
    ///
    /// When the schema has been generated, it's recommended practice to take
    /// the schema hash and compare the generate schema against it in a unit
    /// test, which can ensure the schema stays stable over time.
    pub fn new<T: FogValidate>() -> Self {
        let mut this = Self {
            doc: None,
            max_regex: 0,
            description: None,
            name: None,
            version: None,
            doc_compress: None,
            types: BTreeMap::new(),
            entries: BTreeMap::new(),
            long_names: Vec::new(),
            ident_tracker: BTreeMap::new(),
        };
        this.doc = Some((
            T::validator(&mut this, false),
            T::validator_name(false).to_string(),
        ));
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
                    type_name: T::validator_name(false).to_string(),
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
    pub fn type_add<T: FogValidate>(&mut self) -> Validator {
        self.type_add_inner::<T>(false)
    }

    /// Create a validator for a type, returning either the validator itself or
    /// a reference to it. In general, this should only be called by an
    /// implementation of the [`FogValidate`] trait.
    ///
    /// This is specifically for when a type is in a field that has both a
    /// default implementation and has a `skip_serializing_if` attribute.
    pub fn type_add_opt<T: FogValidate>(&mut self) -> Validator {
        self.type_add_inner::<T>(true)
    }

    fn type_add_inner<T: FogValidate>(&mut self, opt: bool) -> Validator {
        let opt = opt && T::has_opt();
        let name = T::validator_name(opt);
        let name_str = name.to_string();
        // Immediately return a ref if it's already been loaded into the schema.
        if self.types.contains_key(&name_str) {
            return Validator::new_ref(name_str);
        }

        if T::should_reference(opt) {
            // CAREFUL ORDERING: We insert our name into both maps immediately,
            // and only then do we generate the validator. In this way,
            // recursive types won't infinitely recurse - they'll stop once they
            // see their own name in the type list. This does require inserting
            // a placeholder validator, but that's not really a big deal.

            self.types.insert(name_str.clone(), Validator::Any);
            self.ident_tracker
                .entry(name.id.name)
                .or_default()
                .modify(name.id.mods.iter().copied());
            self.long_names.push(name);

            let validator = T::validator(self, opt);
            self.types.insert(name_str.clone(), validator);
            Validator::new_ref(name_str)
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
        // We're ready to build. Begin by shortening identifiers as much as we can.
        let mut ident_fixups = HashMap::new();
        for (k, v) in self.ident_tracker.iter() {
            let mut state = FixupState {
                map: &mut ident_fixups,
                full_mods: Vec::new(),
                new_mods: Vec::new(),
                name: k,
            };
            v.shorten(&mut state);
        }
        // With the identifiers shortened, let's try shortening the names
        let fixups: BTreeMap<String, String> = self
            .long_names
            .into_iter()
            .filter_map(|name| {
                let new_name = name.try_shorten(&ident_fixups);
                if new_name != name {
                    Some((name.to_string(), new_name.to_string()))
                } else {
                    None
                }
            })
            .collect();

        // Function to recursively check for references and perform the renaming
        // as needed. Most of this is to visit every validator in the tree; it's
        // only the `Validator::Ref` case that needs changing.
        fn fixup(fixups: &BTreeMap<String, String>, v: &mut Validator) {
            match v {
                Validator::Ref(s) => {
                    if let Some(new_s) = fixups.get(s) {
                        *s = new_s.clone();
                    }
                }
                Validator::Array(v) => {
                    v.contains.iter_mut().for_each(|v| fixup(fixups, v));
                    v.prefix.iter_mut().for_each(|v| fixup(fixups, v));
                    fixup(fixups, &mut v.items);
                }
                Validator::Map(v) => {
                    v.req.values_mut().for_each(|v| fixup(fixups, v));
                    v.opt.values_mut().for_each(|v| fixup(fixups, v));
                    if let Some(v) = &mut v.values {
                        fixup(fixups, v)
                    }
                }
                Validator::Hash(v) => {
                    if let Some(v) = &mut v.link {
                        fixup(fixups, v)
                    }
                }
                Validator::Enum(v) => {
                    v.0.values_mut().for_each(|v| {
                        if let Some(v) = v {
                            fixup(fixups, v)
                        }
                    });
                }
                Validator::Multi(v) => {
                    v.0.iter_mut().for_each(|v| fixup(fixups, v));
                }

                _ => (),
            }
        }

        // If the document's validator is in our type list, just use that instead of duplicating it.
        // We have to check if the name must be shortened too.
        let (mut doc, doc_id) = self.doc.unwrap();
        if self.types.contains_key(&doc_id) {
            doc = Validator::new_ref(doc_id);
        }
        // Perform the name shortening
        fixup(&fixups, &mut doc);

        // Build the schema
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
        for (k, mut v) in self.entries {
            // If the entry's validator is in our type list, just use that instead of duplicating it.
            if self.types.contains_key(&v.type_name) {
                v.validator = Validator::new_ref(v.type_name);
            }
            // Perform the name shortening
            fixup(&fixups, &mut v.validator);
            builder = builder.entry_add(&k, v.validator, v.compress);
        }
        for (k, mut v) in self.types {
            // Perform the name shortening
            let name = if let Some(name) = fixups.get(&k) {
                name
            } else {
                &k
            };
            fixup(&fixups, &mut v);
            builder = builder.type_add(name, v);
        }
        builder.build()
    }
}
