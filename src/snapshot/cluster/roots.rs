use std::collections::BTreeMap;

use crate::diagnostic::{ClutterError, Result};

use super::cid::Cids;
use super::reader::Reader;
use super::root_names::ROOT_NAMES_3122;
use super::types::ParseResult;

/// Program roots in `ProgramSerializationRoots::WriteRoots` order: the
/// release's object-store fields, then the initial field table(s), then the
/// dispatch table. Each release family supplies its own root name list.
#[derive(Clone, Debug, Default)]
pub struct SnapshotRoots {
    /// Whether the snapshot hash is the exact Dart 3.12.2 revision whose
    /// other version-specific layouts (function kind bits) are verified.
    pub exact_3122: bool,
    /// Dart minor release whose root list decoded this snapshot.
    pub minor: u32,
    pub named: BTreeMap<String, i32>,
    pub initial_fields: Vec<i32>,
    pub shared_initial_fields: Vec<i32>,
    pub dispatch_offset: usize,
}

pub fn parse_3122(data: &[u8], start: usize, snapshot_end: usize) -> Result<SnapshotRoots> {
    let mut roots = parse(data, start, snapshot_end, ROOT_NAMES_3122, true, 12)?;
    roots.exact_3122 = true;
    Ok(roots)
}

pub fn parse(
    data: &[u8],
    start: usize,
    snapshot_end: usize,
    root_names: &[&str],
    shared_field_table: bool,
    minor: u32,
) -> Result<SnapshotRoots> {
    let data = data.get(..snapshot_end).ok_or_else(|| {
        ClutterError::InvalidArtifact("snapshot root tail exceeds data image".to_owned())
    })?;
    let mut reader = Reader::at(data, start)?;
    let mut named = BTreeMap::new();
    for name in root_names {
        named.insert((*name).to_owned(), reference(&mut reader)?);
    }
    let initial_fields = field_table(&mut reader, "initial")?;
    let shared_initial_fields = if shared_field_table {
        field_table(&mut reader, "shared initial")?
    } else {
        Vec::new()
    };
    Ok(SnapshotRoots {
        exact_3122: false,
        minor,
        named,
        initial_fields,
        shared_initial_fields,
        dispatch_offset: reader.position(),
    })
}

pub fn validate(roots: &SnapshotRoots, snapshot: &ParseResult, cids: &Cids) -> Result<()> {
    for (name, cid) in [
        ("root_library", cids.library),
        ("global_object_pool", cids.object_pool),
        ("allocate_array_stub", cids.code),
    ] {
        let reference =
            roots.named.get(name).copied().ok_or_else(|| {
                ClutterError::InvalidArtifact(format!("Dart root {name} is missing"))
            })?;
        let actual = snapshot.object(reference).map(|object| object.cid);
        if actual != Some(cid) {
            return Err(ClutterError::InvalidArtifact(format!(
                "Dart root {name} points to CID {actual:?}, expected {cid}"
            )));
        }
    }
    Ok(())
}

/// Names the VM snapshot's stub Code objects. `VMSerializationRoots`
/// writes the predefined symbols and the symbol table, then one reference
/// per `VM_STUB_CODE_LIST` entry, and nothing follows the roots. The tail
/// is kept only when every one of those references is a distinct Code
/// object.
pub fn vm_stub_names(
    data: &[u8],
    start: usize,
    snapshot_end: usize,
    stubs: &'static [&'static str],
    snapshot: &ParseResult,
    cids: &Cids,
) -> Option<BTreeMap<i32, &'static str>> {
    let data = data.get(..snapshot_end)?;
    let mut reader = Reader::at(data, start).ok()?;
    let mut references = Vec::new();
    while reader.position() < snapshot_end {
        references.push(reference(&mut reader).ok()?);
    }
    let tail = references.get(references.len().checked_sub(stubs.len())?..)?;
    let names = tail
        .iter()
        .copied()
        .zip(stubs.iter().copied())
        .collect::<BTreeMap<_, _>>();
    let all_code = tail.iter().all(|reference| {
        snapshot
            .object(*reference)
            .is_some_and(|object| object.cid == cids.code)
    });
    (all_code && names.len() == stubs.len()).then_some(names)
}

fn field_table(reader: &mut Reader<'_>, label: &str) -> Result<Vec<i32>> {
    let count = usize::try_from(reader.unsigned()?)
        .map_err(|_| ClutterError::InvalidArtifact(format!("invalid {label} field-table size")))?;
    if count > 1_000_000 {
        return Err(ClutterError::InvalidArtifact(format!(
            "{label} field table has {count} entries"
        )));
    }
    (0..count).map(|_| reference(reader)).collect()
}

fn reference(reader: &mut Reader<'_>) -> Result<i32> {
    i32::try_from(reader.reference()?).map_err(|_| {
        ClutterError::InvalidArtifact("snapshot root reference exceeds i32".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::{ROOT_NAMES_3122, parse_3122, vm_stub_names};
    use crate::snapshot::cluster::cid::test_cids;
    use crate::snapshot::cluster::types::{
        ClusterHeader, ParseResult, SnapshotObjectKind, SnapshotObjectPayload,
    };

    #[test]
    fn names_vm_stub_code_from_the_trailing_roots() {
        let cids = test_cids();
        let mut snapshot = ParseResult::new(ClusterHeader {
            num_base_objects: 0,
            num_objects: 0,
            num_clusters: 0,
            instruction_table_length: 0,
            instruction_table_data_offset: 0,
        });
        for reference in [5, 6, 7] {
            snapshot.insert_object(
                reference,
                cids.code,
                false,
                SnapshotObjectKind::Code,
                SnapshotObjectPayload::default(),
            );
        }
        const STUBS: &[&str] = &["CallToRuntime", "InstanceOf"];
        // Symbol roots (1, 2), the symbol table (3), then one per stub.
        let bytes = [0x81, 0x82, 0x83, 0x86, 0x87];
        let names = vm_stub_names(&bytes, 0, bytes.len(), STUBS, &snapshot, &cids).unwrap();
        assert_eq!(names.get(&6), Some(&"CallToRuntime"));
        assert_eq!(names.get(&7), Some(&"InstanceOf"));
        // A tail that is not all Code objects is rejected.
        let bytes = [0x81, 0x82, 0x83, 0x86];
        assert!(vm_stub_names(&bytes, 0, bytes.len(), STUBS, &snapshot, &cids).is_none());
        assert_eq!(
            super::super::vm_stub_names::for_minor(9).map(<[_]>::len),
            Some(171)
        );
    }

    #[test]
    fn reads_exact_root_order_and_field_tables() {
        assert_eq!(ROOT_NAMES_3122.len(), 244);
        assert_eq!(ROOT_NAMES_3122[118], "root_library");
        assert_eq!(ROOT_NAMES_3122[162], "global_object_pool");
        assert_eq!(ROOT_NAMES_3122[186], "allocate_array_stub");
        let mut bytes = vec![0x81; ROOT_NAMES_3122.len()];
        bytes.extend([0x82, 0x83, 0x84]); // two initial field references
        bytes.extend([0x81, 0x85]); // one shared field reference
        let roots = parse_3122(&bytes, 0, bytes.len()).unwrap();
        assert_eq!(roots.named["root_library"], 1);
        assert_eq!(roots.initial_fields, [3, 4]);
        assert_eq!(roots.shared_initial_fields, [5]);
        assert_eq!(roots.dispatch_offset, bytes.len());
        assert!(parse_3122(&bytes[..10], 0, 10).is_err());
    }
}
