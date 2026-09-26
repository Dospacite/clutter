//! Call shapes recorded by Dart `ArgumentsDescriptor` arrays.
//!
//! An arguments descriptor (`runtime/vm/dart_entry.h`) is an immutable array
//! of Smi and String elements:
//!
//! | Index | Element |
//! | ----- | ------- |
//! | 0 | type-argument vector length (0 when none is passed) |
//! | 1 | `Count()`: arguments excluding the type-argument vector |
//! | 2 | `Size()`: stack words of the arguments, excluding the type-argument vector |
//! | 3 | positional argument count |
//! | 4.. | `(name, position)` per named argument, then a null terminator |
//!
//! `Count()` already excludes the type-argument vector, so the number of
//! value arguments is `count` itself; `type_args_len` describes how many
//! type arguments the hidden vector holds, not how many value arguments to
//! discount. Positions index the value arguments (including an instance
//! call's receiver).

/// One decoded descriptor element.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DescriptorElement {
    Integer(i64),
    String(String),
    Null,
    Other,
}

/// The call shape one descriptor records.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize)]
pub struct ArgumentsShape {
    pub type_args_len: usize,
    /// Value arguments, excluding the type-argument vector.
    pub count: usize,
    /// Stack words the arguments occupy, excluding the type-argument
    /// vector. Register-passed arguments (Dart 3.4+) take no stack words.
    pub size: usize,
    pub positional: usize,
    /// Named arguments with the argument position each occupies.
    pub named: Vec<(String, usize)>,
}

impl ArgumentsShape {
    /// Validates and decodes a descriptor's elements. Every structural
    /// invariant the VM relies on must hold, so an unrelated array of small
    /// integers is not mistaken for a descriptor.
    pub fn from_elements(elements: &[DescriptorElement]) -> Option<Self> {
        let integer = |index: usize| match elements.get(index) {
            Some(DescriptorElement::Integer(value)) => usize::try_from(*value).ok(),
            _ => None,
        };
        let type_args_len = integer(0)?;
        let count = integer(1)?;
        let size = integer(2)?;
        let positional = integer(3)?;
        // `size` counts stack words only: register-passed arguments take
        // none, so it may be anything up to the full argument width.
        if positional > count {
            return None;
        }
        let named_count = count - positional;
        if elements.len() != 4 + 2 * named_count + 1 {
            return None;
        }
        if elements.last() != Some(&DescriptorElement::Null) {
            return None;
        }
        let mut named = Vec::with_capacity(named_count);
        let mut positions = std::collections::BTreeSet::new();
        for entry in 0..named_count {
            let name = match elements.get(4 + 2 * entry) {
                Some(DescriptorElement::String(name)) if !name.is_empty() => name.clone(),
                _ => return None,
            };
            let position = integer(4 + 2 * entry + 1)?;
            if position < positional || position >= count || !positions.insert(position) {
                return None;
            }
            named.push((name, position));
        }
        Some(Self {
            type_args_len,
            count,
            size,
            positional,
            named,
        })
    }

    /// `typeArgs=0, count=3, positional=2, named=["b"@2]`.
    pub fn fields(&self) -> String {
        let mut text = format!(
            "typeArgs={}, count={}, positional={}",
            self.type_args_len, self.count, self.positional
        );
        if !self.named.is_empty() {
            let named = self
                .named
                .iter()
                .map(|(name, position)| {
                    format!(
                        "{}@{position}",
                        serde_json::to_string(name).unwrap_or_else(|_| "\"?\"".to_owned())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            text.push_str(&format!(", named=[{named}]"));
        }
        text
    }

    /// Pool label of a descriptor array.
    pub fn label(&self) -> String {
        format!("argsDescriptor({})", self.fields())
    }

    /// Recovers the shape from an `argsDescriptor(...)` or
    /// `dynamicCall("selector", ...)` label.
    pub fn from_label(label: &str) -> Option<Self> {
        let fields = if let Some(rest) = label.strip_prefix("argsDescriptor(") {
            rest.strip_suffix(')')?
        } else {
            let rest = label.strip_prefix("dynamicCall(")?.strip_suffix(')')?;
            // Skip the quoted selector.
            let selector_end = serde_json::Deserializer::from_str(rest)
                .into_iter::<String>()
                .next()
                .and_then(Result::ok)
                .map(|selector| serde_json::to_string(&selector).map_or(0, |text| text.len()))?;
            rest.get(selector_end..)?.strip_prefix(", ")?
        };
        let value = |key: &str| {
            fields
                .split(", ")
                .find_map(|field| field.strip_prefix(key)?.strip_prefix('='))
                .and_then(|value| value.parse::<usize>().ok())
        };
        let type_args_len = value("typeArgs")?;
        let count = value("count")?;
        let positional = value("positional")?;
        let mut named = Vec::new();
        if let Some(start) = fields.find("named=[") {
            let list = fields[start + "named=[".len()..].strip_suffix(']')?;
            let mut rest = list;
            while !rest.is_empty() {
                let mut stream = serde_json::Deserializer::from_str(rest).into_iter::<String>();
                let name = stream.next()?.ok()?;
                let consumed = stream.byte_offset();
                let tail = rest.get(consumed..)?.strip_prefix('@')?;
                let digits = tail
                    .find(|character: char| !character.is_ascii_digit())
                    .unwrap_or(tail.len());
                named.push((name, tail[..digits].parse().ok()?));
                rest = tail[digits..].strip_prefix(',').unwrap_or(&tail[digits..]);
            }
        }
        Some(Self {
            type_args_len,
            count,
            // The label does not carry the word size; it only matters for
            // unboxed arguments and never for shape constraints.
            size: count,
            positional,
            named,
        })
    }

    pub fn named_names(&self) -> impl Iterator<Item = &str> {
        self.named.iter().map(|(name, _)| name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i64) -> DescriptorElement {
        DescriptorElement::Integer(value)
    }

    #[test]
    fn decodes_generic_calls_without_discounting_type_arguments() {
        // foo<int>(a, b): the type-argument vector is not part of Count().
        let shape = ArgumentsShape::from_elements(&[
            int(1),
            int(2),
            int(2),
            int(2),
            DescriptorElement::Null,
        ])
        .expect("descriptor");
        assert_eq!(shape.type_args_len, 1);
        assert_eq!(shape.count, 2);
        assert_eq!(shape.positional, 2);
    }

    #[test]
    fn decodes_reordered_named_arguments() {
        // receiver.m(1, z: 2, a: 3): names are sorted, positions are not.
        let shape = ArgumentsShape::from_elements(&[
            int(0),
            int(4),
            int(4),
            int(2),
            DescriptorElement::String("a".to_owned()),
            int(3),
            DescriptorElement::String("z".to_owned()),
            int(2),
            DescriptorElement::Null,
        ])
        .expect("descriptor");
        assert_eq!(shape.named, vec![("a".to_owned(), 3), ("z".to_owned(), 2)]);
        let round_trip = ArgumentsShape::from_label(&shape.label()).expect("label");
        assert_eq!(round_trip.named, shape.named);
        assert_eq!(round_trip.count, 4);
    }

    #[test]
    fn rejects_arrays_that_only_resemble_descriptors() {
        assert!(ArgumentsShape::from_elements(&[int(0), int(2), int(2), int(2)]).is_none());
        assert!(
            ArgumentsShape::from_elements(&[
                int(0),
                int(3),
                int(3),
                int(2),
                DescriptorElement::String("a".to_owned()),
                int(0),
                DescriptorElement::Null,
            ])
            .is_none(),
            "a named position inside the positional range is invalid"
        );
    }

    #[test]
    fn parses_dynamic_call_labels() {
        let shape = ArgumentsShape::from_label(
            "dynamicCall(\"a, b\", typeArgs=0, count=2, positional=1, named=[\"x\"@1])",
        )
        .expect("label");
        assert_eq!(shape.named, vec![("x".to_owned(), 1)]);
        assert_eq!(shape.positional, 1);
    }
}
