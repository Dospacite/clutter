use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::model::{Abi, ArchiveInfo, ArtifactFormat, PayloadPaths, SnapshotInfo};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct Sha256Hex(String);

impl Sha256Hex {
    pub(crate) fn digest(bytes: &[u8]) -> Self {
        Self(hex::encode(Sha256::digest(bytes)))
    }
}

impl<'de> Deserialize<'de> for Sha256Hex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(serde::de::Error::custom(
                "expected a 64-character hexadecimal SHA-256 digest",
            ));
        }
        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl fmt::Display for Sha256Hex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ByteIdentity {
    pub(crate) size: u64,
    pub(crate) sha256: Sha256Hex,
}

impl ByteIdentity {
    pub(crate) fn observe(bytes: &[u8]) -> Self {
        Self {
            size: bytes.len() as u64,
            sha256: Sha256Hex::digest(bytes),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SnapshotRegionIdentity {
    pub(crate) name: String,
    pub(crate) virtual_address: String,
    pub(crate) file_offset: u64,
    pub(crate) bytes: ByteIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct PointerLayout {
    pub(crate) word_size: u8,
    pub(crate) compressed_word_size: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ArtifactSubject {
    pub(crate) archive: ByteIdentity,
    pub(crate) format: ArtifactFormat,
    pub(crate) module: String,
    pub(crate) abi: Abi,
    pub(crate) payload_member: String,
    pub(crate) payload: ByteIdentity,
    pub(crate) snapshot_regions: Vec<SnapshotRegionIdentity>,
    pub(crate) snapshot_hash: String,
    pub(crate) pointer_layout: PointerLayout,
}

impl ArtifactSubject {
    pub(crate) fn observe(
        archive: &ArchiveInfo,
        payload: &PayloadPaths,
        libapp: &[u8],
        snapshot: &SnapshotInfo,
    ) -> Self {
        let compressed = snapshot
            .isolate_header
            .features
            .iter()
            .any(|feature| feature == "compressed-pointers");
        let word_size = match payload.abi {
            Abi::ArmeabiV7a => 4,
            Abi::Arm64V8a | Abi::X86_64 => 8,
        };
        Self {
            archive: ByteIdentity {
                size: archive.input_size,
                sha256: Sha256Hex(archive.input_sha256.clone()),
            },
            format: archive.format,
            module: payload.module.clone(),
            abi: payload.abi,
            payload_member: payload.libapp.clone(),
            payload: ByteIdentity::observe(libapp),
            snapshot_regions: snapshot
                .regions
                .iter()
                .map(|region| SnapshotRegionIdentity {
                    name: region.name.clone(),
                    virtual_address: region.virtual_address.clone(),
                    file_offset: region.file_offset,
                    bytes: ByteIdentity {
                        size: region.size,
                        sha256: Sha256Hex(region.sha256.clone()),
                    },
                })
                .collect(),
            snapshot_hash: snapshot.isolate_header.snapshot_hash.clone(),
            pointer_layout: PointerLayout {
                word_size,
                compressed_word_size: if compressed { 4 } else { word_size },
            },
        }
    }

    pub(crate) fn mismatches(&self, observed: &Self) -> Vec<String> {
        let mut mismatches = Vec::new();
        compare(&mut mismatches, "archive", &self.archive, &observed.archive);
        compare(&mut mismatches, "format", &self.format, &observed.format);
        compare(&mut mismatches, "module", &self.module, &observed.module);
        compare(&mut mismatches, "abi", &self.abi, &observed.abi);
        compare(
            &mut mismatches,
            "payload_member",
            &self.payload_member,
            &observed.payload_member,
        );
        compare(&mut mismatches, "payload", &self.payload, &observed.payload);
        compare(
            &mut mismatches,
            "snapshot_regions",
            &self.snapshot_regions,
            &observed.snapshot_regions,
        );
        compare(
            &mut mismatches,
            "snapshot_hash",
            &self.snapshot_hash,
            &observed.snapshot_hash,
        );
        compare(
            &mut mismatches,
            "pointer_layout",
            &self.pointer_layout,
            &observed.pointer_layout,
        );
        mismatches
    }
}

fn compare<T: fmt::Debug + PartialEq>(
    mismatches: &mut Vec<String>,
    field: &str,
    expected: &T,
    observed: &T,
) {
    if expected != observed {
        mismatches.push(format!(
            "{field}: oracle has {expected:?}, selected artifact has {observed:?}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{ArtifactSubject, ByteIdentity, PointerLayout, Sha256Hex};
    use crate::model::{Abi, ArtifactFormat};

    fn subject() -> ArtifactSubject {
        ArtifactSubject {
            archive: ByteIdentity::observe(b"archive"),
            format: ArtifactFormat::Apk,
            module: "base".to_owned(),
            abi: Abi::X86_64,
            payload_member: "lib/x86_64/libapp.so".to_owned(),
            payload: ByteIdentity::observe(b"payload"),
            snapshot_regions: Vec::new(),
            snapshot_hash: "snapshot".to_owned(),
            pointer_layout: PointerLayout {
                word_size: 8,
                compressed_word_size: 4,
            },
        }
    }

    #[test]
    fn sha256_digest_deserializes_only_exact_hex() {
        let digest = Sha256Hex::digest(b"clutter");
        let encoded = serde_json::to_string(&digest).unwrap();
        assert_eq!(serde_json::from_str::<Sha256Hex>(&encoded).unwrap(), digest);
        assert!(serde_json::from_str::<Sha256Hex>("\"abcd\"").is_err());
    }

    #[test]
    fn subject_mismatch_names_each_changed_identity_field() {
        let expected = subject();
        let mut observed = subject();
        observed.abi = Abi::Arm64V8a;
        observed.payload = ByteIdentity::observe(b"different payload");

        let mismatches = expected.mismatches(&observed);
        assert_eq!(mismatches.len(), 2);
        assert!(mismatches[0].starts_with("abi:"));
        assert!(mismatches[1].starts_with("payload:"));
    }
}
