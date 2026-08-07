mod aab_manifest;
mod android_manifest;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::model::{
    Abi, AndroidMetadata, ArchiveInfo, ArtifactFormat, DeferredPayload, PayloadPaths,
};

const MAX_LIBRARY_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 32 * 1024 * 1024;

pub struct Artifact {
    path: PathBuf,
    info: ArchiveInfo,
}

impl Artifact {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let input_size = std::fs::metadata(&path).at(&path)?.len();
        let input_sha256 = hash_file(&path)?;
        let file = File::open(&path).at(&path)?;
        let mut zip = ZipArchive::new(file)?;
        let all_names: BTreeSet<String> = zip.file_names().map(str::to_owned).collect();

        let format = if all_names.contains("AndroidManifest.xml") {
            ArtifactFormat::Apk
        } else if all_names
            .iter()
            .any(|name| name.ends_with("/manifest/AndroidManifest.xml"))
        {
            ArtifactFormat::Aab
        } else {
            return Err(ClutterError::InvalidArtifact(
                "ZIP does not contain an APK or AAB Android manifest".to_owned(),
            ));
        };
        let mut modules = BTreeSet::new();
        let mut payloads = BTreeMap::new();
        let mut deferred_payloads = Vec::new();
        let mut asset_count = 0usize;
        let mut asset_uncompressed_bytes = 0u64;
        let mut manifest_path =
            (format == ArtifactFormat::Apk).then(|| "AndroidManifest.xml".to_owned());

        for index in 0..zip.len() {
            let entry = zip.by_index(index)?;
            let name = entry.name();
            validate_archive_path(name)?;

            if name == "AndroidManifest.xml" {
                manifest_path = Some(name.to_owned());
            } else if let Some(module) = name.strip_suffix("/manifest/AndroidManifest.xml") {
                modules.insert(module.to_owned());
                if module == "base" || manifest_path.is_none() {
                    manifest_path = Some(name.to_owned());
                }
            }

            let Some((module, relative)) = library_entry(format, name) else {
                continue;
            };

            let Some((abi_name, filename)) = relative.split_once('/') else {
                continue;
            };
            let Ok(abi) = abi_name.parse::<Abi>() else {
                continue;
            };
            if format == ArtifactFormat::Aab {
                modules.insert(module.to_owned());
            }
            if is_deferred_payload(filename) {
                deferred_payloads.push(DeferredPayload {
                    module: module.to_owned(),
                    abi,
                    path: name.to_owned(),
                });
                continue;
            }
            if filename != "libapp.so" {
                continue;
            }

            let libflutter = match format {
                ArtifactFormat::Apk => format!("lib/{abi_name}/libflutter.so"),
                ArtifactFormat::Aab => format!("{module}/lib/{abi_name}/libflutter.so"),
            };
            let has_flutter = all_names.contains(&libflutter);
            payloads.insert(
                format!("{module}:{abi}"),
                PayloadPaths {
                    module: module.to_owned(),
                    abi,
                    libapp: name.to_owned(),
                    libflutter: has_flutter.then_some(libflutter),
                },
            );
        }

        if payloads.is_empty() {
            return Err(ClutterError::InvalidArtifact(
                "archive does not contain a supported Flutter libapp.so".to_owned(),
            ));
        }
        if modules.is_empty() {
            modules.insert("base".to_owned());
        }

        let asset_prefixes: Vec<String> = match format {
            ArtifactFormat::Apk => vec!["assets/flutter_assets/".to_owned()],
            ArtifactFormat::Aab => modules
                .iter()
                .map(|module| format!("{module}/assets/flutter_assets/"))
                .collect(),
        };
        for index in 0..zip.len() {
            let entry = zip.by_index(index)?;
            if !entry.is_dir()
                && asset_prefixes
                    .iter()
                    .any(|prefix| entry.name().starts_with(prefix))
            {
                asset_count += 1;
                asset_uncompressed_bytes = asset_uncompressed_bytes.saturating_add(entry.size());
            }
        }

        let mut available_abis: Vec<_> = payloads.values().map(|item| item.abi).collect();
        available_abis.sort();
        available_abis.dedup();
        deferred_payloads.sort_by(|left, right| {
            left.module
                .cmp(&right.module)
                .then(left.abi.cmp(&right.abi))
                .then(left.path.cmp(&right.path))
        });

        Ok(Self {
            path: path.clone(),
            info: ArchiveInfo {
                path,
                format,
                input_size,
                input_sha256,
                modules: modules.into_iter().collect(),
                available_abis,
                payloads,
                deferred_payloads,
                asset_count,
                asset_uncompressed_bytes,
                manifest_path,
            },
        })
    }

    pub fn info(&self) -> &ArchiveInfo {
        &self.info
    }

    pub fn select_payload(&self, module: &str, requested: Option<Abi>) -> Result<PayloadPaths> {
        let available = |abi| self.info.payloads.get(&format!("{module}:{abi}")).cloned();
        if let Some(abi) = requested {
            return available(abi).ok_or_else(|| {
                ClutterError::Unsupported(format!(
                    "module {module:?} does not contain ABI {abi}; available: {}",
                    self.info
                        .available_abis
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            });
        }

        Abi::ALL
            .into_iter()
            .find_map(available)
            .ok_or_else(|| ClutterError::Unsupported(format!("module {module:?} has no payload")))
    }

    pub fn read_payload(&self, path: &str) -> Result<Vec<u8>> {
        self.read_entry_limited(path, MAX_LIBRARY_BYTES)
    }

    pub fn android_metadata(&self) -> Result<AndroidMetadata> {
        let Some(path) = self.info.manifest_path.as_deref() else {
            return Ok(AndroidMetadata::default());
        };
        let bytes = self.read_entry_limited(path, MAX_MANIFEST_BYTES)?;
        match self.info.format {
            ArtifactFormat::Apk => android_manifest::parse_binary_xml(&bytes),
            ArtifactFormat::Aab => aab_manifest::parse_proto_xml(&bytes),
        }
    }

    pub fn for_each_asset(
        &self,
        module: &str,
        mut callback: impl FnMut(&str, &mut dyn Read, u64) -> Result<()>,
    ) -> Result<()> {
        let prefix = match self.info.format {
            ArtifactFormat::Apk => "assets/flutter_assets/".to_owned(),
            ArtifactFormat::Aab => format!("{module}/assets/flutter_assets/"),
        };
        let file = File::open(&self.path).at(&self.path)?;
        let mut zip = ZipArchive::new(file)?;
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index)?;
            if entry.is_dir() || !entry.name().starts_with(&prefix) {
                continue;
            }
            validate_archive_path(entry.name())?;
            let relative = entry.name()[prefix.len()..].to_owned();
            if relative.is_empty() {
                continue;
            }
            let size = entry.size();
            callback(&relative, &mut entry, size)?;
        }
        Ok(())
    }

    fn read_entry_limited(&self, name: &str, limit: u64) -> Result<Vec<u8>> {
        let file = File::open(&self.path).at(&self.path)?;
        let mut zip = ZipArchive::new(file)?;
        let mut entry = zip.by_name(name)?;
        if entry.size() > limit {
            return Err(ClutterError::InvalidArtifact(format!(
                "archive entry {name:?} is {} bytes, exceeding the {} byte limit",
                entry.size(),
                limit
            )));
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes).at(&self.path)?;
        Ok(bytes)
    }
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).at(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let count = file.read(&mut buffer).at(path)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(hex::encode(digest.finalize()))
}

fn validate_archive_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ClutterError::InvalidArtifact(format!(
            "unsafe archive entry path {value:?}"
        )));
    }
    Ok(())
}

fn is_deferred_payload(filename: &str) -> bool {
    filename
        .strip_prefix("libapp.so-")
        .and_then(|value| value.strip_suffix(".part.so"))
        .is_some_and(|loading_unit| {
            !loading_unit.is_empty()
                && loading_unit
                    .bytes()
                    .all(|character| character.is_ascii_digit())
        })
}

fn library_entry(format: ArtifactFormat, name: &str) -> Option<(&str, &str)> {
    match format {
        ArtifactFormat::Apk => name.strip_prefix("lib/").map(|relative| ("base", relative)),
        ArtifactFormat::Aab => name.split_once("/lib/"),
    }
}

#[cfg(test)]
mod tests {
    use crate::model::ArtifactFormat;

    use super::{is_deferred_payload, library_entry, validate_archive_path};

    #[test]
    fn rejects_archive_traversal() {
        assert!(validate_archive_path("../escape").is_err());
        assert!(validate_archive_path("/absolute").is_err());
        assert!(validate_archive_path("assets/good.png").is_ok());
    }

    #[test]
    fn recognizes_flutter_deferred_loading_units() {
        assert!(is_deferred_payload("libapp.so-2.part.so"));
        assert!(is_deferred_payload("libapp.so-123.part.so"));
        assert!(!is_deferred_payload("libapp.so.part.so"));
        assert!(!is_deferred_payload("libapp.so-name.part.so"));
        assert!(!is_deferred_payload("libapp.so-2.so"));
    }

    #[test]
    fn does_not_treat_apk_asset_lib_directories_as_modules() {
        assert_eq!(
            library_entry(ArtifactFormat::Apk, "lib/arm64-v8a/libapp.so"),
            Some(("base", "arm64-v8a/libapp.so"))
        );
        assert_eq!(
            library_entry(
                ArtifactFormat::Apk,
                "assets/flutter_assets/packages/line_icons/lib/assets/font.ttf"
            ),
            None
        );
        assert_eq!(
            library_entry(ArtifactFormat::Aab, "feature/lib/x86_64/libapp.so"),
            Some(("feature", "x86_64/libapp.so"))
        );
    }
}
