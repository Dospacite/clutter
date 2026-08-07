#[derive(Clone, Debug)]
pub struct ScannedString {
    pub offset: usize,
    pub value: String,
}

const MIN_STRING_LENGTH: usize = 4;
const MAX_STRING_LENGTH: usize = 16 * 1024;

pub fn scan(bytes: &[u8]) -> Vec<ScannedString> {
    let mut output = Vec::new();
    let mut start = 0usize;
    while start < bytes.len() {
        while start < bytes.len() && !is_printable(bytes[start]) {
            start += 1;
        }
        let mut end = start;
        while end < bytes.len()
            && is_printable(bytes[end])
            && end.saturating_sub(start) < MAX_STRING_LENGTH
        {
            end += 1;
        }
        if end.saturating_sub(start) >= MIN_STRING_LENGTH {
            output.push(ScannedString {
                offset: start,
                value: String::from_utf8_lossy(&bytes[start..end]).into_owned(),
            });
        }
        start = end.saturating_add(1);
    }
    output
}

pub fn library_uri(value: &str) -> Option<String> {
    let start = ["package:", "dart:"]
        .into_iter()
        .filter_map(|prefix| value.find(prefix))
        .min()?;
    let suffix = &value[start..];
    let dart_end = suffix.find(".dart")? + ".dart".len();
    let candidate = &suffix[..dart_end];
    candidate
        .chars()
        .all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ':' | '/' | '_' | '-' | '.')
        })
        .then(|| candidate.to_owned())
}

pub fn is_identifier(value: &str) -> bool {
    let value = value
        .split_once('@')
        .map_or(value, |(prefix, _)| prefix)
        .trim_matches(|character: char| matches!(character, '"' | '\'' | '<' | '>'));
    if !(2..=128).contains(&value.len()) {
        return false;
    }
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && characters.all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '_' | '$' | '.' | ':' | '+' | '-' | '*' | '/')
        })
}

pub fn is_human_string(value: &str) -> bool {
    if !(4..=1024).contains(&value.len())
        || value.starts_with("package:")
        || value.starts_with("dart:")
        || value.contains('\0')
    {
        return false;
    }
    let alphabetic = value
        .bytes()
        .filter(|value| value.is_ascii_alphabetic())
        .count();
    let spaces = value.bytes().filter(|value| *value == b' ').count();
    let looks_like_url = value.starts_with("http://") || value.starts_with("https://");
    looks_like_url || (alphabetic >= 3 && (spaces > 0 || value.len() <= 80))
}

fn is_printable(value: u8) -> bool {
    value == b'\t' || (0x20..=0x7e).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::{library_uri, scan};

    #[test]
    fn scans_ascii_runs_with_offsets() {
        let values = scan(b"\0hello world\0xx\0package:app/main.dart\0");
        assert_eq!(values[0].offset, 1);
        assert_eq!(values[0].value, "hello world");
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn trims_suffix_after_dart_uri() {
        assert_eq!(
            library_uri("junk package:app/main.dartZ").as_deref(),
            Some("package:app/main.dart")
        );
        assert_eq!(
            library_uri("dart:core/duration.dart").as_deref(),
            Some("dart:core/duration.dart")
        );
    }
}
