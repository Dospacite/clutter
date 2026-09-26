#!/usr/bin/env python3
"""Check static recovery regressions for the local Pavo Connect APK.

This checks retained metadata and emitted evidence, not source equivalence.
The proprietary APK and generated source stay outside version control.
"""

import argparse
from collections import Counter
import json
import re
from pathlib import Path


def load(path):
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def evaluate(output):
    manifest = load(output / "decompilation.json")
    coverage = load(output / "reports/coverage.json")
    functions = load(output / "reports/functions.json")["functions"]
    roots = load(output / "metadata/snapshot_roots.json")
    checks = {}

    def check(name, condition):
        checks[name] = bool(condition)

    check("exact_input", manifest["input"]["input_sha256"] ==
          "6ff2936f5f0d289be21beb098a0356811b68b1b7c18dcc683d0a853bdd2ad5e2")
    check("arm32", manifest["selected_abi"] == "armeabi-v7a")
    check("all_functions_preserved", coverage["recovered_functions"] == 16515)
    check("complete_instruction_decode", coverage["undecoded_function_bytes"] == 0)
    check("root_profile", roots["profile"] == "dart-3.9")
    check("root_reference", roots["root_library_reference"] == 53134)
    check("root_uri", bool(manifest["root_library_uri"]))
    check("no_invented_application_package", manifest["application_package"] is None)
    check("opaque_scope_preserved", any(w["code"] == "W_OBFUSCATED_SCOPE_BROADENED"
                                       for w in manifest["warnings"]))

    modifiers = Counter(f.get("async_modifier") or "unknown" for f in functions)
    check("async_metadata", modifiers["async"] == 24 and modifiers["none"] == 3099)
    stub_refs = set(roots["named_stub_references"].values())
    static_stubs = [f for f in functions if f["code_reference"] in stub_refs
                    or stub_refs.intersection(f["code_alias_references"])]
    check("all_stub_bodies_in_reports", len(static_stubs) == 76)
    check("stubs_evidence_only", coverage["evidence_only_functions"] >= len(static_stubs))
    check("coverage_partitions_functions", coverage["rendered_source_functions"] +
          coverage["evidence_only_functions"] == len(functions))

    # The lift may recover Await only from a suspension descriptor in a
    # Function whose retained modifier proves async. Ordinary calls to an
    # async collaborator must not be promoted into suspension boundaries.
    awaits = []
    for function in functions:
        descriptors = (function.get("code_metadata") or {}).get("pc_descriptors", [])
        suspension_pcs = {int(function["address"], 16) + d["pc_offset"]
                          for d in descriptors if d.get("yield_index", -1) >= 0}
        for statement in function.get("semantic_statements", []):
            if statement.get("target") == "stub Await":
                awaits.append(statement)
                call_pc = int(statement["address"], 16)
                check("await_metadata_" + statement["address"],
                      function.get("async_modifier") == "async" and
                      bool(suspension_pcs.intersection({call_pc, call_pc + 4})))
    check("await_boundaries", len(awaits) >= 27)

    # Calls through pool-loaded Code objects and VM native trampolines.
    targets = Counter(statement["target"]
                      for function in functions
                      for statement in function.get("semantic_statements", [])
                      if statement["kind"] == "resolved_call")
    # Each of the 169 `nativePoolEntry` loads feeds one `NativeCallInstr`.
    check("native_calls", targets["native call"] == 169)
    # `InstanceOf` is named through the VM snapshot's stub roots.
    check("vm_stub_pool_calls", targets["stub InstanceOf"] == 157)
    check("no_code_entry_field_reads", not any(
        statement["kind"] == "field_read"
        and statement["receiver"].startswith(("stub ", "resetPoolEntry("))
        for function in functions
        for statement in function.get("semantic_statements", [])))
    kept_dispatch = sum(count for target, count in targets.items()
                        if target.startswith("dispatch "))
    check("unresolved_dispatch_calls_kept", kept_dispatch >= 5600)
    check("kept_dispatch_not_counted_resolved",
          coverage["resolved_dispatch_table_call_sites"] < 100)

    source = "\n".join(p.read_text(encoding="utf-8")
                       for p in (output / "lib").rglob("*.dart"))
    check("closure_contexts_rendered",
          "aot.context(" in source and "._context.captured0" in source)
    # Reset pool entries that stay in source are FFI lazy-lookup caches;
    # none may surface as a Code object's entry-point field.
    check("no_pool_code_entry_reads",
          re.search(r"resetPoolEntry\(\d+\)\._slot_", source) is None)
    check("awaits_rendered", "await " in source)
    for stub in static_stubs:
        # The declaration's documentation carries the root-proven stub name.
        check("stub_hidden_" + stub["address"],
              f"/// Partially reconstructed `{stub['name']}`." not in source)

    failures = [name for name, passed in checks.items() if not passed]
    return {
        "passed": not failures,
        "failed_checks": failures,
        "check_count": len(checks),
        "functions": len(functions),
        "async_modifiers": dict(modifiers),
        "await_boundaries": len(awaits),
        "native_calls": targets["native call"],
        "kept_dispatch_calls": kept_dispatch,
        "static_stub_bodies": len(static_stubs),
        "root_library_uri": manifest["root_library_uri"],
        "worklist_exhausted": sum(f["machine_code"].get("semantic_worklist_exhausted", False)
                                   for f in functions),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()
    report = evaluate(arguments.output)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
