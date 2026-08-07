use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use gimli::{AttributeValue, EndianSlice, LittleEndian, SectionId};
use object::{Architecture, Object, ObjectSection, ObjectSymbol, SymbolKind};

use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::model::{Abi, RecoveredInlineFunction, RecoveredSourceLocation};

#[derive(Clone, Debug)]
pub struct DebugFunction {
    pub address: u64,
    pub size: u64,
    pub name: String,
    pub library_uri: Option<String>,
    pub source_location: Option<RecoveredSourceLocation>,
}

#[derive(Clone, Debug)]
pub struct DebugDeclaration {
    pub name: String,
    pub library_uri: Option<String>,
    pub source_location: RecoveredSourceLocation,
    pub has_code: bool,
}

#[derive(Debug)]
pub struct DebugSymbols {
    pub functions: Vec<DebugFunction>,
    pub declarations: Vec<DebugDeclaration>,
    pub inlined_functions: Vec<RecoveredInlineFunction>,
    pub application_package: Option<String>,
    pub build_id: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
struct LineLocation {
    path: String,
    line: Option<u64>,
    column: Option<u64>,
}

#[derive(Clone, Debug)]
struct RawInlineFunction {
    address: u64,
    size: u64,
    name: String,
    source_location: Option<LineLocation>,
    call_location: Option<LineLocation>,
}

pub fn load(
    path: &Path,
    libapp: &[u8],
    expected_abi: Abi,
    application_package: Option<&str>,
) -> Result<DebugSymbols> {
    let app_file = object::File::parse(libapp)?;
    let app_build_id = build_id(&app_file)?
        .ok_or_else(|| ClutterError::InvalidArtifact("libapp.so has no GNU build ID".to_owned()))?;
    let (resolved_path, bytes) = resolve_symbol_file(path, app_build_id, expected_abi)?;
    let debug_file = object::File::parse(bytes.as_slice()).map_err(|error| {
        ClutterError::InvalidArtifact(format!(
            "split debug info {} is not a readable ELF: {error}",
            resolved_path.display()
        ))
    })?;
    validate_elf(&debug_file, expected_abi, &resolved_path)?;

    let debug_build_id = build_id(&debug_file)?.ok_or_else(|| {
        ClutterError::InvalidArtifact(format!(
            "split debug info {} has no GNU build ID",
            resolved_path.display()
        ))
    })?;
    if debug_build_id != app_build_id {
        return Err(ClutterError::InvalidArtifact(format!(
            "split debug info build ID {} does not match libapp.so build ID {}",
            hex::encode(debug_build_id),
            hex::encode(app_build_id)
        )));
    }

    let dwarf = load_dwarf(&debug_file)?;
    let lines = read_line_locations(&dwarf)?;
    let (declarations, raw_inlined_functions) = read_dwarf_entries(&dwarf)?;
    let application_lib_root = debug_file
        .symbols()
        .find(|symbol| symbol.name().ok() == Some("main"))
        .and_then(|symbol| lines.get(&symbol.address()))
        .or_else(|| declarations.get("main").and_then(|values| values.first()))
        .and_then(|location| dart_lib_root(&location.path));
    let inferred_application_package = application_package
        .is_none()
        .then(|| {
            application_lib_root
                .as_deref()
                .and_then(package_name_from_lib_root)
        })
        .flatten();
    let effective_application_package =
        application_package.or(inferred_application_package.as_deref());
    let mut functions = debug_file
        .symbols()
        .filter(|symbol| {
            symbol.is_definition()
                && symbol.kind() == SymbolKind::Text
                && symbol.address() != 0
                && symbol.size() != 0
        })
        .filter_map(|symbol| {
            let name = symbol.name().ok()?.trim();
            if name.is_empty() || name.starts_with("_kDart") {
                return None;
            }
            let mut line = lines.get(&symbol.address()).cloned();
            line = best_declaration(
                declarations.get(name),
                line.as_ref(),
                effective_application_package,
                application_lib_root.as_deref(),
            )
            .cloned()
            .or(line);
            let library_uri = line.as_ref().and_then(|location| {
                source_library_uri_with_root(
                    &location.path,
                    effective_application_package,
                    application_lib_root.as_deref(),
                )
            });
            Some(DebugFunction {
                address: symbol.address(),
                size: symbol.size(),
                name: name.to_owned(),
                library_uri,
                source_location: source_span(&lines, symbol.address(), symbol.size(), line),
            })
        })
        .collect::<Vec<_>>();
    functions.sort_by(|left, right| {
        left.address
            .cmp(&right.address)
            .then(left.name.cmp(&right.name))
    });
    functions.dedup_by(|left, right| {
        left.address == right.address && left.size == right.size && left.name == right.name
    });

    if functions.is_empty() {
        return Err(ClutterError::InvalidArtifact(format!(
            "split debug info {} contains no sized text symbols",
            resolved_path.display()
        )));
    }
    let mut inlined_functions = raw_inlined_functions
        .into_iter()
        .map(|inline| RecoveredInlineFunction {
            name: inline.name,
            library_uri: inline.source_location.as_ref().and_then(|location| {
                source_library_uri_with_root(
                    &location.path,
                    effective_application_package,
                    application_lib_root.as_deref(),
                )
            }),
            source_location: inline.source_location.map(recovered_location),
            call_location: inline.call_location.map(recovered_location),
            address: format!("0x{:x}", inline.address),
            size: inline.size,
        })
        .collect::<Vec<_>>();
    inlined_functions.sort_by(|left, right| {
        parse_address(&left.address)
            .cmp(&parse_address(&right.address))
            .then(left.name.cmp(&right.name))
    });
    let mut debug_declarations = declarations
        .into_iter()
        .flat_map(|(name, locations)| {
            locations.into_iter().filter_map({
                let functions = &functions;
                let application_lib_root = application_lib_root.as_deref();
                move |location| {
                    let library_uri = source_library_uri_with_root(
                        &location.path,
                        effective_application_package,
                        application_lib_root,
                    )?;
                    let has_code = functions.iter().any(|function| {
                        function.name == name
                            && function.library_uri.as_deref() == Some(&library_uri)
                    });
                    Some(DebugDeclaration {
                        name: name.clone(),
                        library_uri: Some(library_uri),
                        source_location: recovered_location(location),
                        has_code,
                    })
                }
            })
        })
        .collect::<Vec<_>>();
    debug_declarations.sort_by(|left, right| {
        left.library_uri
            .cmp(&right.library_uri)
            .then(left.name.cmp(&right.name))
            .then(left.source_location.line.cmp(&right.source_location.line))
    });
    debug_declarations.dedup_by(|left, right| {
        left.library_uri == right.library_uri
            && left.name == right.name
            && left.source_location.line == right.source_location.line
    });
    Ok(DebugSymbols {
        functions,
        declarations: debug_declarations,
        inlined_functions,
        application_package: effective_application_package.map(str::to_owned),
        build_id: hex::encode(debug_build_id),
        path: resolved_path,
    })
}

fn resolve_symbol_file(
    path: &Path,
    expected_build_id: &[u8],
    expected_abi: Abi,
) -> Result<(PathBuf, Vec<u8>)> {
    if !path.is_dir() {
        return Ok((path.to_path_buf(), fs::read(path).at(path)?));
    }
    let mut candidates = fs::read_dir(path)
        .at(path)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|candidate| candidate.is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    for candidate in candidates {
        let Ok(bytes) = fs::read(&candidate) else {
            continue;
        };
        let Ok(file) = object::File::parse(bytes.as_slice()) else {
            continue;
        };
        if architecture_abi(file.architecture()) != Some(expected_abi) {
            continue;
        }
        if file.build_id().ok().flatten() == Some(expected_build_id) {
            return Ok((candidate, bytes));
        }
    }
    Err(ClutterError::InvalidArtifact(format!(
        "directory {} contains no {expected_abi} split debug ELF matching libapp.so build ID {}",
        path.display(),
        hex::encode(expected_build_id)
    )))
}

fn validate_elf(file: &object::File<'_>, expected_abi: Abi, path: &Path) -> Result<()> {
    if file.format() != object::BinaryFormat::Elf {
        return Err(ClutterError::InvalidArtifact(format!(
            "split debug info {} is not an ELF file",
            path.display()
        )));
    }
    let architecture = file.architecture();
    let actual = match architecture_abi(architecture) {
        Some(abi) => abi,
        None => {
            return Err(ClutterError::Unsupported(format!(
                "split debug info uses unsupported architecture {architecture:?}"
            )));
        }
    };
    if actual != expected_abi {
        return Err(ClutterError::InvalidArtifact(format!(
            "split debug info is for {actual}, but selected artifact ABI is {expected_abi}"
        )));
    }
    Ok(())
}

fn architecture_abi(architecture: Architecture) -> Option<Abi> {
    match architecture {
        Architecture::Aarch64 => Some(Abi::Arm64V8a),
        Architecture::Arm => Some(Abi::ArmeabiV7a),
        Architecture::X86_64 => Some(Abi::X86_64),
        _ => None,
    }
}

fn build_id<'a>(file: &'a object::File<'a>) -> Result<Option<&'a [u8]>> {
    file.build_id().map_err(ClutterError::Object)
}

fn read_line_locations(
    dwarf: &gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
) -> Result<BTreeMap<u64, LineLocation>> {
    let mut locations = BTreeMap::new();
    let mut units = dwarf.units();
    while let Some(header) = units.next().map_err(gimli_error)? {
        let unit = dwarf.unit(header).map_err(gimli_error)?;
        let Some(program) = unit.line_program.clone() else {
            continue;
        };
        let mut rows = program.rows();
        while let Some((header, row)) = rows.next_row().map_err(gimli_error)? {
            if row.end_sequence() {
                continue;
            }
            let Some(file_entry) = row.file(header) else {
                continue;
            };
            let Some(path) = resolve_file_path(dwarf, &unit, header, file_entry) else {
                continue;
            };
            locations.entry(row.address()).or_insert(LineLocation {
                path,
                line: row.line().map(|line| line.get()),
                column: match row.column() {
                    gimli::ColumnType::Column(column) => Some(column.get()),
                    gimli::ColumnType::LeftEdge => None,
                },
            });
        }
    }
    Ok(locations)
}

type DwarfEntries = (BTreeMap<String, Vec<LineLocation>>, Vec<RawInlineFunction>);

fn read_dwarf_entries(dwarf: &gimli::Dwarf<EndianSlice<'_, LittleEndian>>) -> Result<DwarfEntries> {
    let mut declarations = BTreeMap::<String, Vec<LineLocation>>::new();
    let mut inlined_functions = Vec::new();
    let mut units = dwarf.units();
    while let Some(header) = units.next().map_err(gimli_error)? {
        let unit = dwarf.unit(header).map_err(gimli_error)?;
        let Some(line_program) = unit.line_program.as_ref() else {
            continue;
        };
        let line_header = line_program.header();
        let mut origins = BTreeMap::<usize, (String, Option<LineLocation>)>::new();
        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs().map_err(gimli_error)? {
            if entry.tag() != gimli::DW_TAG_subprogram {
                continue;
            }
            let Some(name_value) = entry.attr_value(gimli::DW_AT_name).map_err(gimli_error)? else {
                continue;
            };
            let name = dwarf
                .attr_string(&unit, name_value)
                .map_err(gimli_error)?
                .to_string_lossy()
                .into_owned();
            let location = entry_location(
                dwarf,
                &unit,
                line_header,
                entry,
                gimli::DW_AT_decl_file,
                gimli::DW_AT_decl_line,
            )?;
            if let Some(location) = &location {
                declarations
                    .entry(name.clone())
                    .or_default()
                    .push(location.clone());
            }
            origins.insert(entry.offset().0, (name, location));
        }

        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs().map_err(gimli_error)? {
            if entry.tag() != gimli::DW_TAG_inlined_subroutine {
                continue;
            }
            let Some(AttributeValue::UnitRef(origin)) = entry
                .attr_value(gimli::DW_AT_abstract_origin)
                .map_err(gimli_error)?
            else {
                continue;
            };
            let Some((name, source_location)) = origins.get(&origin.0) else {
                continue;
            };
            let call_location = entry_location(
                dwarf,
                &unit,
                line_header,
                entry,
                gimli::DW_AT_call_file,
                gimli::DW_AT_call_line,
            )?;
            for (address, size) in entry_ranges(dwarf, &unit, entry)? {
                inlined_functions.push(RawInlineFunction {
                    address,
                    size,
                    name: name.clone(),
                    source_location: source_location.clone(),
                    call_location: call_location.clone(),
                });
            }
        }
    }
    for values in declarations.values_mut() {
        values.sort_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)));
        values.dedup_by(|left, right| left.path == right.path && left.line == right.line);
    }
    Ok((declarations, inlined_functions))
}

fn entry_location(
    dwarf: &gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &gimli::Unit<EndianSlice<'_, LittleEndian>>,
    line_header: &gimli::LineProgramHeader<EndianSlice<'_, LittleEndian>>,
    entry: &gimli::DebuggingInformationEntry<'_, '_, EndianSlice<'_, LittleEndian>>,
    file_attribute: gimli::DwAt,
    line_attribute: gimli::DwAt,
) -> Result<Option<LineLocation>> {
    let Some(AttributeValue::FileIndex(file_index)) =
        entry.attr_value(file_attribute).map_err(gimli_error)?
    else {
        return Ok(None);
    };
    let Some(file_entry) = line_header.file(file_index) else {
        return Ok(None);
    };
    let Some(path) = resolve_file_path(dwarf, unit, line_header, file_entry) else {
        return Ok(None);
    };
    let line = entry
        .attr_value(line_attribute)
        .map_err(gimli_error)?
        .and_then(|value| value.udata_value())
        .filter(|line| *line > 0);
    let column_attribute = if line_attribute == gimli::DW_AT_call_line {
        gimli::DW_AT_call_column
    } else {
        gimli::DW_AT_decl_column
    };
    let column = entry
        .attr_value(column_attribute)
        .map_err(gimli_error)?
        .and_then(|value| value.udata_value())
        .filter(|column| *column > 0);
    Ok(Some(LineLocation { path, line, column }))
}

fn entry_ranges(
    dwarf: &gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &gimli::Unit<EndianSlice<'_, LittleEndian>>,
    entry: &gimli::DebuggingInformationEntry<'_, '_, EndianSlice<'_, LittleEndian>>,
) -> Result<Vec<(u64, u64)>> {
    let mut ranges = dwarf.die_ranges(unit, entry).map_err(gimli_error)?;
    let mut values = Vec::new();
    while let Some(range) = ranges.next().map_err(gimli_error)? {
        if range.end > range.begin {
            values.push((range.begin, range.end - range.begin));
        }
    }
    Ok(values)
}

fn best_declaration<'a>(
    candidates: Option<&'a Vec<LineLocation>>,
    current: Option<&LineLocation>,
    application_package: Option<&str>,
    application_lib_root: Option<&str>,
) -> Option<&'a LineLocation> {
    let candidates = candidates?;
    if let Some(current) = current
        && let Some(candidate) = candidates
            .iter()
            .find(|candidate| same_path(&candidate.path, &current.path))
    {
        return Some(candidate);
    }
    candidates
        .iter()
        .find(|candidate| {
            source_library_uri_with_root(&candidate.path, application_package, application_lib_root)
                .is_some()
        })
        .or_else(|| candidates.first())
}

fn source_span(
    lines: &BTreeMap<u64, LineLocation>,
    address: u64,
    size: u64,
    mut primary: Option<LineLocation>,
) -> Option<RecoveredSourceLocation> {
    let end = address.saturating_add(size);
    if primary
        .as_ref()
        .is_none_or(|location| location.line.is_none())
        && let Some(location) = lines
            .range(address..end)
            .map(|(_, location)| location)
            .find(|location| location.line.is_some())
    {
        primary = Some(location.clone());
    }
    let primary = primary?;
    let mut line_values = lines
        .range(address..end)
        .map(|(_, location)| location)
        .filter(|location| same_path(&location.path, &primary.path))
        .filter_map(|location| location.line)
        .filter(|line| *line > 0)
        .collect::<Vec<_>>();
    if let Some(line) = primary.line.filter(|line| *line > 0) {
        line_values.push(line);
    }
    line_values.sort_unstable();
    let line = line_values.first().copied().or(primary.line);
    let end_line = line_values
        .last()
        .copied()
        .filter(|end_line| Some(*end_line) != line);
    Some(RecoveredSourceLocation {
        path: primary.path,
        line,
        column: primary.column,
        end_line,
        end_column: None,
    })
}

fn recovered_location(location: LineLocation) -> RecoveredSourceLocation {
    RecoveredSourceLocation {
        path: location.path,
        line: location.line,
        column: location.column,
        end_line: None,
        end_column: None,
    }
}

fn same_path(left: &str, right: &str) -> bool {
    normalize_path(left) == normalize_path(right)
}

fn normalize_path(value: &str) -> String {
    let mut normalized = value.replace('\\', "/");
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    normalized
}

fn parse_address(value: &str) -> u64 {
    u64::from_str_radix(value.trim_start_matches("0x"), 16).unwrap_or(u64::MAX)
}

fn load_dwarf<'a>(
    file: &'a object::File<'a>,
) -> Result<gimli::Dwarf<EndianSlice<'a, LittleEndian>>> {
    gimli::Dwarf::load(
        |section_id: SectionId| -> std::result::Result<_, gimli::Error> {
            let data = file
                .section_by_name(section_id.name())
                .and_then(|section| section.data().ok())
                .unwrap_or(&[]);
            Ok(EndianSlice::new(data, LittleEndian))
        },
    )
    .map_err(gimli_error)
}

fn resolve_file_path(
    dwarf: &gimli::Dwarf<EndianSlice<'_, LittleEndian>>,
    unit: &gimli::Unit<EndianSlice<'_, LittleEndian>>,
    header: &gimli::LineProgramHeader<EndianSlice<'_, LittleEndian>>,
    file_entry: &gimli::FileEntry<EndianSlice<'_, LittleEndian>>,
) -> Option<String> {
    let path = dwarf
        .attr_string(unit, file_entry.path_name())
        .ok()?
        .to_string_lossy()
        .into_owned();
    if Path::new(&path).is_absolute() {
        return Some(path);
    }
    let directory = file_entry
        .directory(header)
        .and_then(|value| dwarf.attr_string(unit, value).ok())
        .map(|reader| reader.to_string_lossy().into_owned());
    Some(
        directory
            .map(|directory| {
                PathBuf::from(directory)
                    .join(&path)
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or(path),
    )
}

fn gimli_error(error: gimli::Error) -> ClutterError {
    ClutterError::Analysis(format!("read DWARF line information: {error}"))
}

#[cfg(test)]
fn source_library_uri(path: &str, application_package: Option<&str>) -> Option<String> {
    source_library_uri_with_root(path, application_package, None)
}

fn source_library_uri_with_root(
    path: &str,
    application_package: Option<&str>,
    application_lib_root: Option<&str>,
) -> Option<String> {
    let normalized = path.replace('\\', "/");
    if let Some(package) = application_package {
        if let Some(root) = application_lib_root
            && let Some(relative) = normalized
                .strip_prefix(root)
                .and_then(|value| value.strip_prefix('/'))
            && !relative.is_empty()
        {
            return Some(format!("package:{package}/{relative}"));
        }
        if let Some(relative) = after_package_lib(&normalized, package) {
            return Some(format!("package:{package}/{relative}"));
        }
    }
    if let Some(relative) = after_marker(&normalized, "/packages/flutter/lib/") {
        return Some(format!("package:flutter/{relative}"));
    }
    if let Some(relative) = after_marker(&normalized, "/sdk/lib/") {
        let library = relative.split('/').next()?;
        return Some(format!("dart:{library}"));
    }
    if let Some(relative) = after_marker(&normalized, "/dart-sdk/lib/") {
        let library = relative.split('/').next()?;
        return Some(format!("dart:{library}"));
    }

    let segments = normalized.split('/').collect::<Vec<_>>();
    for (index, segment) in segments.iter().enumerate() {
        if *segment != "pub.dev" || index + 2 >= segments.len() {
            continue;
        }
        let package_with_version = segments[index + 1];
        if segments[index + 2] != "lib" {
            continue;
        }
        let package = package_with_version
            .rsplit_once('-')
            .map_or(package_with_version, |(package, _)| package);
        let relative = segments[index + 3..].join("/");
        if !package.is_empty() && !relative.is_empty() {
            return Some(format!("package:{package}/{relative}"));
        }
    }
    None
}

fn dart_lib_root(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let marker = "/lib/";
    let index = normalized.rfind(marker)?;
    Some(normalized[..index + marker.len() - 1].to_owned())
}

fn package_name_from_lib_root(root: &str) -> Option<String> {
    let package = Path::new(root).parent()?.file_name()?.to_str()?.to_owned();
    let mut characters = package.chars();
    let first = characters.next()?;
    (first.is_ascii_lowercase()
        && package.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        }))
    .then_some(package)
}

fn after_package_lib<'a>(path: &'a str, package: &str) -> Option<&'a str> {
    after_marker(path, &format!("/{package}/lib/"))
}

fn after_marker<'a>(path: &'a str, marker: &str) -> Option<&'a str> {
    let index = path.rfind(marker)?;
    let relative = &path[index + marker.len()..];
    (!relative.is_empty()).then_some(relative)
}

#[cfg(test)]
mod tests {
    use super::{
        dart_lib_root, package_name_from_lib_root, source_library_uri, source_library_uri_with_root,
    };

    #[test]
    fn maps_application_flutter_sdk_and_pub_cache_paths() {
        assert_eq!(
            source_library_uri("/work/my_app/lib/features/home.dart", Some("my_app")).as_deref(),
            Some("package:my_app/features/home.dart")
        );
        assert_eq!(
            source_library_uri(
                "/opt/flutter/packages/flutter/lib/src/widgets/app.dart",
                Some("my_app")
            )
            .as_deref(),
            Some("package:flutter/src/widgets/app.dart")
        );
        assert_eq!(
            source_library_uri(
                "/cache/hosted/pub.dev/collection-1.19.1/lib/src/list.dart",
                Some("my_app")
            )
            .as_deref(),
            Some("package:collection/src/list.dart")
        );
    }

    #[test]
    fn maps_an_application_whose_directory_differs_from_its_package() {
        let root = dart_lib_root("/work/mobile_client/lib/main.dart").unwrap();
        assert_eq!(root, "/work/mobile_client/lib");
        assert_eq!(
            source_library_uri_with_root(
                "/work/mobile_client/lib/features/home.dart",
                Some("company_app"),
                Some(&root),
            )
            .as_deref(),
            Some("package:company_app/features/home.dart")
        );
    }

    #[test]
    fn infers_a_valid_package_name_from_the_main_library_root() {
        assert_eq!(
            package_name_from_lib_root("/work/clutter_edge_cases/lib").as_deref(),
            Some("clutter_edge_cases")
        );
        assert_eq!(package_name_from_lib_root("/work/Clutter-App/lib"), None);
    }
}
