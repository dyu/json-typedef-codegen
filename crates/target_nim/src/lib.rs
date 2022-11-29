use jtd_codegen::target::{self, inflect, metadata};
use jtd_codegen::Result;
use lazy_static::lazy_static;
//use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
//use std::simd::MaskElement;

lazy_static! {
    static ref KEYWORDS: BTreeSet<String> = include_str!("keywords")
        .lines()
        .map(str::to_owned)
        .collect();
    static ref TYPE_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::CombiningInflector::new(inflect::Case::pascal_case())
        ));
    static ref FIELD_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::snake_case())
        ));
    static ref ENUM_MEMBER_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::screaming_snake_case())
        ));
}

pub struct KeywordAvoidingInflector<I> {
    keywords: BTreeSet<String>,
    inflector: I,
}

impl<I> KeywordAvoidingInflector<I> {
    pub fn new(keywords: BTreeSet<String>, inflector: I) -> Self {
        Self {
            keywords,
            inflector,
        }
    }
}

impl<I: inflect::Inflector> inflect::Inflector for KeywordAvoidingInflector<I> {
    fn inflect(&self, words: &[String]) -> String {
        let raw_name = self.inflector.inflect(words);

        if self.keywords.contains(&raw_name) {
            format!("`{}`", raw_name)
        } else {
            raw_name
        }
    }
}

pub struct Target {
    filename: String,
}

impl Target {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl jtd_codegen::target::Target for Target {
    type FileState = FileState;

    fn strategy(&self) -> target::Strategy {
        target::Strategy {
            file_partitioning: target::FilePartitioningStrategy::SingleFile(format!(
                "{}.nim",
                self.filename,
            )),
            enum_member_naming: target::EnumMemberNamingStrategy::Modularized,
            optional_property_handling: target::OptionalPropertyHandlingStrategy::WrapWithNullable,
            booleans_are_nullable: false,
            int8s_are_nullable: false,
            uint8s_are_nullable: false,
            int16s_are_nullable: false,
            uint16s_are_nullable: false,
            int32s_are_nullable: false,
            uint32s_are_nullable: false,
            float32s_are_nullable: false,
            float64s_are_nullable: false,
            strings_are_nullable: false,
            timestamps_are_nullable: false,
            arrays_are_nullable: false,
            dicts_are_nullable: false,
            aliases_are_nullable: false,
            enums_are_nullable: false,
            structs_are_nullable: false,
            discriminators_are_nullable: false,
        }
    }

    fn name(&self, kind: target::NameableKind, parts: &[String]) -> String {
        match kind {
            target::NameableKind::Type => TYPE_NAMING_CONVENTION.inflect(parts),
            target::NameableKind::Field => FIELD_NAMING_CONVENTION.inflect(parts),
            target::NameableKind::EnumMember => ENUM_MEMBER_NAMING_CONVENTION.inflect(parts),
        }
    }

    fn expr(
        &self,
        state: &mut FileState,
        metadata: metadata::Metadata,
        expr: target::Expr,
    ) -> String {
        _ = metadata;
        // if let Some(s) = metadata.get("pythonType").and_then(|v| v.as_str()) {
        //     return s.into();
        // }

        match expr {
            target::Expr::Empty => {
                state
                    .imports
                    .entry("json".into())
                    .or_default()
                    .insert("JsonNode".into());

                "JsonNode".into()
            }
            target::Expr::Boolean => "bool".into(),
            target::Expr::Int8 => "int8".into(),
            target::Expr::Uint8 => "uint8".into(),
            target::Expr::Int16 => "int16".into(),
            target::Expr::Uint16 => "uint16".into(),
            target::Expr::Int32 => "int32".into(),
            target::Expr::Uint32 => "uint32".into(),
            target::Expr::Float32 => "float32".into(),
            target::Expr::Float64 => "float64".into(),
            target::Expr::String => "string".into(),
            target::Expr::Timestamp => "uint64".into(),
            target::Expr::ArrayOf(sub_expr) => format!("seq[{}]", sub_expr),
            target::Expr::DictOf(sub_expr) => {
                state
                    .imports
                    .entry("tables".into())
                    .or_default()
                    .insert("Table".into());

                format!("Table[string, {}]", sub_expr)
            }
            target::Expr::NullableOf(sub_expr) => {
                state
                    .imports
                    .entry("options".into())
                    .or_default()
                    .insert("Option".into());

                format!("Option[{}]", sub_expr)
            }
        }
    }

    fn item(
        &self,
        out: &mut dyn Write,
        state: &mut FileState,
        item: target::Item,
    ) -> Result<Option<String>> {
        Ok(match item {
            target::Item::Auxiliary { .. } => {
                // No auxiliary files needed.
                None
            }

            target::Item::Preamble => {
                writeln!(
                    out,
                    "# Code generated by jtd-codegen for Nim v{}",
                    env!("CARGO_PKG_VERSION")
                )?;
                writeln!(out)?;
                // writeln!(
                //     out,
                //     "import jsony",
                // )?;
                // writeln!(out)?;

                for (module, idents) in &state.imports {
                    writeln!(
                        out,
                        "from {} import {}",
                        module,
                        idents.iter().cloned().collect::<Vec<_>>().join(", ")
                    )?;
                }

                writeln!(out)?;

                None
            }

            target::Item::Postamble => {
                writeln!(out)?;

                None
            }

            target::Item::Alias {
                metadata,
                name,
                type_,
            } => {
                _ = metadata;
                writeln!(out)?;
                writeln!(out, "type {}* = {}", name, type_)?;

                None
            }

            target::Item::Enum {
                metadata,
                name,
                members,
            } => {
                _ = metadata;
                writeln!(out)?;
                writeln!(out, "type {}* {{.pure.}} = enum", name)?;
                for member in &members {
                    writeln!(out, "    {} = {:?}", member.name, member.json_value)?;
                }

                None
            }

            target::Item::Struct {
                metadata,
                name,
                has_additional: _,
                fields,
            } => {
                _ = metadata;
                writeln!(out)?;
                writeln!(out, "type {}* = object", name)?;
                //write!(out, "{}", description(&metadata, 1))?;
                for field in &fields {
                    writeln!(out, "    {}*: {}", field.name, field.type_,)?;
                }

                None
            }

            target::Item::Discriminator {
                metadata,
                name,
                tag_field_name,
                tag_json_name,
                variants,
            } => {
                _ = metadata;
                _ = name;
                _ = tag_field_name;
                _ = tag_json_name;
                _ = variants;
                writeln!(out)?;
                None
            }

            target::Item::DiscriminatorVariant {
                metadata,
                name,
                parent_name,
                tag_json_name,
                tag_value,
                fields,
                ..
            } => {
                _ = metadata;
                _ = name;
                _ = parent_name;
                _ = tag_json_name;
                _ = tag_value;
                _ = fields;
                writeln!(out)?;

                None
            }
        })
    }
}

#[derive(Default)]
pub struct FileState {
    imports: BTreeMap<String, BTreeSet<String>>,
}

/*
fn description(metadata: &BTreeMap<String, Value>, indent: usize) -> String {
    doc(indent, jtd_codegen::target::metadata::description(metadata))
}

fn enum_variant_description(
    metadata: &BTreeMap<String, Value>,
    indent: usize,
    value: &str,
) -> String {
    doc(
        indent,
        jtd_codegen::target::metadata::enum_variant_description(metadata, value),
    )
}

fn doc(ident: usize, s: &str) -> String {
    let prefix = "    ".repeat(ident);
    let out = jtd_codegen::target::fmt::comment_block(
        &format!("{}\"\"\"", prefix),
        &format!("{}", prefix),
        &format!("{}\"\"\"", prefix),
        s,
    );

    if out.is_empty() {
        out
    } else {
        out + "\n"
    }
}
*/

#[cfg(test)]
mod tests {
    mod std_tests {
        jtd_codegen_test::std_test_cases!(&crate::Target::new("jtd_codegen_e2e".into()));
    }

    mod optional_std_tests {
        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into()),
            empty_and_nonascii_properties
        );

        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into()),
            empty_and_nonascii_enum_values
        );
    }
}