use crate::blob::{Blob, BlobError};
use asn1_rs::{Any, Class, FromDer, Tag};
use core::fmt::Write;
use thiserror::Error;

type Result<T> = std::result::Result<T, Asn1Error>;

#[derive(Error, Debug, Clone)]
pub enum Asn1Error {
    #[error("formatting error")]
    Formatting(#[from] std::fmt::Error),
    #[error("binary corrupted")]
    BlobCorrupted(#[from] BlobError),
    #[error("asn1 parse error: {0}")]
    ParseError(String),
}

pub struct Asn1Binary {
    blob: Blob,
}

impl Asn1Binary {
    pub fn new(blob: Blob) -> Self {
        Self { blob }
    }

    /// Parse the entire blob as ASN.1 DER and return a human-readable string.
    pub fn to_readable_string(&self) -> Result<String> {
        let data = self.raw_data();
        let mut output = String::new();
        let (_, any) = Any::from_der(data).map_err(|e| Asn1Error::ParseError(e.to_string()))?;
        format_any(&any, 0, &mut output)?;
        Ok(output)
    }

    pub fn header_info(&self) -> Vec<(String, String)> {
        vec![("Ident".to_string(), "ASN.1 DER encoded file".to_string())]
    }

    fn raw_data(&self) -> &[u8] {
        // Reconstruct a slice over the internal blob bytes by reading all bytes.
        // We expose this via a helper so the rest of the module doesn't reach
        // into blob internals.
        self.blob.raw_bytes()
    }
}

// ---------------------------------------------------------------------------
// Recursive formatter
// ---------------------------------------------------------------------------

fn format_any(any: &Any, depth: usize, out: &mut String) -> Result<()> {
    let indent = "  ".repeat(depth);
    let class_str = class_name(any.class());
    let tag_str = tag_name(any.class(), any.tag());
    let constructed = any.header.constructed();

    if constructed {
        writeln!(out, "{indent}[{class_str}] {tag_str} (constructed) {{",)?;
        // Try to parse the contents as a sequence of nested TLVs.
        match parse_children(any.data) {
            Ok(children) => {
                for child in &children {
                    format_any(child, depth + 1, out)?;
                }
            }
            Err(_) => {
                // Fall back to hex dump of the raw content.
                let hex = bytes_to_hex(any.data);
                writeln!(out, "{indent}  <unparseable content: {hex}>")?;
            }
        }
        writeln!(out, "{indent}}}")?;
    } else {
        let value_str = format_primitive(any.tag(), any.data);
        writeln!(out, "{indent}[{class_str}] {tag_str}: {value_str}")?;
    }

    Ok(())
}

/// Parse a byte slice as a sequence of consecutive DER TLV entries.
fn parse_children<'a>(mut data: &'a [u8]) -> std::result::Result<Vec<Any<'a>>, ()> {
    let mut children = Vec::new();
    while !data.is_empty() {
        let (rest, any) = Any::from_der(data).map_err(|_| ())?;
        children.push(any);
        data = rest;
    }
    Ok(children)
}

// ---------------------------------------------------------------------------
// Tag / class helpers
// ---------------------------------------------------------------------------

fn class_name(class: Class) -> &'static str {
    match class {
        Class::Universal => "UNIVERSAL",
        Class::Application => "APPLICATION",
        Class::ContextSpecific => "CONTEXT",
        Class::Private => "PRIVATE",
    }
}

fn tag_name(class: Class, tag: Tag) -> String {
    if class != Class::Universal {
        return format!("{}", tag.0);
    }
    match tag {
        Tag::Boolean => "BOOLEAN".to_string(),
        Tag::Integer => "INTEGER".to_string(),
        Tag::BitString => "BIT STRING".to_string(),
        Tag::OctetString => "OCTET STRING".to_string(),
        Tag::Null => "NULL".to_string(),
        Tag::Oid => "OID".to_string(),
        Tag::ObjectDescriptor => "ObjectDescriptor".to_string(),
        Tag::External => "EXTERNAL".to_string(),
        Tag::RealType => "REAL".to_string(),
        Tag::Enumerated => "ENUMERATED".to_string(),
        Tag::EmbeddedPdv => "EMBEDDED PDV".to_string(),
        Tag::Utf8String => "UTF8String".to_string(),
        Tag::RelativeOid => "RELATIVE-OID".to_string(),
        Tag::Sequence => "SEQUENCE".to_string(),
        Tag::Set => "SET".to_string(),
        Tag::NumericString => "NumericString".to_string(),
        Tag::PrintableString => "PrintableString".to_string(),
        Tag::T61String => "T61String".to_string(),
        Tag::VideotexString => "VideotexString".to_string(),
        Tag::Ia5String => "IA5String".to_string(),
        Tag::UtcTime => "UTCTime".to_string(),
        Tag::GeneralizedTime => "GeneralizedTime".to_string(),
        Tag::GraphicString => "GraphicString".to_string(),
        Tag::VisibleString => "VisibleString".to_string(),
        Tag::GeneralString => "GeneralString".to_string(),
        Tag::UniversalString => "UniversalString".to_string(),
        Tag::BmpString => "BMPString".to_string(),
        other => format!("[{}]", other.0),
    }
}

// ---------------------------------------------------------------------------
// Primitive value formatters
// ---------------------------------------------------------------------------

fn format_primitive(tag: Tag, data: &[u8]) -> String {
    match tag {
        Tag::Boolean => format_boolean(data),
        Tag::Integer => format_integer(data),
        Tag::BitString => format_bit_string(data),
        Tag::OctetString => format_octet_string(data),
        Tag::Null => "NULL".to_string(),
        Tag::Oid => format_oid(data),
        Tag::Utf8String
        | Tag::PrintableString
        | Tag::Ia5String
        | Tag::VisibleString
        | Tag::NumericString
        | Tag::T61String
        | Tag::GeneralString
        | Tag::UniversalString
        | Tag::GraphicString
        | Tag::VideotexString
        | Tag::BmpString => format_string(data),
        Tag::UtcTime | Tag::GeneralizedTime => format_string(data),
        Tag::Enumerated => format_integer(data),
        _ => bytes_to_hex(data),
    }
}

fn format_boolean(data: &[u8]) -> String {
    match data.first() {
        Some(0x00) => "FALSE".to_string(),
        Some(_) => "TRUE".to_string(),
        None => "<empty>".to_string(),
    }
}

fn format_integer(data: &[u8]) -> String {
    if data.is_empty() {
        return "<empty>".to_string();
    }
    // For small integers show decimal; otherwise show hex.
    if data.len() <= 8 {
        let mut value: i64 = if data[0] & 0x80 != 0 { -1i64 } else { 0i64 };
        for &b in data {
            value = (value << 8) | (b as i64);
        }
        format!("{value} (0x{})", bytes_to_hex(data))
    } else {
        format!("0x{}", bytes_to_hex(data))
    }
}

fn format_bit_string(data: &[u8]) -> String {
    if data.is_empty() {
        return "<empty>".to_string();
    }
    let unused_bits = data[0];
    let hex = bytes_to_hex(&data[1..]);
    format!("({unused_bits} unused bits) 0x{hex}")
}

fn format_octet_string(data: &[u8]) -> String {
    // Try to display as UTF-8; fall back to hex.
    match std::str::from_utf8(data) {
        Ok(s) if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\r') => {
            format!("\"{}\"", s)
        }
        _ => format!("0x{}", bytes_to_hex(data)),
    }
}

fn format_string(data: &[u8]) -> String {
    match std::str::from_utf8(data) {
        Ok(s) => format!("\"{}\"", s),
        Err(_) => format!("0x{}", bytes_to_hex(data)),
    }
}

/// Decode a BER/DER OID from its raw content bytes (after the tag+length).
fn format_oid(data: &[u8]) -> String {
    if data.is_empty() {
        return "<empty OID>".to_string();
    }
    let mut components: Vec<u64> = Vec::new();

    // First byte encodes two components: first = byte / 40, second = byte % 40
    let first = data[0] as u64;
    components.push(first / 40);
    components.push(first % 40);

    let mut idx = 1;
    while idx < data.len() {
        // Each subsequent component is base-128 big-endian with the high bit
        // set on all but the last byte of a multi-byte component.
        let mut value: u64 = 0;
        loop {
            if idx >= data.len() {
                break;
            }
            let b = data[idx];
            idx += 1;
            value = (value << 7) | (b & 0x7f) as u64;
            if b & 0x80 == 0 {
                break;
            }
        }
        components.push(value);
    }

    components
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}
