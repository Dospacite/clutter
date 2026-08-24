// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.
// ignore_for_file: unused_element, unused_import, non_constant_identifier_names
// Recovered from: package:edge_probe/probe_code.dart

import '../support/aot_intrinsics.dart' as aot;

const String recoveredSourceUri = 'package:edge_probe/probe_code.dart';
/// Dart VM retained declaration object unknown; no distinct executable body survived.
int e08LabeledLoops() => throw UnsupportedError('AOT body unavailable');

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 199.
int e19Ackermann(int arg0, [int arg1 /* default unavailable */]) {
  if (x1 < 2) {  /* low-confidence predicate */
    return (2 + 1);
  } else {
  }
  if (2 != 0) {  /* low-confidence predicate */
    final e19AckermannResult = e19Ackermann(aot.snapshotRef(23), (2 - 1));
    final e19AckermannResult2 =
      e19Ackermann(local8, e19AckermannResult, aot.snapshotRef(23), local8, e19AckermannResult);
    return e19AckermannResult2;
  } else {
    final e19AckermannResult3 = e19Ackermann(aot.snapshotRef(22));
    return e19AckermannResult3;
  }
  // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 196.
int e19Ackermann_tearOff(int arg0, [int arg1 /* default unavailable */]) {
  if (x1 < 2) {  /* low-confidence predicate */
    final e19AckermannResult = e19Ackermann(2, aot.snapshotRef(23), 2);
    return e19AckermannResult;
  } else {
  }
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 267.
bool e25Intrinsics() {
  // Recovered source literals:
  //   line 267: 'ey'
  final combinedResult = aot.invoke('String.+', <dynamic>[aot.snapshotRef(870), 'ey']);
  return false;
  // 5 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E13Dynamic`.
dynamic package_edge_probe_probe_code_dart_E13Dynamic(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E13Dynamic', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 245.
dynamic e23DynamicApply(List<dynamic> args) {
  final applyResult = apply();
  if ((applyResult & 1) == 0) {  /* low-confidence predicate */
  } else {
  }
  // Dynamic-call evidence:
  //   .toString(...) at 1 site(s): 76 candidate implementation(s), e.g. Abi.toString, ArgumentError.toString, AssertionError.toString, AsyncError.toString.
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e23DynamicApply', <dynamic>[]);
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E20Combo`.
dynamic package_edge_probe_probe_code_dart_E20Combo(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E20Combo', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 190.
/// Inlined by the optimizer (statements live inside this body):
///   BigInt.from
dynamic e18NumericEdges(List<dynamic> args) {
  // Recovered source literals:
  //   line 190: 'integral:'
  //   line 192: 'frac:'
  if (2.5 != 2.5) {
    final bigIntImpl = _BigIntImpl.from(null);
    final toRadixStringResult = aot.invoke('_BigIntImpl.toRadixString', <dynamic>[bigIntImpl, 16]);
    final toStringAsExponentialResult =
      aot.invoke('_Double.toStringAsExponential', <dynamic>[aot.snapshotRef(15147), 2]);
    return 'frac:${toRadixStringResult}${aot.unresolvedValue('interpolated part')}${toStringAsExponentialResult}';
  } else {
    if (x16 != x1) {  /* low-confidence predicate */
      final interpolateResult = aot.invoke('String._interpolate', <dynamic>[]);
      return interpolateResult;
    } else {
    }
  }
  // Dynamic-call evidence:
  //   2 register-indirect call site(s) remain unresolved.
  // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 180.
/// Inlined by the optimizer (statements live inside this body):
///   toList
///   List.of
dynamic e17JsonRoundTrip(List<dynamic> args) {

  /// Closure recovered from package:edge_probe/probe_code.dart near line 183.
  Map<String, Object?> closureAtLine183(Map arg0) {
    return aot.unresolvedRegion('Remaining behavior of anonymous closure', <dynamic>[]);
  }
  final jsonDecodeResult = jsonDecode();
  if ((jsonDecodeResult & 1) == 0) {  /* low-confidence predicate */
    if (x4 <= 2) {  /* low-confidence predicate */
      final mapResult =
        aot.invoke('Iterable.map', <dynamic>[aot.snapshotRef(55), aot.snapshotRef(17926), jsonDecodeResult, aot.unresolvedValue('shared-code result')]);
      final growableList = _GrowableList._of(mapResult);
      return growableList;
    } else {
      if (x4 <= 55) {  /* low-confidence predicate */
      } else {
        if (x4 == 2046) {  /* low-confidence predicate */
        } else {
          if (x4 == 2105) {  /* low-confidence predicate */
          } else {
          }
        }
      }
    }
  } else {
  }
  // Dynamic-call evidence:
  //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
  // 5 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 173.
/// Inlined by the optimizer (statements live inside this body):
///   toList
///   List.of
dynamic e16SortedCopy(List<dynamic> args) {
  final splayTreeSetResult = aot.invoke('dart:collection.SplayTreeSet', <dynamic>[aot.snapshotRef(18493), arg0]);
  final splayTreeSet = SplayTreeSet(splayTreeSetResult, null);
  final addAllResult = aot.invoke('SplayTreeSet.addAll', <dynamic>[splayTreeSetResult, arg0]);
  final list = _List._of(aot.snapshotRef(18493), splayTreeSetResult);
  return list;
  // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E15Vec`.
dynamic package_edge_probe_probe_code_dart_E15Vec(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E15Vec', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 120.
/// Inlined by the optimizer (statements live inside this body):
///   isEven
dynamic e11SyncGen(List<dynamic> args) {
  while (x4 < arg0) {
  }
  return false;
  // Statements recovered from unreached machine regions:
  final e11SyncGenResult = e11SyncGen(local18, 0);
  // 1 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e11SyncGen', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 110.
/// Inlined by the optimizer (statements live inside this body):
///   Future.value
///   _Future.immediate
dynamic e10AsyncLoop(List<dynamic> args) async {
  while (x0 < 3) {
    final futureResult = aot.invoke('dart:async._Future', <dynamic>[aot.snapshotRef(18479)]);
    if (x0 != x16) {  /* low-confidence predicate */
      final asyncCompleteResult =
        aot.invoke('_Future._asyncComplete', <dynamic>[futureResult, local18, futureResult, local18]);
      final combinedResult =
        aot.invoke('_IntegerImplementation.+', <dynamic>[aot.unresolvedValue('shared-code result'), local10, local10, aot.unresolvedValue('shared-code result')]);
      if (combinedResult > 100) {  /* low-confidence predicate */
      } else {
      }
    } else {
    }
  }
  final future = Future.delayed(aot.snapshotRef(18448), const Duration());
  // Body compiled to an async state machine; await boundaries are unnamed in this snapshot.
  // 3 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e10AsyncLoop', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 92.
/// Inlined by the optimizer (statements live inside this body):
///   hashCode
dynamic e09TryRethrow(List<dynamic> args) {
  // Recovered source literals:
  //   line 99: 'fallback'
  //   line 102: 'ok:'
  //   line 102: 'ok:3'
  final combinedResult = aot.invoke('String.+', <dynamic>[aot.snapshotRef(903), aot.snapshotRef(610)]);
  final combinedResult2 = aot.invoke('String.+', <dynamic>[combinedResult, aot.snapshotRef(109)]);
  return 'ok:3';
  // Control-flow evidence: 3 conditional branch(es), 1 loop back-edge(s), 2 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 70.
Map<String, num> e07GenericBound<T extends num, U extends T>(Map<String, U> arg0) {

  /// Closure recovered from package:edge_probe/probe_code.dart near line 70.
  MapEntry<String, num> closureAtLine70(String arg0, U arg1) {
    if ((arg1 & 1) == 0) {
      final mapEntryResult = aot.invoke('dart:core.MapEntry', <dynamic>[aot.snapshotRef(18337)]);
      return mapEntryResult;
    } else {
    }
  }
  final mapResult =
    map(aot.unresolvedValue('shared-code result'), aot.snapshotRef(34520), aot.snapshotRef(18337), arg0, aot.unresolvedValue('shared-code result'));
  return mapResult;
  // Control-flow evidence: 3 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 59.
/// Inlined by the optimizer (statements live inside this body):
///   _GrowableList._literal2
///   current
dynamic e06RecordDestructure(List<dynamic> args) {
  while (x0 < 2) {
    if (x5 != null) {  /* low-confidence predicate */
    } else {
    }
  }
  // Dynamic-call evidence:
  //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
  // 2 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e06RecordDestructure', <dynamic>[]);
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 50.
/// Inlined by the optimizer (statements live inside this body):
///   []
///   _GrowableList._literal1
dynamic e05NullFlow(List<dynamic> args) {
  // Recovered source literals:
  //   line 53: 'missing'
  final lookupResult = aot.invoke('Map.lookup', <dynamic>[arg0, aot.snapshotRef(870)]);
  if (x2 != lookupResult) {  /* low-confidence predicate */
    if (local10 != null) {  /* low-confidence predicate */
      if (x2 == null) {  /* low-confidence predicate */
        if (local10 != null) {  /* low-confidence predicate */
          final lookupResult2 = aot.invoke('Map.lookup', <dynamic>[arg0, 'missing']);
          if (x2 != lookupResult2) {  /* low-confidence predicate */
            if (x0 != null) {  /* low-confidence predicate */
            } else {
              final updatedItemResult =
                aot.invoke('__Map&_HashVMBase&MapMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashMapMixin.[]=', <dynamic>[arg0, 'missing', local10]);
            }
          } else {
          }
        } else {
          final joinResult =
            aot.invoke('List.join', <dynamic>[aot.unresolvedValue('shared-code result'), aot.snapshotRef(34387), aot.snapshotRef(272)]);
        }
      } else {
        final addAllResult = aot.invoke('List.addAll', <dynamic>[aot.unresolvedValue('shared-code result')]);
      }
    } else {
    }
  } else {
  }
  // Dynamic-call evidence:
  //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
  // 6 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e05NullFlow', <dynamic>[]);
}

/// Partially reconstructed `e04BitTwiddle`.
int e04BitTwiddle() {
  return 68;
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 19.
/// Inlined by the optimizer (statements live inside this body):
///   add
dynamic e02Cascade(List<dynamic> args) {
  // Recovered source literals:
  //   line 18: 'done'
  final growableList = <dynamic>[];
  final addAllResult = aot.invoke('List.addAll', <dynamic>[growableList, arg0]);
  final sortResult = aot.invoke('List.sort', <dynamic>[growableList, aot.snapshotRef(34439)]);
  if (x2 != x1) {  /* low-confidence predicate */
    return growableList;
  } else {
    final growToNextCapacityResult = aot.invoke('List._growToNextCapacity', <dynamic>[growableList]);
  }
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed from package:edge_probe/probe_code.dart near line 13.
dynamic e01InterpChain(List<dynamic> args) {
  // Recovered source literals:
  //   line 13: ' bool='
  //   line 13: ' id='
  //   line 13: ' nullish='
  //   line 13: ' pct='
  //   line 13: '% nested=inner-'
  //   line 13: 'user='
  final toStringAsFixedResult = aot.invoke('_Double.toStringAsFixed', <dynamic>[aot.snapshotRef(15148), 1]);
  return 'user=${snapshotRef(458)} id=${6} pct=${toStringAsFixedResult}% nested=inner-${snapshotRef(458)} bool=${false} nullish=${null}';
  // Control-flow evidence: 3 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

mixin Counter {
}

class E13Dynamic {

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 145.
  dynamic probe(List<dynamic> args) {
    // Dynamic-call evidence:
    //   1 register-indirect call site(s) remain unresolved.
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
    return aot.unresolvedRegion('Remaining behavior of E13Dynamic.probe', <dynamic>[]);
  }

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 143.
  dynamic noSuchMethod(List<dynamic> args) {
    // Recovered source literals:
    //   line 143: 'unhandled:'
    final get_memberNameResult = memberName;
    final get_positionalArgumentsResult = positionalArguments;
    return 'unhandled:${get_memberNameResult}${aot.unresolvedValue('interpolated part')}${aot.unresolvedValue('interpolated part')}';
    // Dynamic-call evidence:
    //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
    // Control-flow evidence: 5 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}

abstract class E14Statics {

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 154.
  static int bump() {
    if (x0 != x16) {  /* low-confidence predicate */
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
    return aot.unresolvedRegion('Remaining behavior of E14Statics.bump', <dynamic>[]);
  }

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 151.
  /// Inlined by the optimizer (statements live inside this body):
  ///   DateTime.now
  ///   DateTime._now
  static late final int stamp = _getCurrentMicros();
}

class E15Vec implements Comparable<E15Vec> {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  num _slot_8; // AOT slot +0x8; unboxed_field
  num _slot_c; // AOT slot +0xc; unboxed_field
  num _slot_10; // AOT slot +0x10; unboxed_field
  num _slot_14; // AOT slot +0x14; unboxed_field

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 167.
  dynamic compareTo(List<dynamic> args) {
    if ((arg1 & 1) == 0) {
      if (x4 == 756) {  /* low-confidence predicate */
        final compareToResult =
          aot.invoke('_IntegerImplementation.compareTo', <dynamic>[((this._slot_8 * this._slot_8) + (this._slot_10 * this._slot_10)), ((this._slot_8 * this._slot_8) + (this._slot_10 * this._slot_10))]);
        return compareToResult;
      } else {
      }
    } else {
    }
    // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed `E15Vec.get:hashCode`.
  dynamic get hashCode {
    return (this._slot_8 ^ this._slot_10);
    // Control-flow evidence: 1 conditional branch(es), 0 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }

  /// Partially reconstructed `E15Vec.==`.
  dynamic operator_equals(List<dynamic> args) {
    if (this != null) {
      if ((this & 1) == 0) {
        if (x2 != 756) {  /* low-confidence predicate */
        } else {
        }
      } else {
      }
    } else {
      return false;
    }
    // 3 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }
}

abstract class E20Base {
}

class E20Combo extends _MixinApplication1_E20Base_Counter with Loud {

  /// Partially reconstructed `E20Combo.greet`.
  dynamic greet(List<dynamic> args) {
    // Recovered source literals:
    //   'base+combo'
    return 'base+combo';
  }
}

/// Snapshot class flags identify this declaration as an enum; values may be tree-shaken.
abstract class E21Mode extends _Enum {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  num _slot_8; // AOT slot +0x8; unboxed_field
  num _slot_c; // AOT slot +0xc; unboxed_field
  num _slot_14; // AOT slot +0x14; unboxed_field
  num _slot_18; // AOT slot +0x18; unboxed_field

  /// Partially reconstructed from package:edge_probe/probe_code.dart near line 234.
  static dynamic parse(List<dynamic> args) {
    final enumByName_byNameResult =
      aot.invoke('EnumByName|byName', <dynamic>[aot.snapshotRef(53), aot.snapshotRef(18055)]);
    return enumByName_byNameResult;
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }

  /// Partially reconstructed `E21Mode._enumToString`.
  dynamic _enumToString(List<dynamic> args) {
    // Recovered source literals:
    //   'E21Mode.'
    return 'E21Mode.${aot.unresolvedValue('interpolated part')}';
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}

mixin Loud {
}

/// Synthetic mixin-application class retained by the AOT compiler.
abstract class _MixinApplication1_E20Base_Counter extends E20Base with Counter {
}

/// Synthetic mixin-application class retained by the AOT compiler.
abstract class _MixinApplication2_E20Base_Counter_Loud extends _MixinApplication1_E20Base_Counter with Loud {
}
