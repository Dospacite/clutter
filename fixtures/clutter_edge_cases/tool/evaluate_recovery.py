#!/usr/bin/env python3
"""Compare Clutter IR with the fixture's recovery expectations."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as source:
        return json.load(source)


def contains(symbols: set[str], expected: str) -> bool:
    return any(expected == symbol or expected in symbol for symbol in symbols)


def evaluate_semantic_features(
    expected_features: list[str],
    program: dict,
    all_symbols: set[str],
    coverage: dict,
) -> dict[str, bool]:
    functions = program["functions"]
    strings = {item.get("value", "") for item in program.get("strings", [])}
    snapshot = program.get("snapshot_evidence") or {}

    def function(name: str) -> dict | None:
        return next(
            (
                value
                for value in functions
                if name == value.get("name") or name in value.get("name", "")
            ),
            None,
        )

    recursive = function("recursiveChecksum")
    recursive_call = bool(
        recursive
        and any(
            statement.get("kind") == "direct_call"
            and "recursiveChecksum" in (statement.get("target") or "")
            for statement in recursive.get("statements", [])
        )
    )
    optional_signatures = all(
        bool(
            (value := function(name))
            and (value.get("signature") or {}).get("optional_parameter_count", 0) > 0
        )
        for name in ("optionalPositional", "optionalNamed")
    )
    has_exception_handlers = any(
        (value.get("code_metadata") or {}).get("exception_handlers")
        for value in functions
    )
    known = {
        "async state machines": contains(all_symbols, "guardedAsyncValue"),
        "async-star stream state machines": contains(all_symbols, "countedStream"),
        "capturing and non-capturing closures": (
            contains(all_symbols, "closurePipeline")
            and coverage.get("function_kinds", {}).get("closure", 0) > 0
        ),
        "constructors and factory constructors": (
            contains(all_symbols, "EdgeVector")
            and coverage.get("function_kinds", {}).get("constructor", 0) > 0
        ),
        "default optional positional and named arguments": optional_signatures,
        "enhanced enums": (
            contains(all_symbols, "EdgeFlavor")
            or (
                "plain" in strings
                and contains(all_symbols, "label")
                and contains(all_symbols, "weight")
            )
        ),
        "extensions": (
            contains(all_symbols, "WordMetrics")
            or (
                contains(all_symbols, "vowelCount")
                and contains(all_symbols, "bracketed")
            )
        ),
        "generic classes and generic functions": (
            contains(all_symbols, "GenericBox")
            and contains(all_symbols, "genericFirst")
        ),
        "getters and operators": (
            coverage.get("function_kinds", {}).get("getter", 0) > 0
            and (
                contains(all_symbols, "operator_add")
                or contains(all_symbols, "dyn:+")
                or contains(all_symbols, "+")
            )
        ),
        "inheritance, interfaces, and mixins": all(
            contains(all_symbols, name)
            for name in ("ArithmeticOperation", "AddOperation", "EdgeVector")
        ),
        "private identifiers": any(
            symbol.startswith("_") and len(symbol) > 1 for symbol in all_symbols
        ),
        "records and pattern switches": (
            contains(all_symbols, "classifyRecord")
            and snapshot.get("object_kinds", {}).get("record", 0) > 0
        ),
        "recursion": recursive_call,
        "try/catch/finally": (
            contains(all_symbols, "guardedAsyncValue") and has_exception_handlers
        ),
        "Unicode and escaped strings": any(
            "Καλημέρα" in value and "\n" in value for value in strings
        ),
    }
    return {feature: known.get(feature, False) for feature in expected_features}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path, help="Clutter output directory")
    parser.add_argument(
        "--expectations",
        type=Path,
        default=Path(__file__).resolve().parent / "recovery_expectations.json",
        help="expectation JSON (defaults to the non-obfuscated fixture profile)",
    )
    args = parser.parse_args()

    expected = load_json(args.expectations)
    program = load_json(args.output / "ir" / "program.json")
    coverage = load_json(args.output / "reports" / "coverage.json")

    functions = program["functions"]
    function_symbols = {
        str(value)
        for function in functions
        for value in (function.get("name"), function.get("owner"))
        if value
    }
    all_symbols = function_symbols | set(program["identifiers"])

    recovered = {
        name: contains(all_symbols, name) for name in expected["must_recover"]
    }
    retained = {
        name: contains(function_symbols, name)
        for name in expected["must_retain_by_pragma"]
    }
    shaken = {
        name: not contains(all_symbols, name)
        for name in expected["must_be_tree_shaken"]
    }
    minimum_metrics = {
        name: {
            "minimum": minimum,
            "actual": coverage.get(name, 0),
            "passed": coverage.get(name, 0) >= minimum,
        }
        for name, minimum in expected.get("minimum_metrics", {}).items()
    }
    direct_calls = coverage.get("direct_call_sites", 0)
    resolved_call_ratio = (
        coverage.get("resolved_direct_call_sites", 0) / direct_calls
        if direct_calls
        else 1.0
    )
    recovered_functions = coverage.get("recovered_functions", 0)
    source_location_ratio = (
        coverage.get("functions_with_source_locations", 0) / recovered_functions
        if recovered_functions
        else 0.0
    )
    split_debug_names = coverage.get("function_name_sources", {}).get(
        "split_debug_info", 0
    )
    split_debug_name_ratio = (
        split_debug_names / recovered_functions if recovered_functions else 0.0
    )
    ratios = {
        "resolved_call_ratio": {
            "minimum": expected.get("minimum_resolved_call_ratio", 0.0),
            "actual": resolved_call_ratio,
        },
        "source_location_ratio": {
            "minimum": expected.get("minimum_source_location_ratio", 0.0),
            "actual": source_location_ratio,
        },
        "split_debug_name_ratio": {
            "minimum": expected.get("minimum_split_debug_name_ratio", 0.0),
            "actual": split_debug_name_ratio,
        },
    }
    for value in ratios.values():
        value["passed"] = value["actual"] >= value["minimum"]
    semantic_features = evaluate_semantic_features(
        expected.get("semantic_features", []),
        program,
        all_symbols,
        coverage,
    )
    report = {
        "recovered_expectations": recovered,
        "retained_entrypoints": retained,
        "tree_shaken_expectations": shaken,
        "minimum_metrics": minimum_metrics,
        "quality_ratios": ratios,
        "semantic_features": semantic_features,
        "metrics": coverage,
    }
    print(json.dumps(report, indent=2, sort_keys=True))

    checks = (
        *recovered.values(),
        *retained.values(),
        *shaken.values(),
        *(value["passed"] for value in minimum_metrics.values()),
        *(value["passed"] for value in ratios.values()),
        *semantic_features.values(),
    )
    return 0 if all(checks) else 1


if __name__ == "__main__":
    raise SystemExit(main())
