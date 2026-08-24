import 'dart:async';
import 'dart:collection';
import 'dart:convert';

// ============================================================
// Every function below is a deliberate decompiler stress case.
// Each is kept small and self-contained so a failure in the
// recovered output can be attributed to exactly one construct.
// ============================================================

/// E01 — deeply chained string interpolation with mixed operand kinds.
String e01InterpChain(String name, int id, double ratio) {
  return 'user=$name id=${id + 1} pct=${(ratio * 100).toStringAsFixed(1)}% '
      'nested=${'inner-$name'} bool=${id > 10} nullish=${null}';
}

/// E02 — cascades lower to a sequence of receiver-slot stores.
List<String> e02Cascade(List<String> items) {
  final out = <String>[]
    ..addAll(items)
    ..sort()
    ..add('done');
  return out;
}

/// E03 — switch on strings with fallthrough-free cases and default.
String e03StringSwitch(String code) {
  switch (code) {
    case 'a':
      return 'alpha';
    case 'b':
    case 'c':
      return 'beta-or-gamma';
    default:
      return 'other';
  }
}

/// E04 — integer bit twiddling: shifts, masks, signed division.
int e04BitTwiddle(int x) {
  final y = (x << 3) & 0xFF;
  final z = (x >> 2) | (x & 0x0F);
  final w = x ~/ 7;
  final u = x.remainder(-3);
  return y ^ z ^ w ^ u;
}

/// E05 — nullable flow: ??, ?., ?[], late field via local, null-aware spread.
String? e05NullFlow(Map<String, String?> m, String key) {
  final v = m[key];
  final other = <String>['x', ...?v?.split('')];
  final fallback = v ?? other.join('-');
  return m['missing'] ??= fallback;
}

/// E06 — typed record with nested destructuring in a for-in pattern.
int e06RecordDestructure((int, int, {String tag}) rec) {
  var total = 0;
  final pairs = <(int, int)>[(1, 2), (3, 4)];
  for (final (a, b) in pairs) {
    total += a * b;
  }
  final (p, q, :tag) = rec;
  return total + p - q + tag.length;
}

/// E07 — generic method with F-bounded type parameter and explicit instantiate.
Map<String, num> e07GenericBound<T extends num, U extends T>(
    Map<String, U> src) {
  return src.map((k, v) => MapEntry(k, v + 1));
}

/// E08 — labeled break/continue inside nested loops.
int e08LabeledLoops(int n) {
  var hits = 0;
  outer:
  for (var i = 0; i < n; i++) {
    for (var j = 0; j < n; j++) {
      if (i * j > 6) continue outer;
      if (i + j == n) break outer;
      hits++;
    }
    hits += 100;
  }
  return hits;
}

/// E09 — try/on/finally with rethrow and exception variable shadowing.
String e09TryRethrow(Object input) {
  var log = '';
  try {
    log += 't';
    if (input is! int) throw FormatException('bad');
    return 'ok:$input';
  } on FormatException catch (e) {
    log += e.message;
    rethrow;
  } catch (e, s) {
    log += 'fallback${s.hashCode}';
    return log;
  } finally {
    log += 'f';
  }
}

/// E10 — async loop with early return across awaits (state-machine hops).
Future<int> e10AsyncLoop(int upto) async {
  var acc = 0;
  for (var i = 0; i < upto; i++) {
    acc += await Future<int>.value(i);
    if (acc > 100) return acc;
  }
  await Future<void>.delayed(Duration.zero);
  return acc;
}

/// E11 — sync* generator with yield/yield*/continue.
Iterable<int> e11SyncGen(int n) sync* {
  for (var i = 0; i < n; i++) {
    if (i.isEven) yield i;
    else yield* e11SyncGen(i);
  }
}

/// E12 — tear-offs of constructors, getters, and operators.
List<Object> e12TearOffs() {
  final list = <int>[3, 1];
  return [
    list.length,
    list.first,
    list.sort,
    list.removeLast,
    DateTime.now,
    Duration.new,
    List.filled,
  ];
}

/// E13 — dynamic dispatch through Object members and noSuchMethod override.
class E13Dynamic {
  @override
  dynamic noSuchMethod(Invocation inv) =>
      'unhandled:${inv.memberName}:${inv.positionalArguments.length}';

  String probe(dynamic d) => d.unknownMember;
}

/// E14 — deferred static state: static fields with initializers + lazy init.
class E14Statics {
  static const version = '1.0';
  static final stamp = DateTime.now().microsecondsSinceEpoch;
  static int counter = 0;

  static int bump() => ++counter + stamp % 2 + version.length;
}

/// E15 — operator overloading family including == / hashCode / compareTo.
class E15Vec implements Comparable<E15Vec> {
  final int dx, dy;
  const E15Vec(this.dx, this.dy);

  E15Vec operator -() => E15Vec(-dx, -dy);
  E15Vec operator %(E15Vec o) => E15Vec(dx % o.dx, dy % o.dy);
  bool operator ==(Object other) => other is E15Vec && dx == other.dx;
  int get hashCode => dx ^ dy;
  @override
  int compareTo(E15Vec other) => (dx * dx + dy * dy)
      .compareTo(other.dx * other.dx + other.dy * other.dy);
}

/// E16 — SplayTree custom comparator closure + Set identity semantics.
List<String> e16SortedCopy(Set<String> s, int Function(String, String)? cmp) {
  final t = SplayTreeSet<String>(cmp);
  t.addAll(s);
  return t.toList(growable: false);
}

/// E17 — JSON round-trip forcing runtime casts and type checks.
List<Map<String, Object?>> e17JsonRoundTrip(String raw) {
  final decoded = jsonDecode(raw) as List<dynamic>;
  return decoded
      .whereType<Map<dynamic, dynamic>>()
      .map((m) => m.cast<String, Object?>())
      .toList();
}

/// E18 — numeric tower edges: BigInt, doubles as int, NaN paths.
String e18NumericEdges(double d) {
  if (d.isNaN) return 'nan';
  if (d == d.truncateToDouble()) return 'integral:${d.toInt()}';
  final b = BigInt.from(d);
  return 'frac:${b.toRadixString(16)}:${d.toStringAsExponential(2)}';
}

/// E19 — deep recursion the tree shaker must keep (entry-point anchored).
@pragma('vm:entry-point')
int e19Ackermann(int m, [int n = 2]) {
  if (m == 0) return n + 1;
  if (n == 0) return e19Ackermann(m - 1);
  return e19Ackermann(m - 1, e19Ackermann(m, n - 1));
}

/// E20 — mixin application order with super forwards and abstract overrides.
mixin Loud {
  String shout(String w) => w.toUpperCase();
}

mixin Counter on Object {
  int _n = 0;
  int next() => ++_n;
}

class E20Base {
  String greet() => 'base';
}

class E20Combo extends E20Base with Counter, Loud {
  @override
  String greet() => '${super.greet()}+combo';
}

/// E21 — enum with methods, enhanced members, and values/name usage.
enum E21Mode {
  idle(0),
  busy(2),
  error(-1);

  const E21Mode(this.severity);

  final int severity;

  bool get isBad => this == E21Mode.error || severity < 0;

  static E21Mode parse(String s) => E21Mode.values.byName(s);
}

/// E22 — extension on nullable type + generic extension with type test.
extension E22Ext<T extends Object> on Iterable<T>? {
  int get safeLen => this?.length ?? -1;
}

/// E23 — Function.apply with symbols and optional positional tail.
String e23DynamicApply(Function f, List<dynamic> args,
        [Map<Symbol, dynamic> named = const {}]) =>
    Function.apply(f, args, named).toString();

/// E24 — control-flow knot: conditionals returning from all arms plus
/// an unreachable-after-return statement the structurer must not emit.
int e24Knot(bool a, bool b, int x) {
  if (a) {
    if (b) return 1;
    return 2;
  } else if (x < 0) {
    while (true) {
      if (x.isEven) return 3;
      x = -x;
    }
  }
  do {
    x >>= 1;
  } while (x > 4);
  return x;
}

/// E25 — string operations that compile to VM intrinsics.
bool e25Intrinsics(String a, String b) {
  final c = a + b;
  if (c.isEmpty) return false;
  if (!c.startsWith('k')) return false;
  return c.codeUnitAt(0) == 107 && identical(a, b);
}
