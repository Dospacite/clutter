#!/usr/bin/env python3
"""Check Clutter's output for the adversarial constructs in lib/hardening.dart.

The simple_app fixtures carry code an app author could use to minimize what a
decompiler recovers or to break its output (see lib/hardening.dart). This
scorer checks that the generated pseudocode stays well-formed and honest:

- every generated library parses as Dart (`dart format`);
- no raw control or bidirectional-formatting character reaches a generated
  file or report;
- no hostile literal escapes its string or comment into code;
- snapshot label internals never leak into expressions;
- output paths stay unique, even case-insensitively;
- application names that collide with Clutter's own vocabulary (`aot`,
  `native`, `Context`, `sub_1000`, register-shaped `x1`/`r2`) keep their
  meaning in builds that retain names.

Usage: evaluate_hardening.py [--no-parse] NAME=OUTPUT_DIRECTORY...
"""

import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

NAMED_VARIANTS = {"plain", "plain_arm32", "plain_x64"}
BIDI_OR_CONTROL = re.compile("[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f"
                             "؜‎‏‪-‮⁦-⁩]")


def code_only(source):
    """`source` with string literal contents and comments blanked out."""
    out = []
    i = 0
    length = len(source)
    while i < length:
        c = source[i]
        if source.startswith("//", i):
            end = source.find("\n", i)
            i = length if end < 0 else end
        elif source.startswith("/*", i):
            end = source.find("*/", i + 2)
            i = length if end < 0 else end + 2
        elif c in "'\"":
            raw = i > 0 and source[i - 1] == "r"
            triple = source.startswith(c * 3, i)
            quote = c * 3 if triple else c
            i += len(quote)
            while i < length and not source.startswith(quote, i):
                i += 2 if source[i] == "\\" and not raw else 1
            i += len(quote)
            out.append('""')
        else:
            out.append(c)
            i += 1
    return "".join(out)


def unparsable(files):
    if not files:
        return []
    result = subprocess.run(["dart", "format", "--output=none", *map(str, files)],
                            capture_output=True, text=True)
    return sorted(set(re.findall(r"of (\S+\.dart):", result.stdout + result.stderr)))


def evaluate(name, output, parse):
    checks = {}

    def check(check_name, condition):
        checks[check_name] = bool(condition)

    files = sorted((output / "lib").rglob("*.dart"))
    sources = {path: path.read_text(encoding="utf-8") for path in files}
    if parse:
        broken = unparsable(files)
        check("all_libraries_parse", not broken)
        for path in broken:
            check(f"parses:{Path(path).relative_to(output / 'lib')}", False)
    check("no_raw_control_or_bidi",
          not any(BIDI_OR_CONTROL.search(text) for text in sources.values()))
    # Reports carry the app's strings too; they must display as they parse.
    reports = [path for path in output.rglob("*")
               if path.is_file() and path.suffix in {".json", ".jsonl", ".s", ".md"}]
    check("reports_free_of_bidi", not any(
        BIDI_OR_CONTROL.search(path.read_text(encoding="utf-8", errors="replace"))
        for path in reports))
    code = {path: code_only(text) for path, text in sources.items()}
    check("no_injected_code", not any("injectedCanary" in text for text in code.values()))
    check("no_nested_string_labels", not any("nestedStrings[" in text for text in code.values()))
    check("no_pool_code_entry_reads",
          not any(re.search(r"resetPoolEntry\(\d+\)\._slot_", text) for text in code.values()))

    libraries = json.loads((output / "reports/libraries.json").read_text(encoding="utf-8"))
    libraries = libraries if isinstance(libraries, list) else libraries["libraries"]
    paths = [library["output_path"].lower() for library in libraries]
    check("unique_output_paths", len(paths) == len(set(paths)))

    if name in NAMED_VARIANTS:
        hardening = sources.get(output / "lib/hardening.dart", "")
        hardening_code = code.get(output / "lib/hardening.dart", "")
        # The application declares `aot`, so the support prefix moves.
        check("support_prefix_renamed",
              "as aot_;" in hardening and re.search(r"\baot\(", hardening_code))
        check("aot_collision_not_aliased", "aotResult.closure(" not in hardening_code)
        check("register_named_functions_called",
              re.search(r"\bx1\(", hardening_code) and re.search(r"\br2\(", hardening_code)
              # ARM64's own x1 may stay unresolved; the function call must not.
              and not re.search(r"unresolvedRegister\('(x1|r2)'\)\(", hardening))
        check("sub_named_function_called", re.search(r"\bsub_1000\(", hardening_code))
        check("native_function_not_vm_native", "aot_.native(" not in hardening_code)
        check("application_context_not_vm_context",
              not re.search(r"contextResult\d*\.(parent|captured\d+)\b", hardening_code))
        check("vm_closure_contexts_named", "._context.captured" in hardening_code)
        # `runHardening` builds its report as a list literal; the element
        # stores into the backing array must stay visible.
        # `hostileLiterals[seed % n]` reads a constant list by index.
        check("list_element_read_recovered",
              re.search(r"hostileScore\(const <String>\[.*\]\[[^\]]+\]\)", hardening_code))
        check("list_literal_elements_recovered",
              re.search(r"\[\d+\] = fakeCoreResult;", hardening_code)
              and re.search(r"\[0\] = hostileScoreResult;", hardening_code))

    failures = [check_name for check_name, passed in checks.items() if not passed]
    return {"passed": not failures, "failed_checks": failures, "check_count": len(checks),
            "libraries": len(files)}


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("variants", nargs="+", metavar="NAME=OUTPUT")
    parser.add_argument("--no-parse", action="store_true",
                        help="skip the `dart format` parse check")
    arguments = parser.parse_args()
    parse = not arguments.no_parse
    if parse and shutil.which("dart") is None:
        print("dart is not on PATH; rerun with --no-parse", file=sys.stderr)
        return 2
    report = {}
    for variant in arguments.variants:
        name, _, path = variant.partition("=")
        report[name] = evaluate(name, Path(path), parse)
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if all(result["passed"] for result in report.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
