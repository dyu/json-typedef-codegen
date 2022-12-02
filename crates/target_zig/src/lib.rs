use jtd_codegen::target::{self, inflect, metadata};
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
            format!("@\"{}\"", raw_name)
        } else {
            raw_name
        }
    }
}

pub struct Target {
    filename: String,
    with_optionals: bool,
}

impl Target {
    pub fn new(filename: String, with_optionals: bool) -> Self {
        Self { filename, with_optionals }
    }
}

impl jtd_codegen::target::Target for Target {
    type FileState = FileState;

    fn strategy(&self) -> target::Strategy {
        target::Strategy {
            file_partitioning: target::FilePartitioningStrategy::SingleFile(format!(
                "{}.zig",
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
        match expr {
            target::Expr::Empty => {
                state.imports.insert("std".into());
                "std.json.Value".into()
            },
            target::Expr::Boolean => "bool".into(),
            target::Expr::Int8 => "i8".into(),
            target::Expr::Uint8 => "u8".into(),
            target::Expr::Int16 => "i16".into(),
            target::Expr::Uint16 => "u16".into(),
            target::Expr::Int32 => "i32".into(),
            target::Expr::Uint32 => "u32".into(),
            target::Expr::Float32 => "f32".into(),
            target::Expr::Float64 => "f64".into(),
            target::Expr::String => "[]const u8".into(),
            target::Expr::Timestamp => "u64".into(),
            target::Expr::ArrayOf(sub_expr) => format!("[]{sub_expr}"),
            target::Expr::DictOf(sub_expr) => {
                _ = sub_expr;
                state.imports.insert("jsonm".into());

                format!("jsonm.Map(json.Value)")
            }
            target::Expr::NullableOf(sub_expr) => {
                if !self.with_optionals {
                    return sub_expr
                }

                format!("?{sub_expr}")
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
                    "// Code generated by jtd-codegen for Zig v{}",
                    env!("CARGO_PKG_VERSION")
                )?;
                writeln!(out)?;
                for import in &state.imports {
                    writeln!(out, "const {import} = @import(\"{import}\");")?;
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
                writeln!(out, "pub const {name} = enum {{")?;
                for member in &members {
                    writeln!(out, "    {},", member.name)?;
                }
                writeln!(out)?;
                writeln!(out, "    pub const NAMES = [@typeInfo({name}).Enum.fields.len][:0]const u8 {{")?;
                for member in &members {
                    writeln!(out, "        {:?},", member.json_value)?;
                }
                writeln!(out, "    }};")?;
                let mut i: u8 = 0;
                writeln!(out, "    pub const NAME_MAP = std.ComptimeStringMap({name}, .{{")?;
                for member in &members {
                    writeln!(out, "        .{{ NAMES[{}], .{} }},", i, member.name)?;
                    i += 1;
                }
                writeln!(out, "    }};\n")?;
                writeln!(out, "    pub fn name(self: {name}) [:0]const u8 {{")?;
                writeln!(out, "        return NAMES[@enumToInt(self)];")?;
                writeln!(out, "    }}")?;
                writeln!(out, "    pub fn fromNumber(i: u8) ?{name} {{")?;
                writeln!(out, "        return if (i < NAMES.len) @intToEnum({name}, i) else null;")?;
                writeln!(out, "    }}")?;
                writeln!(out, "    pub fn fromName(s: []const u8]) ?{name} {{")?;
                writeln!(out, "        return NAME_MAP.get(s);")?;
                writeln!(out, "    }}")?;
                writeln!(out, "}};")?;

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
                writeln!(out, "pub const {} = struct {{", name)?;
                for field in &fields {
                    writeln!(out, "    {}: {},", field.name, field.type_,)?;
                }
                writeln!(out, "}};")?;

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
        jtd_codegen_test::std_test_cases!(&crate::Target::new("jtd_codegen_e2e".into(), false));
    }

    mod optional_std_tests {
        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into(), false),
            empty_and_nonascii_properties
        );

        jtd_codegen_test::strict_std_test_case!(
            &crate::Target::new("jtd_codegen_e2e".into(), false),
            empty_and_nonascii_enum_values
        );
    }
}
