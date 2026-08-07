use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::model::{ObfuscationMapInfo, RecoveredProgram};

const MAX_MAP_BYTES: u64 = 256 * 1024 * 1024;

pub struct LoadedObfuscationMap {
    info: ObfuscationMapInfo,
    original_by_obfuscated: BTreeMap<String, String>,
}

impl LoadedObfuscationMap {
    pub fn restore(&self, value: &str) -> String {
        restore_name(value, &self.original_by_obfuscated)
    }

    pub fn record_in(&self, program: &mut RecoveredProgram) {
        program.obfuscation_map = Some(self.info.clone());
    }
}

pub fn load(path: &Path) -> Result<LoadedObfuscationMap> {
    let metadata = std::fs::metadata(path).at(path)?;
    if metadata.len() > MAX_MAP_BYTES {
        return Err(ClutterError::InvalidArtifact(format!(
            "obfuscation map {} is {} bytes, exceeding the {} byte limit",
            path.display(),
            metadata.len(),
            MAX_MAP_BYTES,
        )));
    }
    let file = File::open(path).at(path)?;
    let flat: Vec<String> = serde_json::from_reader(file).map_err(|error| {
        ClutterError::InvalidArtifact(format!(
            "Flutter obfuscation map {} is not a string array: {error}",
            path.display()
        ))
    })?;
    if flat.len() % 2 != 0 {
        return Err(ClutterError::InvalidArtifact(format!(
            "Flutter obfuscation map {} has an odd number of entries",
            path.display()
        )));
    }

    let mut original_by_obfuscated = BTreeMap::new();
    let mut pair_count = 0usize;
    for pair in flat.chunks_exact(2) {
        let original = super::readable_snapshot_name(pair[0].trim());
        let obfuscated = pair[1].trim();
        if original.is_empty() || obfuscated.is_empty() {
            continue;
        }
        pair_count += 1;
        original_by_obfuscated
            .entry(obfuscated.to_owned())
            .or_insert_with(|| original.clone());
        original_by_obfuscated
            .entry(super::readable_snapshot_name(obfuscated))
            .or_insert(original);
    }
    if original_by_obfuscated.is_empty() {
        return Err(ClutterError::InvalidArtifact(format!(
            "Flutter obfuscation map {} contains no identifier pairs",
            path.display()
        )));
    }

    Ok(LoadedObfuscationMap {
        info: ObfuscationMapInfo {
            path: PathBuf::from(path),
            identifier_pairs: pair_count,
        },
        original_by_obfuscated,
    })
}

fn restore_name(value: &str, names: &BTreeMap<String, String>) -> String {
    if let Some(original) = names.get(value) {
        return original.clone();
    }
    let mut output = String::with_capacity(value.len());
    let mut start = None;
    for (index, character) in value.char_indices() {
        let is_identifier = character.is_ascii_alphanumeric() || matches!(character, '_' | '$');
        match (start, is_identifier) {
            (None, true) => start = Some(index),
            (Some(token_start), false) => {
                restore_token(&mut output, &value[token_start..index], names);
                output.push(character);
                start = None;
            }
            (None, false) => output.push(character),
            (Some(_), true) => {}
        }
    }
    if let Some(token_start) = start {
        restore_token(&mut output, &value[token_start..], names);
    }
    output
}

fn restore_token(output: &mut String, token: &str, names: &BTreeMap<String, String>) {
    output.push_str(names.get(token).map_or(token, String::as_str));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::restore_name;

    #[test]
    fn restores_exact_and_nested_obfuscated_tokens() {
        let names = BTreeMap::from([
            ("a".to_owned(), "Controller".to_owned()),
            ("b".to_owned(), "build".to_owned()),
        ]);
        assert_eq!(restore_name("a", &names), "Controller");
        assert_eq!(
            restore_name("a.b.<anonymous closure>", &names),
            "Controller.build.<anonymous closure>"
        );
    }
}
