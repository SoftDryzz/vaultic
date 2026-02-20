use crate::core::errors::{Result, VaulticError};
use crate::core::models::secret_file::{Line, SecretEntry, SecretFile};
use crate::core::traits::parser::ConfigParser;
use std::path::PathBuf;

/// Parses and serializes `.env` files.
///
/// v1.0 supports:
/// - `KEY=value` entries
/// - Quoted values (`KEY="value"` and `KEY='value'`)
/// - Comment lines (`# ...`)
/// - Blank lines
/// - Preserves original ordering for round-trip fidelity
pub struct DotenvParser;

impl DotenvParser {
    /// Parse a single line into a `Line` variant.
    fn parse_line(raw: &str, line_number: usize) -> Result<Line> {
        let trimmed = raw.trim();

        // Blank line
        if trimmed.is_empty() {
            return Ok(Line::Blank);
        }

        // Comment line
        if trimmed.starts_with('#') {
            return Ok(Line::Comment(raw.to_string()));
        }

        // Key=Value line — find the first '='
        let Some(eq_pos) = trimmed.find('=') else {
            return Err(VaulticError::ParseError {
                file: PathBuf::from(".env"),
                detail: format!("line {line_number}: expected KEY=value, got: {trimmed}"),
            });
        };

        let key = trimmed[..eq_pos].trim().to_string();
        if key.is_empty() {
            return Err(VaulticError::ParseError {
                file: PathBuf::from(".env"),
                detail: format!("line {line_number}: empty key"),
            });
        }

        let raw_value = trimmed[eq_pos + 1..].trim();
        let value = strip_quotes(raw_value);

        Ok(Line::Entry(SecretEntry {
            key,
            value,
            comment: None,
            line_number,
        }))
    }
}

/// Remove matching surrounding quotes (single or double) from a value.
fn strip_quotes(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

impl ConfigParser for DotenvParser {
    fn parse(&self, content: &str) -> Result<SecretFile> {
        let mut lines = Vec::new();

        for (idx, raw) in content.lines().enumerate() {
            let line_number = idx + 1;
            lines.push(DotenvParser::parse_line(raw, line_number)?);
        }

        Ok(SecretFile {
            lines,
            source_path: None,
        })
    }

    fn serialize(&self, secrets: &SecretFile) -> Result<String> {
        let mut output = String::new();

        for (i, line) in secrets.lines.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            match line {
                Line::Entry(entry) => {
                    output.push_str(&entry.key);
                    output.push('=');
                    output.push_str(&entry.value);
                }
                Line::Comment(text) => {
                    output.push_str(text);
                }
                Line::Blank => {}
            }
        }

        Ok(output)
    }

    fn supported_extensions(&self) -> &[&str] {
        &[".env"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_entries() {
        let parser = DotenvParser;
        let content = "DB_HOST=localhost\nDB_PORT=5432";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.keys(), vec!["DB_HOST", "DB_PORT"]);
        assert_eq!(file.get("DB_HOST"), Some("localhost"));
        assert_eq!(file.get("DB_PORT"), Some("5432"));
    }

    #[test]
    fn parse_double_quoted_value() {
        let parser = DotenvParser;
        let content = "SECRET=\"my secret value\"";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.get("SECRET"), Some("my secret value"));
    }

    #[test]
    fn parse_single_quoted_value() {
        let parser = DotenvParser;
        let content = "TOKEN='abc123'";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.get("TOKEN"), Some("abc123"));
    }

    #[test]
    fn parse_empty_value() {
        let parser = DotenvParser;
        let content = "EMPTY_VAR=";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.get("EMPTY_VAR"), Some(""));
    }

    #[test]
    fn parse_comments_and_blanks() {
        let parser = DotenvParser;
        let content = "# Database config\nDB_HOST=localhost\n\n# API\nAPI_KEY=secret";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.lines.len(), 5);
        assert!(matches!(file.lines[0], Line::Comment(_)));
        assert!(matches!(file.lines[1], Line::Entry(_)));
        assert!(matches!(file.lines[2], Line::Blank));
        assert!(matches!(file.lines[3], Line::Comment(_)));
        assert!(matches!(file.lines[4], Line::Entry(_)));
    }

    #[test]
    fn parse_preserves_comment_text() {
        let parser = DotenvParser;
        let content = "# This is important";
        let file = parser.parse(content).unwrap();

        assert_eq!(
            file.lines[0],
            Line::Comment("# This is important".to_string())
        );
    }

    #[test]
    fn parse_value_with_equals() {
        let parser = DotenvParser;
        let content = "DATABASE_URL=postgres://user:pass@host/db?opt=val";
        let file = parser.parse(content).unwrap();

        assert_eq!(
            file.get("DATABASE_URL"),
            Some("postgres://user:pass@host/db?opt=val")
        );
    }

    #[test]
    fn parse_invalid_line_fails() {
        let parser = DotenvParser;
        let content = "THIS_IS_NOT_VALID";
        let result = parser.parse(content);

        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_key_fails() {
        let parser = DotenvParser;
        let content = "=value";
        let result = parser.parse(content);

        assert!(result.is_err());
    }

    #[test]
    fn round_trip_preserves_content() {
        let parser = DotenvParser;
        let original = "# Database\nDB_HOST=localhost\nDB_PORT=5432\n\n# API\nAPI_KEY=secret";
        let file = parser.parse(original).unwrap();
        let serialized = parser.serialize(&file).unwrap();

        assert_eq!(serialized, original);
    }

    #[test]
    fn serialize_entries_only() {
        let parser = DotenvParser;
        let file = SecretFile {
            lines: vec![
                Line::Entry(SecretEntry {
                    key: "A".to_string(),
                    value: "1".to_string(),
                    comment: None,
                    line_number: 1,
                }),
                Line::Entry(SecretEntry {
                    key: "B".to_string(),
                    value: "2".to_string(),
                    comment: None,
                    line_number: 2,
                }),
            ],
            source_path: None,
        };

        assert_eq!(parser.serialize(&file).unwrap(), "A=1\nB=2");
    }

    #[test]
    fn supported_extensions() {
        let parser = DotenvParser;
        assert_eq!(parser.supported_extensions(), &[".env"]);
    }

    #[test]
    fn parse_spaces_around_key_and_value() {
        let parser = DotenvParser;
        let content = "  KEY  =  value  ";
        let file = parser.parse(content).unwrap();

        assert_eq!(file.get("KEY"), Some("value"));
    }
}
