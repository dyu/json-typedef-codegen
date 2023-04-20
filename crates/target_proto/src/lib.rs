use jtd_codegen::target::{self, inflect, metadata, Field};
use jtd_codegen::Result;
use lazy_static::lazy_static;
//use serde_json::Value;
use std::collections::{BTreeSet};
use std::io::Write;
//use std::simd::MaskElement;

lazy_static! {
    static ref KEYWORDS: BTreeSet<String> = include_str!("keywords")
        .lines()
        .map(str::to_owned)
        .collect();
    static ref TYPE_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            //KEYWORDS.clone(),
            inflect::CombiningInflector::new(inflect::Case::pascal_case())
        ));
    static ref FIELD_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            //KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::snake_case())
        ));
    static ref ENUM_MEMBER_NAMING_CONVENTION: Box<dyn inflect::Inflector + Send + Sync> =
        Box::new(KeywordAvoidingInflector::new(
            //KEYWORDS.clone(),
            inflect::TailInflector::new(inflect::Case::screaming_snake_case())
        ));
}

pub struct KeywordAvoidingInflector<I> {
    //keywords: BTreeSet<String>,
    inflector: I,
}

impl<I> KeywordAvoidingInflector<I> {
    pub fn new(/*keywords: BTreeSet<String>, */inflector: I) -> Self {
        Self {
            //keywords,
            inflector,
        }
    }
}

impl<I: inflect::Inflector> inflect::Inflector for KeywordAvoidingInflector<I> {
    fn inflect(&self, words: &[String]) -> String {
        let raw_name = self.inflector.inflect(words);
        /*/
        if self.keywords.contains(&raw_name) {
            format!("`{}`", raw_name)
        } else {
            raw_name
        }
        */
        raw_name
    }
}

pub struct Target {
    filename: String,
    package: String,
    emit_required_fields: bool,
}

impl Target {
    pub fn new(filename: String, package: String, emit_required_fields: bool) -> Self {
        Self { filename, package, emit_required_fields }
    }
}

impl jtd_codegen::target::Target for Target {
    type FileState = FileState;

    fn strategy(&self) -> target::Strategy {
        target::Strategy {
            file_partitioning: target::FilePartitioningStrategy::SingleFile(format!(
                "{}.proto",
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
        _: &mut FileState,
        metadata: metadata::Metadata,
        expr: target::Expr,
    ) -> String {
        _ = metadata;
        // if let Some(s) = metadata.get("pythonType").and_then(|v| v.as_str()) {
        //     return s.into();
        // }

        match expr {
            target::Expr::Empty => "reserved".into(),
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
            target::Expr::ArrayOf(sub_expr) => format!("repeated {}", sub_expr),
            target::Expr::DictOf(sub_expr) => format!("map<string, {}>", sub_expr),
            target::Expr::NullableOf(sub_expr) => sub_expr,
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
                    "// Code generated by jtd-codegen for proto v{}\n\npackage {};\n",
                    env!("CARGO_PKG_VERSION"),
                    self.package,
                )?;

                for import in &state.imports {
                    writeln!(out, "import {};", import)?;
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
                writeln!(out, "//alias {} = {}\n", name, type_)?;

                None
            }

            target::Item::Enum {
                metadata,
                name,
                members,
            } => {
                _ = metadata;
                writeln!(out, "enum {} {{", name)?;
                let mut numeric = false;
                
                //for member in &members {
                for (index, member) in members.into_iter().enumerate() {
                    if index == 0 {
                        // only test the first member
                        numeric = member.json_value.chars().all(char::is_numeric);
                    }
                    write!(out, "  {} = ", member.name)?;
                    if numeric {
                        writeln!(out, "{};", member.json_value)?;
                    } else {
                        writeln!(out, "{}; // {:?}", index + 1, member.json_value)?;
                    }
                }
                writeln!(out, "}}\n")?;

                None
            }

            target::Item::Struct {
                metadata,
                name,
                has_additional: _,
                fields,
            } => {
                _ = metadata;
                writeln!(out, "message {} {{", name)?;
                //write!(out, "{}", description(&metadata, 1))?;
                
                let mut required_fields: Vec<&Field> = Vec::new();
                let mut optional_fields: Vec<&Field> = Vec::new();
                for field in &fields {
                    if self.emit_required_fields && !field.optional && !field.type_.starts_with("repeated ") {
                        required_fields.push(field);
                    } else {
                        optional_fields.push(field);
                    }
                }
                let mut count: usize = 0;
                if self.emit_required_fields {
                    for (_, field) in required_fields.into_iter().enumerate() {
                        count += 1;
                        if field.name != field.json_name || field.json_name.contains('_') {
                            writeln!(out, "  required {} {} = {} [ alias = \"{}\" ];",
                                    field.type_, field.name, count, field.json_name)?;
                        } else {
                            writeln!(out, "  required {} {} = {};",
                                    field.type_, field.name, count)?;
                        }

                    }
                }
                
                optional_fields.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
                
                for (_, field) in optional_fields.into_iter().enumerate() {
                    count += 1;
                    if field.type_.starts_with("repeated ") {
                        write!(out, "  ")?;
                    } else {
                        write!(out, "  optional ")?;
                    }
                    if field.name != field.json_name || field.json_name.contains('_') {
                        writeln!(out, "{} {} = {} [ alias = \"{}\" ];",
                                field.type_, field.name, count, field.json_name)?;
                    } else {
                        writeln!(out, "{} {} = {};",
                                field.type_, field.name, count)?;
                    }
                }
                
                writeln!(out, "}}\n")?;
                
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
    imports: BTreeSet<String>,
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
        jtd_codegen_test::std_test_cases!(&crate::Target::new("jtd_codegen_e2e".into(), "jtd_codegen_e2e".into(), false));
    }

    mod optional_std_tests {
        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into(), "jtd_codegen_e2e".into(), false),
            empty_and_nonascii_properties
        );

        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into(), "jtd_codegen_e2e".into(), false),
            empty_and_nonascii_enum_values
        );
    }
}
