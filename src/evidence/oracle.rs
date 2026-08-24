use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::subject::{ArtifactSubject, ByteIdentity, PointerLayout};
use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::model::VmOracleEvidence;

const BINDING_SCHEMA: &str = "clutter.vm-oracle-binding/v1";
const MAX_BINDING_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct AnalyzerIdentity {
    executable: ByteIdentity,
    analyzer_schema: u64,
    dart_version: Option<String>,
    dart_commit: String,
    target_arch: String,
    pointer_layout: PointerLayout,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct OracleBindingManifest {
    schema: String,
    subject: ArtifactSubject,
    document: ByteIdentity,
    analyzer: AnalyzerIdentity,
}

pub(crate) fn binding_path(oracle_path: &Path) -> PathBuf {
    let mut value = OsString::from(oracle_path.as_os_str());
    value.push(".binding.json");
    PathBuf::from(value)
}

pub(crate) fn write_binding(
    oracle_path: &Path,
    analyzer_path: &Path,
    subject: ArtifactSubject,
    evidence: &VmOracleEvidence,
    replace: bool,
) -> Result<PathBuf> {
    let path = binding_path(oracle_path);
    if path.exists() && !replace {
        return Err(ClutterError::OutputExists(path));
    }

    let document_bytes = fs::read(oracle_path).at(oracle_path)?;
    let analyzer_bytes = fs::read(analyzer_path).at(analyzer_path)?;
    let dart_commit = evidence.dart_commit.clone().ok_or_else(|| {
        ClutterError::Analysis(
            "Dart VM oracle does not identify the Dart commit used to build its analyzer"
                .to_owned(),
        )
    })?;
    let target_arch = evidence.target_arch.clone().ok_or_else(|| {
        ClutterError::Analysis(
            "Dart VM oracle does not identify its analyzer target architecture".to_owned(),
        )
    })?;
    let manifest = OracleBindingManifest {
        schema: BINDING_SCHEMA.to_owned(),
        subject,
        document: ByteIdentity::observe(&document_bytes),
        analyzer: AnalyzerIdentity {
            executable: ByteIdentity::observe(&analyzer_bytes),
            analyzer_schema: evidence.analyzer_version,
            dart_version: evidence.dart_version.clone(),
            dart_commit,
            target_arch,
            pointer_layout: PointerLayout {
                word_size: u8::try_from(evidence.word_size).map_err(|_| {
                    ClutterError::Analysis(format!(
                        "oracle word size {} cannot be represented in its binding",
                        evidence.word_size
                    ))
                })?,
                compressed_word_size: u8::try_from(evidence.compressed_word_size).map_err(
                    |_| {
                        ClutterError::Analysis(format!(
                            "oracle compressed word size {} cannot be represented in its binding",
                            evidence.compressed_word_size
                        ))
                    },
                )?,
            },
        },
    };

    let parent = path.parent().filter(|value| !value.as_os_str().is_empty());
    if let Some(parent) = parent {
        fs::create_dir_all(parent).at(parent)?;
    }
    let temporary_parent = parent.unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(temporary_parent).at(temporary_parent)?;
    serde_json::to_writer_pretty(&mut temporary, &manifest)?;
    temporary.write_all(b"\n").at(&path)?;
    temporary.flush().at(&path)?;
    temporary.persist(&path).map_err(|error| ClutterError::Io {
        path: path.clone(),
        source: error.error,
    })?;
    Ok(path)
}

pub(crate) fn verify_binding(
    oracle_path: &Path,
    observed: &ArtifactSubject,
    evidence: &VmOracleEvidence,
) -> Result<()> {
    let path = binding_path(oracle_path);
    let metadata = fs::metadata(&path).at(&path)?;
    if metadata.len() > MAX_BINDING_BYTES {
        return Err(ClutterError::InvalidArtifact(format!(
            "Dart VM oracle binding is {} bytes, exceeding the {} byte limit",
            metadata.len(),
            MAX_BINDING_BYTES
        )));
    }
    let manifest: OracleBindingManifest = serde_json::from_slice(&fs::read(&path).at(&path)?)?;
    if manifest.schema != BINDING_SCHEMA {
        return Err(ClutterError::Unsupported(format!(
            "Dart VM oracle binding schema {:?} is unsupported",
            manifest.schema
        )));
    }

    let mismatches = manifest.subject.mismatches(observed);
    if !mismatches.is_empty() {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle subject differs from selected artifact: {}",
            mismatches.join("; ")
        )));
    }

    let document_bytes = fs::read(oracle_path).at(oracle_path)?;
    let document = ByteIdentity::observe(&document_bytes);
    if manifest.document != document {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle document identity differs from its binding: expected {:?}, observed {:?}",
            manifest.document, document
        )));
    }

    let mut metadata_mismatches = Vec::new();
    compare(
        &mut metadata_mismatches,
        "analyzer_schema",
        &manifest.analyzer.analyzer_schema,
        &evidence.analyzer_version,
    );
    compare(
        &mut metadata_mismatches,
        "dart_version",
        &manifest.analyzer.dart_version,
        &evidence.dart_version,
    );
    compare(
        &mut metadata_mismatches,
        "dart_commit",
        &Some(manifest.analyzer.dart_commit.clone()),
        &evidence.dart_commit,
    );
    compare(
        &mut metadata_mismatches,
        "target_arch",
        &Some(manifest.analyzer.target_arch.clone()),
        &evidence.target_arch,
    );
    compare(
        &mut metadata_mismatches,
        "word_size",
        &u64::from(manifest.analyzer.pointer_layout.word_size),
        &evidence.word_size,
    );
    compare(
        &mut metadata_mismatches,
        "compressed_word_size",
        &u64::from(manifest.analyzer.pointer_layout.compressed_word_size),
        &evidence.compressed_word_size,
    );
    if !metadata_mismatches.is_empty() {
        return Err(ClutterError::Analysis(format!(
            "Dart VM oracle metadata differs from its binding: {}",
            metadata_mismatches.join("; ")
        )));
    }
    Ok(())
}

fn compare<T: std::fmt::Debug + PartialEq>(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: &T,
    observed: &T,
) {
    if expected != observed {
        mismatches.push(format!(
            "{field}: binding has {expected:?}, document has {observed:?}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::binding_path;

    #[test]
    fn binding_path_does_not_replace_the_oracle_extension() {
        assert_eq!(
            binding_path(Path::new("evidence/oracle.json")),
            Path::new("evidence/oracle.json.binding.json")
        );
    }
}
