//! A serde Deserializer for single G-code lines.
//!
//! ```text
//! G1 X10 Y20 Z30 E0.5 F3000
//! G1 X10 ; move to x=10
//! ```
//!
//! Two ways to consume a line:
//!
//! 1. Deserialize directly into a struct whose *name* matches the command
//!    word (case-insensitively), e.g. `from_str::<G1>("G1 X10 Y20")`.
//! 2. Deserialize into an enum whose variant names are the command words,
//!    e.g. `from_str::<Command>("G1 X10 Y20")` where
//!    `enum Command { G1(Move), G28(Home), .. }`.

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, VariantAccess, Visitor};
use std::fmt;

// ============================================================================
// Error
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The line was empty / only whitespace. Not a real error — callers
    /// typically want to just skip these.
    BlankLine,
    /// The line was entirely a comment (nothing but whitespace before `;`).
    /// Also typically something callers skip rather than treat as failure.
    CommentOnly,
    /// The command word (`G1`, `M104`, ...) didn't match any known
    /// variant/struct. Carries the offending word.
    UnknownCommand(String),
    /// A parameter letter appeared that the target type doesn't know about
    /// (only surfaces if the target uses `#[serde(deny_unknown_fields)]`).
    UnknownField(String),
    /// A token in the parameter list wasn't `<letter><value>`.
    MalformedToken(String),
    /// A parameter's value couldn't be parsed as the requested type.
    InvalidValue {
        value: String,
        expected: &'static str,
    },
    /// Ran out of tokens where a value was expected.
    Eof,
    /// Catch-all for messages from serde derive machinery.
    Message(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BlankLine => write!(f, "line is blank"),
            Error::CommentOnly => write!(f, "line is only a comment"),
            Error::UnknownCommand(cmd) => write!(f, "unknown command `{cmd}`"),
            Error::UnknownField(field) => write!(f, "unknown field `{field}`"),
            Error::MalformedToken(tok) => write!(f, "malformed parameter `{tok}`"),
            Error::InvalidValue { value, expected } => {
                write!(f, "invalid value `{value}`, expected {expected}")
            }
            Error::Eof => write!(f, "unexpected end of parameters"),
            Error::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }

    // This is what makes an unrecognized command word surface as our own
    // variant: when deserializing an enum, serde's derive code matches the
    // command word against the known variant names and calls
    // `Error::unknown_variant` if nothing matches.
    fn unknown_variant(variant: &str, _expected: &'static [&'static str]) -> Self {
        Error::UnknownCommand(variant.to_string())
    }

    fn unknown_field(field: &str, _expected: &'static [&'static str]) -> Self {
        Error::UnknownField(field.to_string())
    }
}

// ============================================================================
// Line parsing
// ============================================================================

struct ParsedLine<'a> {
    command: &'a str,
    fields: Vec<(char, &'a str)>,
}

impl<'a> ParsedLine<'a> {
    fn parse(line: &'a str) -> Result<Self, Error> {
        let had_comment_marker = line.contains(';');
        let code_part = match line.find(';') {
            Some(idx) => &line[..idx],
            None => line,
        };

        let trimmed = code_part.trim();
        if trimmed.is_empty() {
            return if had_comment_marker {
                Err(Error::CommentOnly)
            } else {
                Err(Error::BlankLine)
            };
        }

        let mut tokens = trimmed.split_whitespace();
        let command = tokens.next().ok_or(Error::BlankLine)?;

        let mut fields = Vec::new();
        for tok in tokens {
            let mut chars = tok.chars();
            let letter = chars
                .next()
                .ok_or_else(|| Error::MalformedToken(tok.to_string()))?;
            if !letter.is_ascii_alphabetic() {
                return Err(Error::MalformedToken(tok.to_string()));
            }
            let value = &tok[letter.len_utf8()..];
            fields.push((letter.to_ascii_uppercase(), value));
        }

        Ok(ParsedLine { command, fields })
    }
}

// ============================================================================
// Entry point
// ============================================================================

pub fn from_str<'de, T>(line: &'de str) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let parsed = ParsedLine::parse(line)?;
    T::deserialize(LineDeserializer {
        command: parsed.command,
        fields: parsed.fields,
    })
}

// ============================================================================
// Top-level Deserializer: dispatches on the command word
// ============================================================================

struct LineDeserializer<'de> {
    command: &'de str,
    fields: Vec<(char, &'de str)>,
}

impl<'de> de::Deserializer<'de> for LineDeserializer<'de> {
    type Error = Error;

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(CommandEnumAccess {
            command: self.command,
            fields: self.fields,
        })
    }

    /// Deserializing straight into a struct: the struct's *name* must match
    /// the command word (case-insensitively). `struct G1 { .. }` matches a
    /// line starting with `G1`.
    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if !self.command.eq_ignore_ascii_case(name) {
            return Err(Error::UnknownCommand(self.command.to_string()));
        }
        visitor.visit_map(FieldsAccess::new(self.fields))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FieldsAccess::new(self.fields))
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct identifier ignored_any
    }
}

// ---- enum dispatch machinery ----

struct CommandEnumAccess<'de> {
    command: &'de str,
    fields: Vec<(char, &'de str)>,
}

impl<'de> EnumAccess<'de> for CommandEnumAccess<'de> {
    type Error = Error;
    type Variant = CommandVariantAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let CommandEnumAccess { command, fields } = self;
        let value = seed.deserialize(CommandDeserializer { command })?;
        Ok((value, CommandVariantAccess { fields }))
    }
}

struct CommandVariantAccess<'de> {
    fields: Vec<(char, &'de str)>,
}

impl<'de> VariantAccess<'de> for CommandVariantAccess<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(()) // e.g. `enum Command { G28 }` for a line with no parameters
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(FieldsDeserializer {
            fields: self.fields,
        })
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("tuple variants aren't supported".into()))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // `enum Command { G1 { x: Option<f64>, .. } }` also works directly.
        visitor.visit_map(FieldsAccess::new(self.fields))
    }
}

/// Feeds the command word to whatever seed is matching it against variant
/// names. `unknown_variant` (see the Error impl above) is what turns a
/// mismatch into `Error::UnknownCommand`.
struct CommandDeserializer<'de> {
    command: &'de str,
}

impl<'de> de::Deserializer<'de> for CommandDeserializer<'de> {
    type Error = Error;

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.command)
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_identifier(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum ignored_any
    }
}

// ============================================================================
// Deserializer over the parameter list (letter -> value)
// ============================================================================

/// Used for the *inner* type of an enum variant, so it deliberately does
/// **not** re-check the command word against the type name — that check
/// already happened in `CommandDeserializer`.
struct FieldsDeserializer<'de> {
    fields: Vec<(char, &'de str)>,
}

impl<'de> de::Deserializer<'de> for FieldsDeserializer<'de> {
    type Error = Error;

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FieldsAccess::new(self.fields))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(FieldsAccess::new(self.fields))
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

struct FieldsAccess<'de> {
    iter: std::vec::IntoIter<(char, &'de str)>,
    value: Option<&'de str>,
}

impl<'de> FieldsAccess<'de> {
    fn new(fields: Vec<(char, &'de str)>) -> Self {
        FieldsAccess {
            iter: fields.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for FieldsAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((letter, value)) => {
                self.value = Some(value);
                seed.deserialize(KeyDeserializer { letter }).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or(Error::Eof)?;
        seed.deserialize(ValueDeserializer { value })
    }
}

/// Turns a parameter letter (`X`, `Y`, ...) into the lowercase field-name
/// string serde's derive expects (`x`, `y`, ...).
struct KeyDeserializer {
    letter: char,
}

impl<'de> de::Deserializer<'de> for KeyDeserializer {
    type Error = Error;

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let lower = self.letter.to_ascii_lowercase();
        let mut buf = [0u8; 4];
        visitor.visit_str(lower.encode_utf8(&mut buf))
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_identifier(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum ignored_any
    }
}

// ============================================================================
// Deserializer over a single parameter's value (a &str slice of the line)
// ============================================================================

struct ValueDeserializer<'de> {
    value: &'de str,
}

/// Generates `deserialize_i8`..`deserialize_f64` in one shot: parse the
/// token with `str::parse`, turning a `ParseIntError`/`ParseFloatError`
/// into `Error::InvalidValue`, then hand it to the matching `visit_*`.
macro_rules! impl_deserialize_num {
    ($($deserialize_fn:ident => $visit_fn:ident : $ty:ty),* $(,)?) => {
        $(
            fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                let parsed: $ty = self.value.trim().parse().map_err(|_| Error::InvalidValue {
                    value: self.value.to_string(),
                    expected: stringify!($ty),
                })?;
                visitor.$visit_fn(parsed)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    impl_deserialize_num! {
        deserialize_i8  => visit_i8:  i8,
        deserialize_i16 => visit_i16: i16,
        deserialize_i32 => visit_i32: i32,
        deserialize_i64 => visit_i64: i64,
        deserialize_u8  => visit_u8:  u8,
        deserialize_u16 => visit_u16: u16,
        deserialize_u32 => visit_u32: u32,
        deserialize_u64 => visit_u64: u64,
        deserialize_f32 => visit_f32: f32,
        deserialize_f64 => visit_f64: f64,
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value.trim() {
            "1" | "true" | "TRUE" | "True" => visitor.visit_bool(true),
            "0" | "false" | "FALSE" | "False" => visitor.visit_bool(false),
            other => Err(Error::InvalidValue {
                value: other.to_string(),
                expected: "bool",
            }),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.value)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        if self.value.trim().is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let v = self.value.trim();
        if let Ok(i) = v.parse::<i64>() {
            visitor.visit_i64(i)
        } else if let Ok(f) = v.parse::<f64>() {
            visitor.visit_f64(f)
        } else {
            visitor.visit_borrowed_str(self.value)
        }
    }

    serde::forward_to_deserialize_any! {
        i128 u128 char bytes byte_buf unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
