// Adversarial constructs for the Clutter recovery test apps.
//
// Each section models something an app author could do to reduce what a
// decompiler recovers, or to break the pseudocode it emits. Everything is
// reachable from `runHardening`, which the UI calls with a runtime seed, so
// tree shaking and constant folding keep it.
//
// ignore_for_file: camel_case_types, non_constant_identifier_names

import 'dart:async';
import 'dart:typed_data';

// Libraries whose output paths collide with SDK libraries or with each
// other by case alone.
import 'Case.dart';
import 'case.dart';
import 'dart/core/bool.dart';
import 'packages/flutter/src/widgets/framework.dart';

// --- 1. Output injection ---------------------------------------------------
//
// Literals aimed at emitters that paste strings into comments or quotes.
// A correct decompiler escapes all of them; none may turn into code.

const List<String> hostileLiterals = [
  "quote ' and \" and \\ backslash",
  r'dollar $notInterpolated ${alsoNot}',
  'line one\nline two\r\n*/ /* // closing comments',
  '\u202Eevil\u202C reversed (Trojan Source)',
  'nul \u0000 bell \u0007 esc \u001B',
  "'); injectedCanary(); ('",
  '*/ injectedCanary(); /*',
  '\n}\nvoid injectedCanary() {}\n',
  'lone surrogate \uD800 end',
  'emoji \u{1F47E}',
];

int hostileScore(String input) {
  var score = 0;
  for (final literal in hostileLiterals) {
    if (input.contains(literal)) {
      score++;
    }
  }
  if (input == '*/ injectedCanary(); /*') {
    score += 100;
  }
  final framed = '<<$input\n*/ \${literal} >>';
  return score + framed.length;
}

/// Longer than the 160-character pool-label cut, with a bidi override, so
/// the interpolation builder sees an abbreviated label.
const String longHostile =
    'This literal is deliberately longer than the abbreviation limit used by '
    'decompilers for pool labels, and it hides a \u202E bidi override plus a '
    'quote \' and a dollar \$ near its end.';

@pragma('vm:never-inline')
String framedLong(int seed) => '[$seed] $longHostile [$seed]';

/// A literal holding the comparison separator a text IR might use.
@pragma('vm:never-inline')
bool questionMark(String input) => input == 'a ? b' || input.length > 3;

/// A string that looks like an ICData selector label.
@pragma('vm:never-inline')
dynamic forgedSelector(dynamic target) =>
    target is Shouter
        ? (target as dynamic).transform("dynamicCall('delete')")
        : null;

/// Comparisons of comparisons: Dart does not chain `==`.
@pragma('vm:never-inline')
bool comparisonOfComparisons(int a, int b, int c, int d) =>
    (a == b) != (c == d);

/// Nested generic type arguments in a constant.
const Map<String, List<String>> routes = {
  'home': ['/', '/index'],
  'shop': ['/catalog'],
};
const Map<String, List<String>> emptyRoutes = <String, List<String>>{};

// --- 2. Collisions with the decompiler's own vocabulary ---------------------
//
// Names Clutter uses for synthetic output (`aot`, `Context`, `_Closure`,
// `captured0`, `sub_<addr>`, `native`, `_slot_<n>`) declared as real code.

class Context {
  final int captured0;
  Context? parent;
  final int _slot_8 = 8;

  Context(this.captured0);

  int depth() => parent == null ? _slot_8 : 1 + parent!.depth();
}

class _Closure {
  final Object? _context;
  const _Closure(this._context);

  String describe() => '_Closure($_context)';
}

@pragma('vm:never-inline')
int aot(int x) => x * 3;
@pragma('vm:never-inline')
int sub_1000(int arg0) => arg0 + 1;
@pragma('vm:never-inline')
String native(String closureContext) => closureContext.toUpperCase();

// Register- and spill-shaped names that pseudocode sanitizers rewrite.
@pragma('vm:never-inline')
int x1(int value) => value ^ 0x55;
@pragma('vm:never-inline')
int r2(int value) => value - 2;

class $Weird$ {
  int $$ = 1;
  int bump() => $$ += 2;
}

// --- 3. Control-flow flattening and opaque predicates -----------------------

int flattened(int seed) {
  var state = 7;
  var acc = seed;
  var rounds = 0;
  while (true) {
    switch (state) {
      case 7:
        acc = acc * 31 + 3;
        state = (acc & 1) == 0 ? 12 : 3;
      case 12:
        acc ^= 0x5a5a;
        state = 3;
      case 3:
        // Squares are never 2 mod 4: the branch to 99 is dead.
        state = (acc * acc) % 4 == 2 ? 99 : 5;
      case 5:
        acc = acc >> 1;
        rounds++;
        state = rounds < 4 ? 7 : 42;
      case 42:
        return acc;
      case 99:
        return -1;
      default:
        throw StateError('unreachable $state');
    }
  }
}

/// A dense integer switch the compiler may lower to a jump table.
String opcodeName(int op) => switch (op) {
  0 => 'nop',
  1 => 'push',
  2 => 'pop',
  3 => 'add',
  4 => 'sub',
  5 => 'mul',
  6 => 'div',
  7 => 'jmp',
  8 => 'jz',
  9 => 'call',
  10 => 'ret',
  11 => 'load',
  12 => 'store',
  13 => 'dup',
  14 => 'swap',
  15 => 'halt',
  _ => 'bad',
};

// --- 4. Encrypted strings ----------------------------------------------------

const List<int> _sealedEndpoint = [
  52, 23, 30, 1, 11, 69, 169, 162, 245, 235, 203, 135, 213, 207, 223, 168, //
  188, 191, 191, 207, 129, 129, 128, 156, 104, 98, 118, 54, 86, 22, 1, 70, //
  89, 32, 56, 52, 44, 114, 18, 2, 31, 30, 236,
];

String unseal(List<int> data, int key) {
  final out = Uint8List(data.length);
  for (var i = 0; i < data.length; i++) {
    out[i] = data[i] ^ ((key + i * 7) & 0xff);
  }
  return String.fromCharCodes(out);
}

// --- 5. Indirection -----------------------------------------------------------

typedef BinaryOp = int Function(int, int);

final Map<String, BinaryOp> _ops = {
  'add': (a, b) => a + b,
  'mul': (a, b) => a * b,
  'xor': (a, b) => a ^ b,
};

int viaTable(String name, int a, int b) => (_ops[name] ?? (x, y) => 0)(a, b);

class Doubler {
  int transform(int v) => v * 2;
}

class Shouter {
  String transform(String v) => v.toUpperCase();
}

dynamic viaDynamic(dynamic target, dynamic argument) =>
    target.transform(argument);

int viaApply(Function f, List<Object?> arguments) =>
    Function.apply(f, arguments) as int;

class Ghost {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      invocation.memberName.toString().length;
}

class Callable {
  int call(int x) => x + 100;
}

// --- 6. Sealed hierarchies, patterns and records -----------------------------

sealed class Shape {}

final class Circle extends Shape {
  final double r;
  Circle(this.r);
}

final class Rect extends Shape {
  final double w;
  final double h;
  Rect(this.w, this.h);
}

double area(Shape shape) => switch (shape) {
  Circle(r: final r) => 3.14159 * r * r,
  Rect(:final w, :final h) when w == h => w * w,
  Rect(:final w, :final h) => w * h,
};

(int, String, {bool even}) describe(int v) => (v, 'v$v', even: v.isEven);

// --- 7. Extension types, mixins and late state --------------------------------

extension type const Cents(int value) {
  Cents operator +(Cents other) => Cents(value + other.value);
  String get label =>
      '${value ~/ 100}.${(value % 100).toString().padLeft(2, '0')}';
}

mixin Audited {
  final List<String> log = [];
  void audit(String message) => log.add(message);
}

class Vault with Audited {
  late final String secret = unseal(_sealedEndpoint, 0x5c);
  int _opens = 0;

  String open(int pin) {
    if (pin != 4242) {
      audit('denied $pin');
      throw StateError('denied');
    }
    _opens++;
    return secret;
  }

  int get opens => _opens;
}

// --- 8. Exceptions as control flow ---------------------------------------------

int _finallyCount = 0;

int exceptional(int v) {
  try {
    if (v.isOdd) {
      throw FormatException('odd', v);
    }
    return v;
  } on FormatException catch (e, stack) {
    return (e.offset ?? 0) + stack.toString().length.sign;
  } finally {
    _finallyCount++;
  }
}

// --- 9. Deep, mutating closures --------------------------------------------------

int deepClosures(int seed) {
  var a = seed;
  int level1(int b) {
    final c = a + b;
    int level2(int d) {
      final e = c * d;
      int level3(int f) {
        a++;
        return e + f + a;
      }

      return level3(e);
    }

    return level2(c);
  }

  return level1(3) + a;
}

// --- 10. Generators and async streams ------------------------------------------

Iterable<int> fib(int n) sync* {
  var a = 0;
  var b = 1;
  for (var i = 0; i < n; i++) {
    yield a;
    (a, b) = (b, a + b);
  }
}

Stream<int> ticks(int n) async* {
  for (var i = 0; i < n; i++) {
    await Future<void>.delayed(Duration.zero);
    yield i * i;
  }
}

Future<int> sumTicks(int n) async {
  var total = 0;
  try {
    await for (final value in ticks(n)) {
      total += value;
    }
  } finally {
    total++;
  }
  return total;
}

// --- 11. Numeric edge values ------------------------------------------------------

String numericEdges(int seed) {
  const big = 0x7fffffffffffffff;
  final negativeZero = -0.0 * seed.sign;
  final wide = Int64List.fromList([big, -big - 1, seed]);
  final simd = Float32x4(1.5, -2.5, double.infinity, seed.toDouble());
  return '${wide.reduce((a, b) => a ^ b)} ${negativeZero.isNegative} '
      '${simd.x + simd.w} ${double.nan.isNaN} ${BigInt.from(seed).pow(20)}';
}

// --- 12. Resource exhaustion --------------------------------------------------------

int ackermann(int m, int n) {
  if (m == 0) {
    return n + 1;
  }
  if (n == 0) {
    return ackermann(m - 1, 1);
  }
  return ackermann(m - 1, ackermann(m, n - 1));
}

/// Register-pressure body: 40 live integers across three nested loops.
/// Aims at dataflow visit budgets and spill-slot tracking.
int registerPressure(int seed) {
  var v0 = seed + 0, v1 = seed + 1, v2 = seed + 2, v3 = seed + 3, v4 = seed + 4, v5 = seed + 5, v6 = seed + 6, v7 = seed + 7, v8 = seed + 8, v9 = seed + 9, v10 = seed + 10, v11 = seed + 11, v12 = seed + 12, v13 = seed + 13, v14 = seed + 14, v15 = seed + 15, v16 = seed + 16, v17 = seed + 17, v18 = seed + 18, v19 = seed + 19, v20 = seed + 20, v21 = seed + 21, v22 = seed + 22, v23 = seed + 23, v24 = seed + 24, v25 = seed + 25, v26 = seed + 26, v27 = seed + 27, v28 = seed + 28, v29 = seed + 29, v30 = seed + 30, v31 = seed + 31, v32 = seed + 32, v33 = seed + 33, v34 = seed + 34, v35 = seed + 35, v36 = seed + 36, v37 = seed + 37, v38 = seed + 38, v39 = seed + 39;
  for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 2; j++) {
      for (var k = 0; k < 2; k++) {
        v0 = (v0 + v3) & 0xffff;
        if (v0 > v5) { v5 += 0; } else { v0 -= v5 & 7; }
        v1 = (v1 ^ v10) & 0xffff;
        v2 = (v2 - v17) & 0xffff;
        v3 = (v3 * v24) & 0xffff;
        v4 = (v4 + v31) & 0xffff;
        v5 = (v5 ^ v38) & 0xffff;
        if (v5 > v30) { v30 += 5; } else { v5 -= v30 & 7; }
        v6 = (v6 - v5) & 0xffff;
        v7 = (v7 * v12) & 0xffff;
        v8 = (v8 + v19) & 0xffff;
        v9 = (v9 ^ v26) & 0xffff;
        v10 = (v10 - v33) & 0xffff;
        if (v10 > v15) { v15 += 10; } else { v10 -= v15 & 7; }
        v11 = (v11 * v0) & 0xffff;
        v12 = (v12 + v7) & 0xffff;
        v13 = (v13 ^ v14) & 0xffff;
        v14 = (v14 - v21) & 0xffff;
        v15 = (v15 * v28) & 0xffff;
        if (v15 > v0) { v0 += 15; } else { v15 -= v0 & 7; }
        v16 = (v16 + v35) & 0xffff;
        v17 = (v17 ^ v2) & 0xffff;
        v18 = (v18 - v9) & 0xffff;
        v19 = (v19 * v16) & 0xffff;
        v20 = (v20 + v23) & 0xffff;
        if (v20 > v25) { v25 += 20; } else { v20 -= v25 & 7; }
        v21 = (v21 ^ v30) & 0xffff;
        v22 = (v22 - v37) & 0xffff;
        v23 = (v23 * v4) & 0xffff;
        v24 = (v24 + v11) & 0xffff;
        v25 = (v25 ^ v18) & 0xffff;
        if (v25 > v10) { v10 += 25; } else { v25 -= v10 & 7; }
        v26 = (v26 - v25) & 0xffff;
        v27 = (v27 * v32) & 0xffff;
        v28 = (v28 + v39) & 0xffff;
        v29 = (v29 ^ v6) & 0xffff;
        v30 = (v30 - v13) & 0xffff;
        if (v30 > v35) { v35 += 30; } else { v30 -= v35 & 7; }
        v31 = (v31 * v20) & 0xffff;
        v32 = (v32 + v27) & 0xffff;
        v33 = (v33 ^ v34) & 0xffff;
        v34 = (v34 - v1) & 0xffff;
        v35 = (v35 * v8) & 0xffff;
        if (v35 > v20) { v20 += 35; } else { v35 -= v20 & 7; }
        v36 = (v36 + v15) & 0xffff;
        v37 = (v37 ^ v22) & 0xffff;
        v38 = (v38 - v29) & 0xffff;
        v39 = (v39 * v36) & 0xffff;
      }
    }
  }
  return v0 ^ v1 ^ v2 ^ v3 ^ v4 ^ v5 ^ v6 ^ v7 ^ v8 ^ v9 ^ v10 ^ v11 ^ v12 ^ v13 ^ v14 ^ v15 ^ v16 ^ v17 ^ v18 ^ v19 ^ v20 ^ v21 ^ v22 ^ v23 ^ v24 ^ v25 ^ v26 ^ v27 ^ v28 ^ v29 ^ v30 ^ v31 ^ v32 ^ v33 ^ v34 ^ v35 ^ v36 ^ v37 ^ v38 ^ v39;
}

/// A 300-entry constant map: stresses constant-pool decoding and rendering.
const Map<String, int> largeTable = {
  'key_000': 0,
  'key_001': 56132,
  'key_002': 12261,
  'key_003': 68393,
  'key_004': 24522,
  'key_005': 80654,
  'key_006': 36783,
  'key_007': 92915,
  'key_008': 49044,
  'key_009': 5173,
  'key_010': 61305,
  'key_011': 17434,
  'key_012': 73566,
  'key_013': 29695,
  'key_014': 85827,
  'key_015': 41956,
  'key_016': 98088,
  'key_017': 54217,
  'key_018': 10346,
  'key_019': 66478,
  'key_020': 22607,
  'key_021': 78739,
  'key_022': 34868,
  'key_023': 91000,
  'key_024': 47129,
  'key_025': 3258,
  'key_026': 59390,
  'key_027': 15519,
  'key_028': 71651,
  'key_029': 27780,
  'key_030': 83912,
  'key_031': 40041,
  'key_032': 96173,
  'key_033': 52302,
  'key_034': 8431,
  'key_035': 64563,
  'key_036': 20692,
  'key_037': 76824,
  'key_038': 32953,
  'key_039': 89085,
  'key_040': 45214,
  'key_041': 1343,
  'key_042': 57475,
  'key_043': 13604,
  'key_044': 69736,
  'key_045': 25865,
  'key_046': 81997,
  'key_047': 38126,
  'key_048': 94258,
  'key_049': 50387,
  'key_050': 6516,
  'key_051': 62648,
  'key_052': 18777,
  'key_053': 74909,
  'key_054': 31038,
  'key_055': 87170,
  'key_056': 43299,
  'key_057': 99431,
  'key_058': 55560,
  'key_059': 11689,
  'key_060': 67821,
  'key_061': 23950,
  'key_062': 80082,
  'key_063': 36211,
  'key_064': 92343,
  'key_065': 48472,
  'key_066': 4601,
  'key_067': 60733,
  'key_068': 16862,
  'key_069': 72994,
  'key_070': 29123,
  'key_071': 85255,
  'key_072': 41384,
  'key_073': 97516,
  'key_074': 53645,
  'key_075': 9774,
  'key_076': 65906,
  'key_077': 22035,
  'key_078': 78167,
  'key_079': 34296,
  'key_080': 90428,
  'key_081': 46557,
  'key_082': 2686,
  'key_083': 58818,
  'key_084': 14947,
  'key_085': 71079,
  'key_086': 27208,
  'key_087': 83340,
  'key_088': 39469,
  'key_089': 95601,
  'key_090': 51730,
  'key_091': 7859,
  'key_092': 63991,
  'key_093': 20120,
  'key_094': 76252,
  'key_095': 32381,
  'key_096': 88513,
  'key_097': 44642,
  'key_098': 771,
  'key_099': 56903,
  'key_100': 13032,
  'key_101': 69164,
  'key_102': 25293,
  'key_103': 81425,
  'key_104': 37554,
  'key_105': 93686,
  'key_106': 49815,
  'key_107': 5944,
  'key_108': 62076,
  'key_109': 18205,
  'key_110': 74337,
  'key_111': 30466,
  'key_112': 86598,
  'key_113': 42727,
  'key_114': 98859,
  'key_115': 54988,
  'key_116': 11117,
  'key_117': 67249,
  'key_118': 23378,
  'key_119': 79510,
  'key_120': 35639,
  'key_121': 91771,
  'key_122': 47900,
  'key_123': 4029,
  'key_124': 60161,
  'key_125': 16290,
  'key_126': 72422,
  'key_127': 28551,
  'key_128': 84683,
  'key_129': 40812,
  'key_130': 96944,
  'key_131': 53073,
  'key_132': 9202,
  'key_133': 65334,
  'key_134': 21463,
  'key_135': 77595,
  'key_136': 33724,
  'key_137': 89856,
  'key_138': 45985,
  'key_139': 2114,
  'key_140': 58246,
  'key_141': 14375,
  'key_142': 70507,
  'key_143': 26636,
  'key_144': 82768,
  'key_145': 38897,
  'key_146': 95029,
  'key_147': 51158,
  'key_148': 7287,
  'key_149': 63419,
  'key_150': 19548,
  'key_151': 75680,
  'key_152': 31809,
  'key_153': 87941,
  'key_154': 44070,
  'key_155': 199,
  'key_156': 56331,
  'key_157': 12460,
  'key_158': 68592,
  'key_159': 24721,
  'key_160': 80853,
  'key_161': 36982,
  'key_162': 93114,
  'key_163': 49243,
  'key_164': 5372,
  'key_165': 61504,
  'key_166': 17633,
  'key_167': 73765,
  'key_168': 29894,
  'key_169': 86026,
  'key_170': 42155,
  'key_171': 98287,
  'key_172': 54416,
  'key_173': 10545,
  'key_174': 66677,
  'key_175': 22806,
  'key_176': 78938,
  'key_177': 35067,
  'key_178': 91199,
  'key_179': 47328,
  'key_180': 3457,
  'key_181': 59589,
  'key_182': 15718,
  'key_183': 71850,
  'key_184': 27979,
  'key_185': 84111,
  'key_186': 40240,
  'key_187': 96372,
  'key_188': 52501,
  'key_189': 8630,
  'key_190': 64762,
  'key_191': 20891,
  'key_192': 77023,
  'key_193': 33152,
  'key_194': 89284,
  'key_195': 45413,
  'key_196': 1542,
  'key_197': 57674,
  'key_198': 13803,
  'key_199': 69935,
  'key_200': 26064,
  'key_201': 82196,
  'key_202': 38325,
  'key_203': 94457,
  'key_204': 50586,
  'key_205': 6715,
  'key_206': 62847,
  'key_207': 18976,
  'key_208': 75108,
  'key_209': 31237,
  'key_210': 87369,
  'key_211': 43498,
  'key_212': 99630,
  'key_213': 55759,
  'key_214': 11888,
  'key_215': 68020,
  'key_216': 24149,
  'key_217': 80281,
  'key_218': 36410,
  'key_219': 92542,
  'key_220': 48671,
  'key_221': 4800,
  'key_222': 60932,
  'key_223': 17061,
  'key_224': 73193,
  'key_225': 29322,
  'key_226': 85454,
  'key_227': 41583,
  'key_228': 97715,
  'key_229': 53844,
  'key_230': 9973,
  'key_231': 66105,
  'key_232': 22234,
  'key_233': 78366,
  'key_234': 34495,
  'key_235': 90627,
  'key_236': 46756,
  'key_237': 2885,
  'key_238': 59017,
  'key_239': 15146,
  'key_240': 71278,
  'key_241': 27407,
  'key_242': 83539,
  'key_243': 39668,
  'key_244': 95800,
  'key_245': 51929,
  'key_246': 8058,
  'key_247': 64190,
  'key_248': 20319,
  'key_249': 76451,
  'key_250': 32580,
  'key_251': 88712,
  'key_252': 44841,
  'key_253': 970,
  'key_254': 57102,
  'key_255': 13231,
  'key_256': 69363,
  'key_257': 25492,
  'key_258': 81624,
  'key_259': 37753,
  'key_260': 93885,
  'key_261': 50014,
  'key_262': 6143,
  'key_263': 62275,
  'key_264': 18404,
  'key_265': 74536,
  'key_266': 30665,
  'key_267': 86797,
  'key_268': 42926,
  'key_269': 99058,
  'key_270': 55187,
  'key_271': 11316,
  'key_272': 67448,
  'key_273': 23577,
  'key_274': 79709,
  'key_275': 35838,
  'key_276': 91970,
  'key_277': 48099,
  'key_278': 4228,
  'key_279': 60360,
  'key_280': 16489,
  'key_281': 72621,
  'key_282': 28750,
  'key_283': 84882,
  'key_284': 41011,
  'key_285': 97143,
  'key_286': 53272,
  'key_287': 9401,
  'key_288': 65533,
  'key_289': 21662,
  'key_290': 77794,
  'key_291': 33923,
  'key_292': 90055,
  'key_293': 46184,
  'key_294': 2313,
  'key_295': 58445,
  'key_296': 14574,
  'key_297': 70706,
  'key_298': 26835,
  'key_299': 82967,
};

// --- Entry point ---------------------------------------------------------------------

String runHardening(int seed) {
  final context = Context(seed)..parent = Context(seed + 1);
  final vault = Vault();
  var opened = '';
  try {
    opened = vault.open(seed == -1 ? 0 : 4242);
  } on StateError {
    opened = 'locked';
  }
  final dynamic ghost = Ghost();
  final record = describe(seed);
  final shapes = <Shape>[Circle(seed.toDouble()), Rect(2, 2), Rect(2, 3)];
  final parts = <Object?>[
    hostileScore(hostileLiterals[seed % hostileLiterals.length]),
    context.depth(),
    const _Closure('ctx').describe(),
    aot(seed) + sub_1000(seed),
    native('closure'),
    $Weird$().bump(),
    flattened(seed),
    opcodeName(seed % 17),
    opened,
    viaTable(seed.isEven ? 'add' : 'xor', seed, 7),
    viaDynamic(seed.isEven ? Doubler() : Shouter(), seed.isEven ? seed : 'x'),
    viaApply(ackermann, [2, seed % 3]),
    ghost.anything(seed),
    Callable()(seed),
    shapes.map(area).reduce((a, b) => a + b).toStringAsFixed(2),
    '${record.$1}/${record.$2}/${record.even}',
    (const Cents(150) + Cents(seed)).label,
    exceptional(seed) + _finallyCount,
    deepClosures(seed),
    fib(seed % 12 + 1).last,
    numericEdges(seed),
    registerPressure(seed),
    largeTable['key_${(seed % 300).toString().padLeft(3, '0')}'],
    x1(seed) + r2(seed),
    framedLong(seed).length,
    questionMark(seed.isEven ? 'a ? b' : 'x'),
    forgedSelector(seed.isEven ? Shouter() : Object()),
    comparisonOfComparisons(seed, 1, seed, 2),
    (seed.isEven ? routes : emptyRoutes).length,
    fakeCore(seed),
    fakeWidgets(seed),
    upperCase(seed) + lowerCase(seed),
  ];
  unawaited(sumTicks(seed % 5).then((value) => parts.add(value)));
  return parts.join(' | ');
}
