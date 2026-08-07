use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Output as ProcessOutput};

use clap::{Args, Parser, Subcommand};
use serde::Serialize;

use crate::analysis;
use crate::archive::Artifact;
use crate::diagnostic::{ClutterError, IoContext, Result};
use crate::elf::ElfImage;
use crate::model::{
    Abi, AndroidMetadata, ArchiveInfo, EvidenceConfidence, Scope, SnapshotInfo, SplitDebugInfo,
    Warning,
};
use crate::output::WriteRequest;

#[derive(Parser)]
#[command(
    name = "clutter",
    version,
    about = "Recover conservative Dart-like pseudocode from Flutter Android AOT artifacts"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Inspect(InspectArgs),
    Decompile(DecompileArgs),
    VmOracle(VmOracleArgs),
    Version(VersionArgs),
}

#[derive(Args)]
struct InspectArgs {
    input: PathBuf,

    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct DecompileArgs {
    input: PathBuf,

    #[arg(long)]
    out: PathBuf,

    /// Flutter --split-debug-info ELF or directory matching the selected ABI.
    #[arg(long, value_name = "PATH")]
    symbols: Option<PathBuf>,

    /// JSON produced by gen_snapshot --save-obfuscation-map.
    #[arg(long, value_name = "JSON")]
    obfuscation_map: Option<PathBuf>,

    /// JSON produced by Clutter's patched Dart VM snapshot analyzer.
    #[arg(long, value_name = "JSON")]
    vm_oracle: Option<PathBuf>,

    #[arg(long, default_value = "auto")]
    abi: String,

    #[arg(long, default_value = "base")]
    module: String,

    #[arg(long, default_value = "app")]
    scope: String,

    #[arg(long)]
    no_assets: bool,

    #[arg(long)]
    emit_ir: bool,

    /// Compare logical functions and control-flow fingerprints across every packaged ABI.
    #[arg(long)]
    cross_abi: bool,

    #[arg(long)]
    jobs: Option<usize>,

    #[arg(long)]
    replace: bool,
}

#[derive(Args)]
struct VersionArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct VmOracleArgs {
    input: PathBuf,

    #[arg(long)]
    out: PathBuf,

    /// ABI-compatible patched Dart analyze_snapshot executable.
    #[arg(long, value_name = "PATH")]
    analyzer: PathBuf,

    /// Run the Android analyzer through adb on this device serial, or `auto`.
    #[arg(long, value_name = "SERIAL")]
    adb: Option<String>,

    #[arg(long, default_value = "auto")]
    abi: String,

    #[arg(long, default_value = "base")]
    module: String,

    #[arg(long)]
    replace: bool,
}

#[derive(Serialize)]
struct InspectReport<'a> {
    schema: &'static str,
    tool_version: &'static str,
    artifact: &'a ArchiveInfo,
    selected_abi: Abi,
    android: &'a AndroidMetadata,
    snapshot: &'a SnapshotInfo,
    instruction_bytes: usize,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Inspect(arguments) => inspect(arguments),
        Command::Decompile(arguments) => decompile(arguments),
        Command::VmOracle(arguments) => vm_oracle(arguments),
        Command::Version(arguments) => version(arguments),
    }
}

fn inspect(arguments: InspectArgs) -> Result<()> {
    let artifact = Artifact::open(&arguments.input)?;
    let payload = artifact.select_payload("base", None)?;
    let android = artifact.android_metadata()?;
    let (snapshot, instruction_bytes) = load_snapshot(&artifact, &payload)?;
    let report = InspectReport {
        schema: "clutter.inspect/v1",
        tool_version: env!("CARGO_PKG_VERSION"),
        artifact: artifact.info(),
        selected_abi: payload.abi,
        android: &android,
        snapshot: &snapshot,
        instruction_bytes,
    };
    if arguments.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_inspect(&report);
    }
    Ok(())
}

fn decompile(arguments: DecompileArgs) -> Result<()> {
    if let Some(jobs) = arguments.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.max(1))
            .build_global()
            .map_err(|error| ClutterError::Usage(format!("invalid --jobs setting: {error}")))?;
    }
    let abi = if arguments.abi == "auto" {
        None
    } else {
        Some(arguments.abi.parse::<Abi>().map_err(ClutterError::Usage)?)
    };
    let scope = arguments
        .scope
        .parse::<Scope>()
        .map_err(ClutterError::Usage)?;
    let artifact = Artifact::open(&arguments.input)?;
    let payload = artifact.select_payload(&arguments.module, abi)?;
    let android = artifact.android_metadata()?;
    let libapp = artifact.read_payload(&payload.libapp)?;
    let libflutter = payload
        .libflutter
        .as_deref()
        .map(|path| artifact.read_payload(path))
        .transpose()?;
    let elf = ElfImage::parse(&libapp, payload.abi)?;
    let snapshot = crate::snapshot::inspect(&elf, libflutter.as_deref())?;
    let code = crate::snapshot::isolate_code(&snapshot, elf.pointer_width)?;
    let mut program = analysis::recover(&libapp, &snapshot, scope);
    let deferred_payloads = artifact
        .info()
        .deferred_payloads
        .iter()
        .filter(|deferred| deferred.module == payload.module && deferred.abi == payload.abi)
        .cloned()
        .collect::<Vec<_>>();
    for deferred in &deferred_payloads {
        match artifact
            .read_payload(&deferred.path)
            .and_then(|bytes| analysis::inspect_deferred_unit(&deferred.path, deferred.abi, &bytes))
        {
            Ok(evidence) => program.deferred_units.push(evidence),
            Err(error) => program.warnings.push(Warning {
                code: "W_DEFERRED_UNIT_INDEX_FAILED".to_owned(),
                message: format!(
                    "Could not index deferred loading unit {}: {error}",
                    deferred.path
                ),
            }),
        }
    }
    if !deferred_payloads.is_empty() {
        program.warnings.push(Warning {
            code: "W_DEFERRED_UNITS_INDEXED".to_owned(),
            message: format!(
                "The selected payload has {} deferred AOT loading unit(s). Their ELF identity, snapshot symbols, and instruction-section sizes are recorded in metadata/deferred_units.json; logical function reconstruction currently covers the root unit.",
                deferred_payloads.len(),
            ),
        });
    }
    let obfuscation_map = arguments
        .obfuscation_map
        .as_deref()
        .map(analysis::load_obfuscation_map)
        .transpose()?;
    let snapshot_scope = if arguments.symbols.is_some() {
        Scope::All
    } else {
        scope
    };
    let recover_snapshot = || {
        crate::snapshot::recover_functions(
            &snapshot,
            &code,
            payload.abi,
            elf.pointer_width,
            snapshot_scope,
            program.application_package.as_deref(),
            obfuscation_map.as_ref(),
        )
    };
    let (snapshot_recovery, debug_symbols) = if let Some(symbol_path) = arguments.symbols.as_deref()
    {
        let load_debug = || {
            analysis::load_debug_symbols(
                symbol_path,
                &libapp,
                payload.abi,
                program.application_package.as_deref(),
            )
        };
        let (snapshot_result, debug_result) = rayon::join(recover_snapshot, load_debug);
        (snapshot_result?, Some(debug_result?))
    } else {
        (recover_snapshot()?, None)
    };
    if program.application_package.is_none()
        && let Some(package) = debug_symbols
            .as_ref()
            .and_then(|debug| debug.application_package.clone())
    {
        program.application_package = Some(package.clone());
        program
            .warnings
            .retain(|warning| warning.code != "W_APP_PACKAGE_UNKNOWN");
        program.warnings.push(Warning {
            code: "W_APP_PACKAGE_INFERRED_FROM_DWARF".to_owned(),
            message: format!(
                "Application package `{package}` was inferred from the split-debug main library root because obfuscation removed package URIs from libapp.so."
            ),
        });
    }
    let ownership_obfuscated = snapshot_recovery.ownership_obfuscated;
    program.snapshot_evidence = Some(snapshot_recovery.snapshot_evidence.clone());
    program.dispatch_table = snapshot_recovery.dispatch_table;
    analysis::attach_snapshot_strings(&mut program, snapshot_recovery.snapshot_strings);
    let vm_oracle = arguments
        .vm_oracle
        .as_deref()
        .map(|path| crate::vm_oracle::load(path, &snapshot, payload.abi))
        .transpose()?;
    if let Some(oracle) = &vm_oracle {
        crate::vm_oracle::apply_root(&mut program, oracle);
    }
    let heuristic_string_count = program
        .strings
        .iter()
        .filter(|value| value.confidence == Some(EvidenceConfidence::Medium))
        .count();
    if heuristic_string_count > 0 {
        program.warnings.push(Warning {
            code: "W_HEURISTIC_STRING_TRANSDUCTION".to_owned(),
            message: format!(
                "Recovered {heuristic_string_count} medium-confidence string candidate(s) through bounded XOR transduction. Treat them as heuristic evidence and verify them against use-site behavior."
            ),
        });
    }
    let output_scope = if arguments.symbols.is_none()
        && obfuscation_map.is_none()
        && ownership_obfuscated
        && scope != Scope::All
    {
        program.warnings.push(Warning {
            code: "W_OBFUSCATED_SCOPE_BROADENED".to_owned(),
            message: format!(
                "Flutter obfuscation hides complete library ownership needed for --scope {}. Clutter is preserving all raw snapshot functions instead of silently discarding them. {}Supply --obfuscation-map for identifier restoration; --symbols remains optional source-level enrichment.",
                arguments.scope,
                if vm_oracle.is_some() {
                    "The Dart VM oracle identifies the root library but cannot reconstruct source package boundaries erased by obfuscation. "
                } else {
                    ""
                }
            ),
        });
        Scope::All
    } else {
        scope
    };
    let snapshot_functions = snapshot_recovery.functions;
    let snapshot_declarations = snapshot_recovery.declarations;
    let functions = if let Some(debug) = debug_symbols {
        let linked_declarations = analysis::recover_linked_snapshot_declarations(
            &debug,
            &snapshot_functions,
            snapshot_declarations,
            scope,
            program.application_package.as_deref(),
        );
        analysis::attach_declarations(&mut program, linked_declarations, scope);
        let functions = analysis::recover_debug_functions(
            &debug,
            snapshot_functions,
            &code,
            payload.abi,
            scope,
            program.application_package.as_deref(),
        )?;
        let debug_declarations = analysis::recover_debug_declarations(
            &debug,
            scope,
            program.application_package.as_deref(),
        );
        analysis::attach_declarations(&mut program, debug_declarations, scope);
        if functions.is_empty() {
            return Err(ClutterError::Analysis(format!(
                "matching split debug info {} yielded no functions for --scope {}",
                debug.path.display(),
                arguments.scope
            )));
        }
        program.split_debug_info = Some(SplitDebugInfo {
            path: debug.path,
            build_id: debug.build_id,
            text_symbol_count: debug.functions.len(),
        });
        functions
    } else {
        analysis::attach_declarations(&mut program, snapshot_declarations, output_scope);
        if snapshot
            .isolate_header
            .features
            .iter()
            .any(|feature| feature == "dwarf_stack_traces_mode")
        {
            program.warnings.push(Warning {
                code: "W_SPLIT_DEBUG_INFO_UNAVAILABLE".to_owned(),
                message: "The matching Flutter split-debug ELF was not supplied. Snapshot and machine-code analysis continues, including serialized CodeSourceMap lines when present; --symbols can optionally add original qualified names, DWARF line/column spans, inline ranges, and ELF-sized ranges."
                    .to_owned(),
            });
        }
        snapshot_functions
    };
    if let Some(oracle) = &vm_oracle {
        crate::vm_oracle::apply_declarations(&mut program, oracle, output_scope);
    }
    analysis::attach_functions(&mut program, functions, output_scope);
    if let Some(oracle) = vm_oracle {
        crate::vm_oracle::attach(&mut program, oracle, &snapshot, payload.abi)?;
    }
    analysis::relink_calls(&mut program);
    if arguments.cross_abi && ownership_obfuscated && obfuscation_map.is_none() {
        program.warnings.push(Warning {
            code: "W_CROSS_ABI_NEEDS_NAME_MAP".to_owned(),
            message: "Cross-ABI comparison was skipped because library ownership is obfuscated. Supply the matching --obfuscation-map so logical functions can be aligned across architectures."
                .to_owned(),
        });
    } else if arguments.cross_abi {
        let mut alternatives = Vec::new();
        for other_abi in artifact
            .info()
            .available_abis
            .iter()
            .copied()
            .filter(|abi| *abi != payload.abi)
        {
            let recovery = (|| -> Result<Vec<crate::model::RecoveredFunction>> {
                let other_payload = artifact.select_payload(&arguments.module, Some(other_abi))?;
                let other_libapp = artifact.read_payload(&other_payload.libapp)?;
                let other_libflutter = other_payload
                    .libflutter
                    .as_deref()
                    .map(|path| artifact.read_payload(path))
                    .transpose()?;
                let other_elf = ElfImage::parse(&other_libapp, other_abi)?;
                let other_snapshot =
                    crate::snapshot::inspect(&other_elf, other_libflutter.as_deref())?;
                let other_code =
                    crate::snapshot::isolate_code(&other_snapshot, other_elf.pointer_width)?;
                Ok(crate::snapshot::recover_functions(
                    &other_snapshot,
                    &other_code,
                    other_abi,
                    other_elf.pointer_width,
                    output_scope,
                    program.application_package.as_deref(),
                    obfuscation_map.as_ref(),
                )?
                .functions)
            })();
            match recovery {
                Ok(functions) => alternatives.push((other_abi, functions)),
                Err(error) => program.warnings.push(Warning {
                    code: "W_CROSS_ABI_RECOVERY_FAILED".to_owned(),
                    message: format!(
                        "Could not compare packaged ABI {other_abi} with {}: {error}",
                        payload.abi
                    ),
                }),
            }
        }
        if alternatives.is_empty() {
            program.warnings.push(Warning {
                code: "W_CROSS_ABI_UNAVAILABLE".to_owned(),
                message: "Cross-ABI comparison was requested, but no additional ABI payload could be analyzed."
                    .to_owned(),
            });
        } else {
            program.cross_abi = Some(analysis::compare_cross_abi(
                payload.abi,
                &program.functions,
                alternatives,
            ));
        }
    }
    if let Some(map) = obfuscation_map {
        map.record_in(&mut program);
        program.warnings.push(Warning {
            code: "W_OBFUSCATION_MAP_UNVERIFIED".to_owned(),
            message: "Flutter obfuscation-map JSON contains no ABI or build ID. Name restoration is trustworthy only when the map came from the exact selected APK/AAB build and target ABI."
                .to_owned(),
        });
        if arguments.symbols.is_none() {
            program.warnings.push(Warning {
                code: "W_OBFUSCATION_MAP_WITHOUT_DEBUG_SYMBOLS".to_owned(),
                message: "The obfuscation map restored retained identifiers and library ownership without debug symbols. Synthetic instruction-table entries remain address-named; DWARF line/column spans and inline ranges are unavailable, though serialized CodeSourceMap lines may still survive."
                    .to_owned(),
            });
        }
    }

    let backup = crate::output::write(
        &arguments.out,
        WriteRequest {
            artifact: &artifact,
            module: &arguments.module,
            abi: payload.abi,
            android: &android,
            snapshot: &snapshot,
            program: &program,
            instruction_bytes: code.bytes.len(),
            include_assets: !arguments.no_assets,
            emit_ir: arguments.emit_ir,
            replace: arguments.replace,
        },
    )?;
    println!(
        "Recovered {} libraries, {} functions, and {} strings into {}",
        program.libraries.len(),
        program.functions.len(),
        program.strings.len(),
        arguments.out.display()
    );
    for warning in &program.warnings {
        eprintln!("warning [{}]: {}", warning.code, warning.message);
    }
    if let Some(backup) = backup {
        println!("Previous output preserved at {}", backup.display());
    }
    Ok(())
}

fn vm_oracle(arguments: VmOracleArgs) -> Result<()> {
    if arguments.out.exists() && !arguments.replace {
        return Err(ClutterError::OutputExists(arguments.out));
    }
    fs::metadata(&arguments.analyzer).at(&arguments.analyzer)?;
    if let Some(parent) = arguments
        .out
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).at(parent)?;
    }

    let requested_abi = if arguments.abi == "auto" {
        None
    } else {
        Some(arguments.abi.parse::<Abi>().map_err(ClutterError::Usage)?)
    };
    let artifact = Artifact::open(&arguments.input)?;
    let payload = artifact.select_payload(&arguments.module, requested_abi)?;
    let libapp = artifact.read_payload(&payload.libapp)?;
    let libflutter = payload
        .libflutter
        .as_deref()
        .map(|path| artifact.read_payload(path))
        .transpose()?;
    let elf = ElfImage::parse(&libapp, payload.abi)?;
    let snapshot = crate::snapshot::inspect(&elf, libflutter.as_deref())?;

    let temporary = tempfile::tempdir().at(&arguments.input)?;
    let local_libapp = temporary.path().join("libapp.so");
    fs::write(&local_libapp, libapp).at(&local_libapp)?;
    let dwarf_stack_traces = snapshot
        .isolate_header
        .features
        .iter()
        .any(|feature| feature == "dwarf_stack_traces_mode");

    if let Some(serial) = arguments.adb.as_deref() {
        run_android_vm_oracle(
            serial,
            &arguments.analyzer,
            &local_libapp,
            &arguments.out,
            dwarf_stack_traces,
        )?;
    } else {
        let mut command = ProcessCommand::new(&arguments.analyzer);
        if dwarf_stack_traces {
            command.arg("--dwarf-stack-traces");
        }
        command
            .arg(format!("--out={}", arguments.out.display()))
            .arg(&local_libapp);
        run_process(&mut command, "Dart VM snapshot analyzer")?;
    }

    let oracle = crate::vm_oracle::load(&arguments.out, &snapshot, payload.abi)?;
    println!(
        "Dart VM oracle wrote {} objects, {} libraries, {} classes, {} functions, and {} code objects to {}",
        oracle.evidence.object_count,
        oracle.evidence.library_count,
        oracle.evidence.class_count,
        oracle.evidence.function_count,
        oracle.evidence.code_object_count,
        arguments.out.display(),
    );
    if let Some(root) = &oracle.evidence.root_library_uri {
        println!("VM-resolved root library: {root}");
    }
    Ok(())
}

fn run_android_vm_oracle(
    serial: &str,
    analyzer: &Path,
    libapp: &Path,
    output: &Path,
    dwarf_stack_traces: bool,
) -> Result<()> {
    let remote = format!("/data/local/tmp/clutter-vm-oracle-{}", std::process::id());
    let remote_analyzer = format!("{remote}/analyze_snapshot");
    let remote_libapp = format!("{remote}/libapp.so");
    let remote_output = format!("{remote}/oracle.json");

    let mut mkdir = adb_command(serial);
    mkdir.args(["shell", "mkdir", "-p", &remote]);
    run_process(&mut mkdir, "adb remote-directory creation")?;

    let result: Result<()> = (|| {
        let mut push_analyzer = adb_command(serial);
        push_analyzer
            .arg("push")
            .arg(analyzer)
            .arg(&remote_analyzer);
        run_process(&mut push_analyzer, "adb analyzer upload")?;

        let mut push_libapp = adb_command(serial);
        push_libapp.arg("push").arg(libapp).arg(&remote_libapp);
        run_process(&mut push_libapp, "adb libapp upload")?;

        let mut chmod = adb_command(serial);
        chmod.args(["shell", "chmod", "700", &remote_analyzer]);
        run_process(&mut chmod, "adb analyzer permission update")?;

        let mut analyze = adb_command(serial);
        analyze.arg("shell").arg(&remote_analyzer);
        if dwarf_stack_traces {
            analyze.arg("--dwarf-stack-traces");
        }
        analyze
            .arg(format!("--out={remote_output}"))
            .arg(&remote_libapp);
        run_process(&mut analyze, "Android Dart VM snapshot analyzer")?;

        let mut pull = adb_command(serial);
        pull.arg("pull").arg(&remote_output).arg(output);
        run_process(&mut pull, "adb oracle download")?;
        Ok(())
    })();

    let mut cleanup = adb_command(serial);
    cleanup.args(["shell", "rm", "-rf", &remote]);
    if let Err(error) = run_process(&mut cleanup, "adb oracle cleanup") {
        eprintln!("warning: {error}");
    }
    result
}

fn adb_command(serial: &str) -> ProcessCommand {
    let mut command = ProcessCommand::new("adb");
    if serial != "auto" {
        command.args(["-s", serial]);
    }
    command
}

fn run_process(command: &mut ProcessCommand, label: &str) -> Result<ProcessOutput> {
    let output = command.output().map_err(|source| ClutterError::Io {
        path: PathBuf::from(label),
        source,
    })?;
    if output.status.success() {
        return Ok(output);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(ClutterError::Analysis(format!(
        "{label} failed with {}: {}{}",
        output.status,
        stderr.trim(),
        if stdout.trim().is_empty() {
            String::new()
        } else {
            format!("; {}", stdout.trim())
        }
    )))
}

fn load_snapshot(
    artifact: &Artifact,
    payload: &crate::model::PayloadPaths,
) -> Result<(SnapshotInfo, usize)> {
    let libapp = artifact.read_payload(&payload.libapp)?;
    let libflutter = payload
        .libflutter
        .as_deref()
        .map(|path| artifact.read_payload(path))
        .transpose()?;
    let elf = ElfImage::parse(&libapp, payload.abi)?;
    let snapshot = crate::snapshot::inspect(&elf, libflutter.as_deref())?;
    let code = crate::snapshot::isolate_code(&snapshot, elf.pointer_width)?;
    Ok((snapshot, code.bytes.len()))
}

fn print_inspect(report: &InspectReport<'_>) {
    println!(
        "{}: {} bytes, SHA-256 {}",
        report.artifact.format, report.artifact.input_size, report.artifact.input_sha256
    );
    println!(
        "Android package: {}",
        report.android.package_name.as_deref().unwrap_or("unknown")
    );
    println!(
        "ABIs: {} (selected {})",
        report
            .artifact
            .available_abis
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        report.selected_abi
    );
    println!(
        "Dart: {} / snapshot {} / profile {}",
        report.snapshot.dart_version.as_deref().unwrap_or("unknown"),
        report.snapshot.vm_header.snapshot_hash,
        report.snapshot.profile_id
    );
    println!(
        "AOT instruction payload: {} bytes; Flutter assets: {} files",
        report.instruction_bytes, report.artifact.asset_count
    );
}

fn version(arguments: VersionArgs) -> Result<()> {
    if arguments.json {
        println!(
            "{}",
            serde_json::json!({
                "name": "clutter",
                "version": env!("CARGO_PKG_VERSION"),
                "schema": "clutter.version/v1"
            })
        );
    } else {
        println!("clutter {}", env!("CARGO_PKG_VERSION"));
    }
    Ok(())
}
