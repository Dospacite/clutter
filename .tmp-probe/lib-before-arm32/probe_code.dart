// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.
// ignore_for_file: unused_element, unused_import, non_constant_identifier_names
// Recovered from: package:edge_probe/probe_code.dart

import '../support/aot_intrinsics.dart' as aot;

const String recoveredSourceUri = 'package:edge_probe/probe_code.dart';
/// Dart VM retained declaration object unknown; no distinct executable body survived.
dynamic e03StringSwitch(List<dynamic> args) => throw UnsupportedError('AOT body unavailable');
/// Dart VM retained declaration object unknown; no distinct executable body survived.
int e08LabeledLoops(int arg0) => throw UnsupportedError('AOT body unavailable');

/// Partially reconstructed `e19Ackermann`.
int e19Ackermann(int arg0, [int arg1 /* default unavailable */]) {
  if (r1 < 2) {  /* low-confidence predicate */
    if (r3 != 0) {  /* low-confidence predicate */
      if (r6 != 0) {  /* low-confidence predicate */
        if (r4 == r0) {  /* low-confidence predicate */
          final e19AckermannResult = e19Ackermann();
          if (local4 == (local4 << 1)) {  /* low-confidence predicate */
            final e19AckermannResult2 = e19Ackermann(e19AckermannResult);
            return e19AckermannResult2;
          } else {
          }
        } else {
        }
      } else {
        if (r4 == r0) {  /* low-confidence predicate */
          final e19AckermannResult3 = e19Ackermann();
          return e19AckermannResult3;
        } else {
        }
      }
    } else {
      if (r4 == r0) {  /* low-confidence predicate */
      } else {
      }
    }
  } else {
  }
  // 7 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e19Ackermann`.
int e19Ackermann_tearOff(int arg0, [int arg1 /* default unavailable */]) {
  if (r1 < 2) {  /* low-confidence predicate */
    final e19AckermannResult = e19Ackermann();
    return e19AckermannResult;
  } else {
  }
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e25Intrinsics`.
dynamic e25Intrinsics(List<dynamic> args) {
  // Recovered source literals:
  //   'ey'
  final combinedResult = aot.invoke('String.+', <dynamic>[]);
  if (r3 != 0) {  /* low-confidence predicate */
    final startsWithResult = aot.invoke('String.startsWith', <dynamic>[combinedResult, aot.snapshotRef(130)]);
    if ((startsWithResult & 8) == 0) {  /* low-confidence predicate */
      if (0 >= local8) {  /* low-confidence predicate */
      } else {
        if (r1 != 188) {  /* low-confidence predicate */
          if (r1 != 107) {  /* low-confidence predicate */
          } else {
          }
        } else {
          if (r1 != 107) {  /* low-confidence predicate */
          } else {
          }
        }
      }
    } else {
    }
  } else {
  }
  // 6 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e25Intrinsics', <dynamic>[]);
}

/// Partially reconstructed `e24Knot`.
dynamic e24Knot(List<dynamic> args) {
  if (arg0 != r4) {  /* low-confidence predicate */
    if (r8 < 0) {  /* low-confidence predicate */
      // Loop shape recovered without a provable predicate /* low-confidence predicate */.
      while (true) {
      }
    } else {
      if (arg2 >= 0) {  /* low-confidence predicate */
        while (r1 > 4) {
          if (r0 > 0) {  /* low-confidence predicate */
          }
        }
      }
    }
  } else {
    if (arg1 != r4) {  /* low-confidence predicate */
      return 2;
    } else {
      return 1;
    }
  }
  return 3;
  // 5 branch region(s) and 2 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e23DynamicApply`.
dynamic e23DynamicApply(List<dynamic> args) {
  final applyResult = apply();
  // Dynamic-call evidence:
  //   1 register-indirect call site(s) remain unresolved.
  return aot.unresolvedRegion('Remaining behavior of e23DynamicApply', <dynamic>[]);
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E20Combo`.
dynamic package_edge_probe_probe_code_dart_E20Combo(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E20Combo', <dynamic>[]);
}

/// Partially reconstructed `e18NumericEdges`.
dynamic e18NumericEdges(List<dynamic> args) {
  // Recovered source literals:
  //   'frac:'
  //   'integral:'
  //   'nan'
  return 'nan';
  // Dynamic-call evidence:
  //   2 register-indirect call site(s) remain unresolved.
  // 3 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e17JsonRoundTrip`.
dynamic e17JsonRoundTrip(List<dynamic> args) {
  final jsonDecodeResult = jsonDecode();
  if (r4 <= 2) {  /* low-confidence predicate */
    final mapResult = aot.invoke('Iterable.map', <dynamic>[]);
    final growableList = _GrowableList._of(mapResult);
    return growableList;
  } else {
    if (r4 <= 55) {  /* low-confidence predicate */
    } else {
    }
  }
  // Dynamic-call evidence:
  //   1 register-indirect call site(s) remain unresolved.
  // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `anonymous closure`.
Map<String, Object?> closure_30723(Map arg0) {
  // Dynamic-call evidence:
  //   .==(...) at 1 site(s): 62 candidate implementation(s), e.g. ==, ButtonThemeData.==, CardThemeData.==, Color.==.
  return aot.unresolvedRegion('Remaining behavior of anonymous closure', <dynamic>[]);
}

/// Partially reconstructed `e16SortedCopy`.
dynamic e16SortedCopy(List<dynamic> args) {
  final splayTreeSetResult = aot.invoke('dart:collection.SplayTreeSet', <dynamic>[arg0, aot.snapshotRef(18569)]);
  final splayTreeSet = SplayTreeSet(splayTreeSetResult);
  final addAllResult = aot.invoke('SplayTreeSet.addAll', <dynamic>[splayTreeSetResult, arg0]);
  final list = _List._of(aot.snapshotRef(18569), splayTreeSetResult);
  return list;
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E15Vec`.
dynamic package_edge_probe_probe_code_dart_E15Vec(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E15Vec', <dynamic>[]);
}

/// Partially reconstructed `package:edge_probe/probe_code.dart.E13Dynamic`.
dynamic package_edge_probe_probe_code_dart_E13Dynamic(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:edge_probe/probe_code.dart.E13Dynamic', <dynamic>[]);
}

/// Partially reconstructed `e12TearOffs`.
dynamic e12TearOffs(List<dynamic> args) {
  final get_firstResult = aot.unresolvedValue('shared-code result').first;
  return aot.unresolvedValue('shared-code result');
}

/// Partially reconstructed `e11SyncGen`.
dynamic e11SyncGen(List<dynamic> args) {
  while (r8 < arg0) {
    if (r6 < arg1) {  /* low-confidence predicate */
    }
    if (r1 != 0) {  /* low-confidence predicate */
      final e11SyncGenResult = e11SyncGen(local10, local14);
    } else {
      if (r8 == r0) {  /* low-confidence predicate */
        if ((r0 & 1) == 0) {  /* low-confidence predicate */
        } else {
        }
      } else {
      }
    }
  }
  // 4 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e11SyncGen', <dynamic>[]);
}

/// Partially reconstructed `e10AsyncLoop`.
dynamic e10AsyncLoop(List<dynamic> args) async {
  while (r4 < arg0) {
    if (r2 < arg1) {  /* low-confidence predicate */
    }
    final futureResult = aot.invoke('dart:async._Future', <dynamic>[arg1, aot.snapshotRef(18555)]);
    if (local14 == (local14 << 1)) {  /* low-confidence predicate */
      final asyncCompleteResult = aot.invoke('_Future._asyncComplete', <dynamic>[]);
      final combinedResult = aot.invoke('_IntegerImplementation.+', <dynamic>[]);
      if (r1 > 0) {  /* low-confidence predicate */
      } else {
        if (r2 > 100) {  /* low-confidence predicate */
        } else {
        }
      }
    } else {
    }
  }
  final future = Future.delayed(aot.snapshotRef(18524), const Duration());
  // Body compiled to an async state machine; await boundaries are unnamed in this snapshot.
  // 4 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e10AsyncLoop', <dynamic>[]);
}

/// Partially reconstructed `e09TryRethrow`.
dynamic e09TryRethrow(List<dynamic> args) {
  // Recovered source literals:
  //   'bad'
  //   'fallback'
  //   'ok:'
  final combinedResult = aot.invoke('String.+', <dynamic>[]);
  if (r2 > 1) {  /* low-confidence predicate */
    final formatExceptionResult = aot.invoke('dart:core.FormatException', <dynamic>[combinedResult]);
    // Loop shape recovered without a provable predicate /* low-confidence predicate */.
    while (true) {
      final combinedResult2 = aot.invoke('String.+', <dynamic>[]);
      final combinedResult3 = aot.invoke('String.+', <dynamic>[]);
    }
  } else {
    final combinedResult4 = aot.invoke('String.+', <dynamic>[]);
    return 'ok:${arg0}';
  }
  // 1 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e07GenericBound`.
Map<String, num> e07GenericBound<T extends num, U extends T>(Map<String, U> arg0) {
  if (r1 != 0) {  /* low-confidence predicate */
    if (r1 != 0) {  /* low-confidence predicate */
      final mapResult = map(aot.unresolvedValue('shared-code result'));
      return mapResult;
    } else {
    }
  } else {
  }
  // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `anonymous closure`.
MapEntry<String, num> closure_30756(String arg0, U arg1) {
  final mapEntryResult = aot.invoke('dart:core.MapEntry', <dynamic>[aot.snapshotRef(18413)]);
  return mapEntryResult;
  // Dynamic-call evidence:
  //   .==(...) at 1 site(s): 126 candidate implementation(s), e.g. ==, AccessibilityFeatures.==, AlignmentGeometry.==, AttributedString.==.
}

/// Partially reconstructed `e06RecordDestructure`.
dynamic e06RecordDestructure(List<dynamic> args) {
  while (r0 < 2) {
  }
  // Dynamic-call evidence:
  //   .==(...) at 1 site(s): 132 candidate implementation(s), e.g. ==, AccessibilityFeatures.==, AlignmentGeometry.==, AsyncSnapshot.==.
  // 0 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e06RecordDestructure', <dynamic>[]);
}

/// Partially reconstructed `e05NullFlow`.
dynamic e05NullFlow(List<dynamic> args) {
  // Recovered source literals:
  //   'missing'
  final lookupResult = aot.invoke('Map.lookup', <dynamic>[arg0, aot.snapshotRef(130)]);
  if (r2 != lookupResult) {  /* low-confidence predicate */
    if (local8 != r4) {  /* low-confidence predicate */
      if (r2 == r0) {  /* low-confidence predicate */
        if (local8 != r0) {  /* low-confidence predicate */
          final lookupResult2 = aot.invoke('Map.lookup', <dynamic>[arg0, 'missing']);
          if (r2 != lookupResult2) {  /* low-confidence predicate */
            if (r2 != r0) {  /* low-confidence predicate */
            } else {
              final updatedItemResult =
                aot.invoke('__Map&_HashVMBase&MapMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashMapMixin.[]=', <dynamic>[arg0, 'missing', local8]);
            }
          } else {
          }
        } else {
          final joinResult =
            aot.invoke('List.join', <dynamic>[aot.unresolvedValue('shared-code result'), aot.snapshotRef(732)]);
        }
      } else {
        final addAllResult = aot.invoke('List.addAll', <dynamic>[aot.unresolvedValue('shared-code result')]);
      }
    } else {
    }
  } else {
  }
  // Dynamic-call evidence:
  //   .==(...) at 1 site(s): 127 candidate implementation(s), e.g. ==, AlignmentGeometry.==, AppBarThemeData.==, AttributedString.==.
  // 6 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e05NullFlow', <dynamic>[]);
}

/// Partially reconstructed `e04BitTwiddle`.
int e04BitTwiddle(int arg0) {
  if (arg0 == (arg0 << 1)) {
    final quotientResult = aot.invoke('_IntegerImplementation.~/', <dynamic>[14]);
    final quotientResult2 = aot.invoke('_IntegerImplementation.~/', <dynamic>[]);
  } else {
  }
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  return aot.unresolvedRegion('Remaining behavior of e04BitTwiddle', <dynamic>[]);
}

/// Partially reconstructed `e02Cascade`.
dynamic e02Cascade(List<dynamic> args) {
  // Recovered source literals:
  //   'done'
  final growableList = <dynamic>[];
  final addAllResult = aot.invoke('List.addAll', <dynamic>[growableList, arg0]);
  final sortResult = aot.invoke('List.sort', <dynamic>[growableList]);
  if (r1 != r8) {  /* low-confidence predicate */
    return growableList;
  } else {
    final growToNextCapacityResult = aot.invoke('List._growToNextCapacity', <dynamic>[growableList]);
  }
  // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `e01InterpChain`.
dynamic e01InterpChain(List<dynamic> args) {
  // Recovered source literals:
  //   ' bool='
  //   ' id='
  //   ' nullish='
  //   ' pct='
  //   '% nested=inner-'
  //   'user='
  if ((r0 & 1) == 0) {  /* low-confidence predicate */
    final toStringAsFixedResult = aot.invoke('_Double.toStringAsFixed', <dynamic>[1, 0]);
    if ((toStringAsFixedResult & 1) == 0) {  /* low-confidence predicate */
      if ((arg0 & 1) == 0) {  /* low-confidence predicate */
        if (arg2 > 0) {  /* low-confidence predicate */
          return 'user=${arg0} id=${aot.unresolvedValue('interpolated part')}${aot.unresolvedValue('interpolated part')}${toStringAsFixedResult}${aot.unresolvedValue('interpolated part')}${aot.unresolvedValue('interpolated part')}${aot.unresolvedValue('interpolated part')}${aot.unresolvedValue('interpolated part')} nullish=${aot.unresolvedValue('interpolated part')}';
        } else {
          if (arg1 > 10) {  /* low-confidence predicate */
          } else {
          }
        }
      } else {
      }
    } else {
    }
  } else {
  }
  // 5 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

mixin Counter {
}

class E13Dynamic {

  /// Partially reconstructed `E13Dynamic.probe`.
  dynamic probe(List<dynamic> args) {
    return aot.unresolvedRegion('Remaining behavior of E13Dynamic.probe', <dynamic>[]);
  }

  /// Partially reconstructed `E13Dynamic.noSuchMethod`.
  dynamic noSuchMethod(List<dynamic> args) {
    // Recovered source literals:
    //   'unhandled:'
    final get_memberNameResult = memberName;
    if ((get_memberNameResult & 1) == 0) {  /* low-confidence predicate */
      final get_positionalArgumentsResult = positionalArguments;
      if ((r0 & 1) == 0) {  /* low-confidence predicate */
        return 'unhandled:${get_memberNameResult}${snapshotRef(244)}${aot.unresolvedValue('interpolated part')}';
      } else {
      }
    } else {
    }
    // Dynamic-call evidence:
    //   1 register-indirect call site(s) remain unresolved.
    // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }
}

abstract class E14Statics {

  /// Partially reconstructed `E14Statics.bump`.
  static int bump() {
    if (r4 == r0) {  /* low-confidence predicate */
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
    return aot.unresolvedRegion('Remaining behavior of E14Statics.bump', <dynamic>[]);
  }

  /// Partially reconstructed `E14Statics.init:stamp`.
  static late final int stamp = _getCurrentMicros();
}

class E15Vec implements Comparable<E15Vec> {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  num _slot_4; // AOT slot +0x4; unboxed_field
  num _slot_8; // AOT slot +0x8; unboxed_field
  num _slot_c; // AOT slot +0xc; unboxed_field
  num _slot_10; // AOT slot +0x10; unboxed_field

  /// Partially reconstructed `E15Vec.compareTo`.
  dynamic compareTo(List<dynamic> args) {
    if (r4 == 756) {  /* low-confidence predicate */
      if (r8 == r0) {  /* low-confidence predicate */
        if (r3 == r0) {  /* low-confidence predicate */
          final compareToResult = aot.invoke('_IntegerImplementation.compareTo', <dynamic>[]);
          return compareToResult;
        } else {
        }
      } else {
      }
    } else {
    }
    // 3 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed `E15Vec.get:hashCode`.
  dynamic get hashCode {
    if ((this._slot_4 ^ this._slot_c) == ((this._slot_4 ^ this._slot_c) << 1)) {
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
    return aot.unresolvedRegion('Remaining behavior of E15Vec.get:hashCode', <dynamic>[]);
  }

  /// Partially reconstructed `E15Vec.==`.
  dynamic operator_equals(List<dynamic> args) {
    if (r2 != 756) {  /* low-confidence predicate */
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
    return aot.unresolvedRegion('Remaining behavior of E15Vec.==', <dynamic>[]);
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
  num _slot_4; // AOT slot +0x4; unboxed_field
  num _slot_8; // AOT slot +0x8; unboxed_field
  num _slot_10; // AOT slot +0x10; unboxed_field
  num _slot_14; // AOT slot +0x14; unboxed_field

  /// Partially reconstructed `E21Mode.parse`.
  static dynamic parse(List<dynamic> args) {
    final enumByName_byNameResult = aot.invoke('EnumByName|byName', <dynamic>[]);
    return enumByName_byNameResult;
  }

  /// Partially reconstructed `E21Mode._enumToString`.
  dynamic _enumToString(List<dynamic> args) {
    // Recovered source literals:
    //   'E21Mode.'
    return 'E21Mode.${this._slot_c}';
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
