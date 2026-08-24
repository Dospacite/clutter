#!/usr/bin/env bash
# Differential compiler laboratory.
#
# Generates small Dart programs (one construct per case), compiles each across
# every installed Flutter/SDK version, all three ABIs, and obfuscation modes,
# runs Clutter on every artifact, then mines cross-build lowering templates
# and regression fixtures from the recovered IR.
#
# Usage:
#   test/tool/compiler_lab.sh [--corpus DIR] [--out DIR] [--abis LIST] [--cases NAME,...]
#
# Outputs under --out (default: target/compiler-lab):
#   <case>/<variant>/decompilation.json   Clutter output per build variant
#   matrix.json                           one row per (case x variant)
#   templates/<case>.json                 mined lowering templates per construct
#   fixtures/regressions.json             stable facts to assert in CI
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || echo "")"
if [[ -z "$root" ]]; then
  root="$(cd "$(dirname "$0")/../.." && pwd)"
fi
lab_out="target/compiler-lab"
abis="android-arm64,android-arm,android-x64"
cases="all"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) lab_out="$2"; shift 2 ;;
    --abis) abis="$2"; shift 2 ;;
    --cases) cases="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

mkdir -p "$lab_out/cases"

# ---------------------------------------------------------------------------
# Corpus: minimal Dart programs. Each isolates ONE language construct so any
# instruction-sequence difference across SDK versions or ABIs maps cleanly to
# a lowering change.
# ---------------------------------------------------------------------------
write_case() {
  local name="$1"; shift
  local dir="$lab_out/cases/$name"
  mkdir -p "$dir/lib"
  cat > "$dir/pubspec.yaml" <<PUBSPEC
name: lab_$name
description: "Compiler-lab case: $name"
publish_to: 'none'
version: 1.0.0

environment:
  sdk: ^3.0.0

flutter:
  uses-material-design: false
PUBSPEC
  cat > "$dir/analysis_options.yaml" <<'LINTS'
# Pure-Dart lab case: no lint package dependency needed.
LINTS
  cat > "$dir/lib/main.dart"
}

write_case const_arith <<'EOF'
// Small integer arithmetic and constants.
int add(int a, int b) => a + b;
int shiftMask(int x) => (x << 3) & 0xff;

void main() {
  print(add(40, 2));
  print(shiftMask(1000));
}
EOF

write_case double_pool <<'EOF'
// Doubles materialize through the object pool; pool bit patterns are the
// template signal for float lowering.
double half(double x) => x * 0.5;
String price(double v) => v.toStringAsFixed(2);

void main() {
  print(half(10.0));
  print(price(19.99));
}
EOF

write_case string_interpolation <<'EOF'
// Interpolation lowers to _interpolate with an array allocation; the call
// topology is ABI-stable and anchors string recovery.
String deal(String name, int off) => 'Deal: $name at $off%';

void main() {
  print(deal('widgets', 20));
}
EOF

write_case named_arguments <<'EOF'
// Optional named parameters exercise argument descriptors end-to-end.
class Order {
  final String sku;
  final int qty;
  Order({required this.sku, this.qty = 1});
  int total(int unit) => unit * qty;
}

void main() {
  final o = Order(sku: 'A-1', qty: 3);
  print(o.total(5));
  print(Order(sku: 'B-2').total(7));
}
EOF

write_case closures <<'EOF'
// Closures share optimized bodies and collide at identical entry addresses;
// the body graph must keep every occurrence distinct.
Function counter(int start) {
  var n = start;
  return () => n++;
}

void main() {
  final c = counter(10);
  print(c());
  print(c());
}
EOF

write_case dispatch_polymorphism <<'EOF'
// Interface dispatch through the megamorphic/dispatch table path.
abstract class Shape {
  double area();
}
class Square extends Shape {
  final double side;
  Square(this.side);
  @override
  double area() => side * side;
}
class Circle extends Shape {
  final double radius;
  Circle(this.radius);
  @override
  double area() => 3.14159 * radius * radius;
}

double total(List<Shape> shapes) =>
    shapes.fold(0.0, (sum, s) => sum + s.area());

void main() {
  print(total([Square(2), Circle(1)]));
}
EOF

write_case async_await <<'EOF'
// Async lowering: suspension points and the async stub machinery.
Future<int> load() async => 42;

Future<void> main() async {
  final v = await load();
  print('value=$v');
}
EOF

write_case generics_bounds <<'EOF'
// Generic function with a bound; type-parameter checks appear as runtime
// entries in PC descriptors.
T pick<T extends num>(List<T> xs, int i) => xs[i];

void main() {
  print(pick([1, 2, 3], 1));
  print(pick([1.5, 2.5], 0));
}
EOF

write_case try_catch <<'EOF'
// Typed exception guards lower through the exception-handler tables; the
// handler rows (handler_pc_offset, outer_try_index) are the template signal.
String describe(String raw) {
  try {
    final value = int.parse(raw);
    return 'ok:$value';
  } on FormatException catch (e) {
    return 'format:${e.runtimeType}';
  } catch (e) {
    return 'other:$e';
  }
}

void main() {
  print(describe('7'));
  print(describe('nope'));
}
EOF

write_case pattern_switch <<'EOF'
// Dart 3 switch-expression patterns: guards, object/list/map patterns and
// the exhaustiveness fallthrough all reshape branch structure.
String shape(Object value) => switch (value) {
      int n when n < 0 => 'negative',
      int() => 'int',
      [_, ...] => 'list',
      {'name': String name} => 'map:${name.length}',
      _ => 'other',
    };

void main() {
  print(shape(-3));
  print(shape(9));
  print(shape([1]));
  print(shape({'name': 'dart'}));
  print(shape(1.5));
}
EOF

write_case records_destructuring <<'EOF'
// Records lower to anonymous shape-class allocations; positional and named
// field reads exercise distinct getter paths.
(int, String) pair(int id) => (id, 'item-$id');
({int x, int y}) point() => (x: 1, y: 2);

void main() {
  final (id, label) = pair(4);
  print('$label=$id');
  final p = point();
  print('${p.x},${p.y}');
}
EOF

write_case async_star_stream <<'EOF'
// async* generators suspend between yields; the stream machinery differs
// from single-shot async (yield vs await stubs).
Stream<int> ticks(int n) async* {
  for (var i = 1; i <= n; i++) {
    yield i;
  }
}

Future<void> main() async {
  await for (final t in ticks(3)) {
    print(t);
  }
}
EOF

write_case factory_constructors <<'EOF'
// Factory constructors compile to static methods returning instances; the
// redirect/const mix pins allocation-site provenance.
class Session {
  final String user;
  final bool admin;
  const Session._(this.user, {this.admin = false});
  factory Session.guest() => const Session._('guest');
  factory Session.privileged(String user) => Session._(user, admin: true);
}

void main() {
  print(Session.guest().user);
  final p = Session.privileged('root');
  print('${p.user}:${p.admin}');
}
EOF

write_case extension_members <<'EOF'
// Extension members exist only at compile time; the lowered top-level
// functions carry mangled names that must still attribute to the extension.
extension Total on List<int> {
  int get total => fold(0, (a, b) => a + b);
  String render() => 'total=$total';
}

void main() {
  print([1, 2, 3].total);
  print([4, 5].render());
}
EOF

if [[ "$cases" != "all" ]]; then
  IFS=',' read -ra wanted <<< "$cases"
else
  mapfile -t wanted < <(cd "$lab_out/cases" && ls)
fi

# ---------------------------------------------------------------------------
# Build + decompile matrix
# ---------------------------------------------------------------------------
flutter_bin="$(command -v flutter || true)"
clutter_bin="$root/target/release/clutter"

if [[ -z "$flutter_bin" ]]; then
  echo "flutter not found on PATH: building corpus only (no matrix)" >&2
fi
if [[ ! -x "$clutter_bin" ]]; then
  (cd "$root" && cargo build --release)
fi

matrix="$lab_out/matrix.json"
echo '{"schema":"clutter.compiler-lab/v1","rows":[' > "$matrix"
first_row=true

IFS=',' read -ra abi_list <<< "$abis"
for case_name in "${wanted[@]}"; do
  case_dir="$lab_out/cases/$case_name"
  for mode in plain obfuscated; do
    extra=()
    if [[ "$mode" == obfuscated ]]; then
      extra+=(--obfuscate --split-debug-info="$lab_out/cases/$case_name/symbols-$mode")
    fi
    # Scaffold the Android platform once per case (pure-Dart cases have no
    # android/ directory until flutter create generates it).
    if [[ ! -f "$case_dir/android/app/build.gradle" && ! -f "$case_dir/android/app/build.gradle.kts" ]]; then
      (cd "$case_dir" && flutter create --platforms=android --project-name "lab_${case_name}" . >/dev/null 2>&1) \
        || { echo "flutter create failed: $case_name" >&2; continue; }
      # Restore our minimal pubspec sections that flutter create may rewrite.
      cp "$case_dir/pubspec.yaml" "$lab_out/cases/$case_name/pubspec.yaml.bak" 2>/dev/null || true
    fi
    for abi in "${abi_list[@]}"; do
      variant="${mode}-${abi}"
      out_dir="$lab_out/$case_name/$variant"
      if [[ -z "$flutter_bin" ]]; then
        continue
      fi
      echo "[lab] building $case_name / $variant" >&2
      (cd "$case_dir" && flutter build apk --release --target-platform "$abi" "${extra[@]+"${extra[@]}"}") \
        || { echo "build failed: $case_name/$variant" >&2; continue; }
      apk_src="$case_dir/build/app/outputs/flutter-apk/app-release.apk"
      [[ -f "$apk_src" ]] || continue
      # Do NOT pre-create out_dir: Clutter refuses to write into a directory
      # that is not a recognizable (or absent) output. Logs go beside it.
      mkdir -p "$(dirname "$out_dir")"
      "$clutter_bin" decompile "$apk_src" --out "$out_dir" --emit-ir --replace \
        >"$lab_out/$case_name/$variant.decompile.log" 2>&1 || {
          echo "clutter failed on $case_name/$variant" >&2
          continue
        }
      # Record one matrix row per successful build+decompile.
      $first_row || echo ',' >> "$matrix"
      first_row=false
      python3 - "$case_name" "$variant" "$out_dir" <<'PYEOF' >> "$matrix"
import json, sys, pathlib
case, variant, out = sys.argv[1], sys.argv[2], pathlib.Path(sys.argv[3])
manifest = json.loads((out / "decompilation.json").read_text())
row = {
    "case": case,
    "variant": variant,
    "functions": manifest["coverage"]["recovered_functions"],
    "semantic_statements": manifest["coverage"]["semantic_statements"],
    "pc_descriptor_entries": manifest["coverage"]["pc_descriptor_entries"],
}
print(json.dumps(row))
PYEOF
    done
  done
done

echo ']}' >> "$matrix"

# ---------------------------------------------------------------------------
# Template mining: group IR statements by shape across variants of one case.
# A "template" is an ordered statement-shape sequence that appears in EVERY
# successful variant — that is what a lowering rule may rely on.
# ---------------------------------------------------------------------------
python3 - "$lab_out" "${wanted[@]}" <<'PYEOF'
import json, sys, pathlib, collections

lab = pathlib.Path(sys.argv[1])
wanted = sys.argv[2:]
templates_root = lab / "templates"
templates_root.mkdir(parents=True, exist_ok=True)

def shape_sequence(program):
    seq = []
    for fn in program.get("functions", []):
        for st in fn.get("statements", []):
            kind = st.get("kind", "Unknown")
            if kind == "DirectCall":
                kind = f"DirectCall[{st.get('target')}]"
            elif kind == "ObjectPoolCall":
                kind = f"ObjectPoolCall[{st.get('target')}]"
            seq.append(kind)
    return tuple(seq)

def common_prefix(seqs):
    if not seqs:
        return []
    shortest = min(seqs, key=len)
    prefix = []
    for i, value in enumerate(shortest):
        column = {seq[i] for seq in seqs}
        if len(column) == 1:
            prefix.append(value)
        else:
            break
    return prefix

summary = {}
for case in wanted:
    shapes = {}
    for variant_dir in sorted((lab / case).glob("*")):
        ir = variant_dir / "ir" / "program.json"
        if not ir.is_file():
            continue
        program = json.loads(ir.read_text())
        shapes[variant_dir.name] = shape_sequence(program)
    if len(shapes) < 2:
        summary[case] = {"variants_compared": len(shapes),
                         "note": "need >=2 variants to mine a template"}
        continue
    shared = common_prefix(list(shapes.values()))
    summary[case] = {
        "variants_compared": len(shapes),
        "template_length": len(shared),
        "template": shared[:32],
    }
    (templates_root / f"{case}.json").write_text(
        json.dumps({"schema": "clutter.lowering-template/v1",
                    "case": case, **summary[case]}, indent=2))

(lab / "templates" / "index.json").write_text(json.dumps(summary, indent=2))
print(f"[lab] mined templates for {len(summary)} case(s)")
PYEOF

# ---------------------------------------------------------------------------
# Regression fixtures: pin stable, ABI-portable facts so future Clutter
# changes that silently break recovery fail loudly in CI.
# ---------------------------------------------------------------------------
python3 - "$lab_out" "${wanted[@]}" <<'PYEOF'
import json, sys, pathlib
lab = pathlib.Path(sys.argv[1])
wanted = sys.argv[2:]
fixtures = {"schema": "clutter.lab-regressions/v1", "cases": {}}
for case in wanted:
    rows = []
    matrix = lab / "matrix.json"
    try:
        data = json.loads(matrix.read_text())
        rows = [r for r in data["rows"] if r["case"] == case]
    except Exception:
        pass
    if not rows:
        continue
    fixtures["cases"][case] = {
        "variants": [r["variant"] for r in rows],
        "min_semantic_statements": min(r["semantic_statements"] for r in rows),
        "min_functions": min(r["functions"] for r in rows),
    }
out = lab / "fixtures"
out.mkdir(exist_ok=True)
(out / "regressions.json").write_text(json.dumps(fixtures, indent=2))
print("[lab] regression fixtures written")
PYEOF

echo "[lab] complete → $lab_out"
