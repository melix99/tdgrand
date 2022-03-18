// Copyright 2020 - developers of the `grammers` project.
// Copyright 2021 - developers of the `tdlib-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Code to generate Rust's `struct`'s from TL definitions.

use crate::ignore_type;
use crate::metadata::Metadata;
use crate::rustifier;
use std::io::{self, Write};
use tdlib_tl_parser::tl::{Category, Definition};

/// Defines the `struct` corresponding to the definition:
///
/// ```ignore
/// pub struct Name {
///     pub field: Type,
/// }
/// ```
fn write_struct<W: Write>(
    file: &mut W,
    def: &Definition,
    _metadata: &Metadata,
    gen_bots_only_api: bool,
) -> io::Result<()> {
    if rustifier::definitions::is_for_bots_only(def) && !gen_bots_only_api {
        return Ok(());
    }

    writeln!(file, "{}", rustifier::definitions::description(def, "    "))?;

    write!(file, "    #[derive(Clone, Debug, ",)?;
    let derive_default = def
        .params
        .iter()
        .all(|p| rustifier::parameters::is_builtin_type(p));
    if derive_default {
        write!(file, "Default, ",)?;
    }
    writeln!(file, "PartialEq, Deserialize, Serialize)]",)?;

    writeln!(
        file,
        "    pub struct {} {{",
        rustifier::definitions::type_name(def),
    )?;

    for param in def.params.iter() {
        if rustifier::parameters::is_for_bots_only(param) && !gen_bots_only_api {
            continue;
        }

        writeln!(
            file,
            "{}",
            rustifier::parameters::description(param, "        ")
        )?;

        if let Some(serde_with) = rustifier::parameters::serde_with(param) {
            writeln!(file, "        #[serde(with = \"{}\")]", serde_with)?;
        }
        write!(
            file,
            "        pub {}: ",
            rustifier::parameters::attr_name(param),
        )?;

        let is_optional = rustifier::parameters::is_optional(param);
        if is_optional {
            write!(file, "Option<")?;
        }
        write!(file, "{}", rustifier::parameters::qual_name(param))?;
        if is_optional {
            write!(file, ">")?;
        }

        writeln!(file, ",")?;
    }

    writeln!(file, "    }}")?;
    Ok(())
}

/// Writes an entire definition as Rust code (`struct`).
fn write_definition<W: Write>(
    file: &mut W,
    def: &Definition,
    metadata: &Metadata,
    gen_bots_only_api: bool,
) -> io::Result<()> {
    write_struct(file, def, metadata, gen_bots_only_api)?;
    Ok(())
}

/// Write the entire module dedicated to types.
pub(crate) fn write_types_mod<W: Write>(
    mut file: &mut W,
    definitions: &[Definition],
    metadata: &Metadata,
    gen_bots_only_api: bool,
) -> io::Result<()> {
    // Begin outermost mod
    writeln!(file, "pub mod types {{")?;
    writeln!(file, "    use serde::{{Deserialize, Serialize}};")?;

    let types = definitions
        .iter()
        .filter(|d| d.category == Category::Types && !ignore_type(&d.ty) && !d.params.is_empty());

    for definition in types {
        write_definition(&mut file, definition, metadata, gen_bots_only_api)?;
    }

    // End outermost mod
    writeln!(file, "}}")
}
