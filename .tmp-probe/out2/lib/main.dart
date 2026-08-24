// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.
// ignore_for_file: unused_element, unused_import, non_constant_identifier_names
// Recovered from: package:edge_probe/main.dart

import '../support/aot_intrinsics.dart' as aot;

const String recoveredSourceUri = 'package:edge_probe/main.dart';

dynamic recoveredEntry(List<dynamic> args) {
  return aot.unresolvedRegion(recoveredSourceUri, args);
}
/// Dart VM retained declaration object unknown; no distinct executable body survived.
void main() => throw UnsupportedError('AOT body unavailable');

/// Partially reconstructed from package:edge_probe/main.dart near line 11.
/// Inlined by the optimizer (statements live inside this body):
///   DateTime.now
///   DateTime._now
int seedNow() {
  final getCurrentMicrosResult = _getCurrentMicros();
  return getCurrentMicrosResult;
  // Control-flow evidence: 2 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

/// Partially reconstructed from package:edge_probe/main.dart near line 13.
/// Inlined by the optimizer (statements live inside this body):
///   main
void main_tearOff() {
  final runAppResult = runApp();
  return;
  // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

class ProbeApp extends StatelessWidget {

  /// Partially reconstructed from package:edge_probe/main.dart near line 25.
  dynamic build(dynamic context) {

    /// Closure recovered from package:edge_probe/main.dart near line 25.
    /// Inlined by the optimizer (statements live inside this body):
    ///   _GrowableList._literal2
    ///   e03StringSwitch
    ///   e08LabeledLoops
    ///   toList
    ///   List.of
    ///   _interpolateSingle
    ///   _GrowableList._literal1
    Center closureAtLine25(BuildContext arg0) {
      // Recovered source literals:
      //   line 30: 'v v'
      //   line 39: 'alpha'
      //   line 39: 'beta-or-gamma'
      //   line 39: 'other'
      //   line 61: '[{"a":'
      //   line 61: '},{"b":null}]'
      final seedNowResult = seedNow(aot.unresolvedValue('shared-code result'));
      final linearTextScalerResult =
        _LinearTextScaler(aot.unresolvedValue('shared-code result'), seedNowResult, seedNowResult);
      if ((seedNowResult & (1 << 63)) == 0) {  /* low-confidence predicate */
        final fromCharCodeResult =
          aot.invoke('String.fromCharCode', <dynamic>[null, linearTextScalerResult, aot.unresolvedValue('shared-code result')]);
        if ((local20 & 1) != 0) {  /* low-confidence predicate */
          final fromLiteralResult =
            aot.invoke('Map._fromLiteral', <dynamic>[local20, linearTextScalerResult, aot.snapshotRef(17932), aot.unresolvedValue('shared-code result')]);
          final e01InterpChainResult =
            e01InterpChain('${snapshotRef(260)}${local20}', (local20 % 97), (local20 % 97), (local20 % 100), (local20 ~/ 100));
          final textResult = Text();
          final interpolateSingleResult =
            aot.invoke('String._interpolateSingle', <dynamic>[aot.unresolvedValue('shared-code result'), local20]);
          final e02CascadeResult = e02Cascade(aot.unresolvedValue('shared-code result'), 4);
          final interpolateSingleResult2 = aot.invoke('String._interpolateSingle', <dynamic>[e02CascadeResult]);
          final textResult2 = Text();
          final equalsResult =
            aot.invoke('String.==', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, aot.snapshotRef(458), fromCharCodeResult]);
          if (!(equalsResult)) {  /* medium-confidence predicate */
            final equalsResult2 = aot.invoke('String.==', <dynamic>[aot.snapshotRef(734), fromCharCodeResult]);
            if (equalsResult2) {  /* medium-confidence predicate */
              final textResult3 = Text(local20, seedNowResult);
              final e04BitTwiddleResult = e04BitTwiddle(local20, linearTextScalerResult);
              final interpolateSingleResult3 =
                aot.invoke('String._interpolateSingle', <dynamic>[e04BitTwiddleResult, e04BitTwiddleResult]);
              final textResult4 = Text();
              final e05NullFlowResult = e05NullFlow(fromLiteralResult, linearTextScalerResult);
              final interpolateSingleResult4 = aot.invoke('String._interpolateSingle', <dynamic>[e05NullFlowResult]);
              final textResult5 = Text();
              final e06RecordDestructureResult = e06RecordDestructure(aot.unresolvedValue('shared-code result'));
              final interpolateSingleResult5 =
                aot.invoke('String._interpolateSingle', <dynamic>[e06RecordDestructureResult, e06RecordDestructureResult]);
              final textResult6 = Text();
              final fromLiteralResult2 =
                aot.invoke('Map._fromLiteral', <dynamic>[local20, (local20 % 11), (local20 % 11), (local20 ~/ 11), aot.snapshotRef(17985), aot.unresolvedValue('shared-code result')]);
              final e07GenericBoundResult =
                e07GenericBound(aot.snapshotRef(34545), aot.snapshotRef(17903), fromLiteralResult2);
              final interpolateSingleResult6 =
                aot.invoke('String._interpolateSingle', <dynamic>[e07GenericBoundResult]);
              final textResult7 = Text();
              while (0 < (local20 % 5)) {
                while (0 < (local20 % 5)) {
                  if ((0 * 0) > 6) {  /* low-confidence predicate */
                  } else {
                    if ((0 + 0) == (local20 % 5)) {  /* low-confidence predicate */
                    } else {
                    }
                  }
                }
              }
            } else {
              final equalsResult3 = aot.invoke('String.==', <dynamic>[aot.snapshotRef(343), fromCharCodeResult]);
              if (!(equalsResult3)) {  /* medium-confidence predicate */
              }
            }
          } else {
          }
        } else {
        }
      } else {
      }
      final interpolateSingleResult7 =
        aot.invoke('String._interpolateSingle', <dynamic>[0, linearTextScalerResult, local20, (local20 % 5), 0, 0]);
      final textResult8 = Text();
      if ((local20 & 1) != 0) {  /* low-confidence predicate */
        final interpolateSingleResult8 =
          aot.invoke('String._interpolateSingle', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, local20]);
        final e09TryRethrowResult = e09TryRethrow(local20, (local20 % 5), local20, seedNowResult);
        final textResult9 = Text();
        final e10AsyncLoopResult = e10AsyncLoop((local20 % 5), linearTextScalerResult);
        final futureBuilderResult = FutureBuilder(aot.snapshotRef(18530));
        final e11SyncGenResult =
          e11SyncGen((((((local20 >> 0) & 0xffffffff) & 3) >> 0) & 0xffffffff), (((local20 >> 0) & 0xffffffff) & 3));
        final growableList = _GrowableList._of(e11SyncGenResult);
        final interpolateSingleResult9 = aot.invoke('String._interpolateSingle', <dynamic>[growableList]);
        final textResult10 = Text();
        final e12TearOffsResult = e12TearOffs(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        final interpolateSingleResult10 = aot.invoke('String._interpolateSingle', <dynamic>[e12TearOffsResult]);
        final textResult11 = Text();
        final e13DynamicResult = E13Dynamic(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        final e13DynamicResult2 = E13Dynamic();
        final probeResult = aot.invoke('E13Dynamic.probe', <dynamic>[e13DynamicResult2, e13DynamicResult2]);
        final textResult12 = Text();
        final bumpResult =
          aot.invoke('E14Statics.bump', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
        final interpolateSingleResult11 = aot.invoke('String._interpolateSingle', <dynamic>[bumpResult, bumpResult]);
        final textResult13 = Text();
        final e15VecResult = E15Vec(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        e15VecResult._slot_8 = (local20 % 5); // AOT field store +0x8
        e15VecResult._slot_10 = 4; // AOT field store +0x10
        final compareToResult = aot.invoke('E15Vec.compareTo', <dynamic>[const E15Vec(), e15VecResult]);
        final interpolateSingleResult12 =
          aot.invoke('String._interpolateSingle', <dynamic>[compareToResult, compareToResult]);
        final textResult14 = Text();
        final setResult = <dynamic>{};
        final interpolateSingleResult13 = aot.invoke('String._interpolateSingle', <dynamic>[setResult, local20]);
        final addResult =
          aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, interpolateSingleResult13]);
        final addResult2 =
          aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, aot.snapshotRef(458)]);
        final e16SortedCopyResult = e16SortedCopy(setResult);
        final interpolateSingleResult14 = aot.invoke('String._interpolateSingle', <dynamic>[e16SortedCopyResult]);
        final textResult15 = Text();
        final e17JsonRoundTripResult = e17JsonRoundTrip('[{"a":${local20}},{"b":null}]');
        final toStringResult = aot.invoke('_Smi.toString', <dynamic>[]);
        final textResult16 = Text();
        final e18NumericEdgesResult =
          e18NumericEdges(1000, linearTextScalerResult, (local20 % 1000), (local20 ~/ 1000));
        final textResult17 = Text();
        final e19AckermannResult =
          e19Ackermann((seedNowResult << 1), linearTextScalerResult, (((local20 >> 0) & 0xffffffff) & 3), aot.snapshotRef(23), (seedNowResult << 1), ((((local20 >> 0) & 0xffffffff) & 3) << 1));
        final interpolateSingleResult15 = aot.invoke('String._interpolateSingle', <dynamic>[e19AckermannResult]);
        final textResult18 = Text();
        final e20ComboResult = E20Combo(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        final greetResult = aot.invoke('E20Combo.greet', <dynamic>[e20ComboResult]);
        final textResult19 = Text();
        final parseResult = aot.invoke('E21Mode.parse', <dynamic>[fromCharCodeResult, linearTextScalerResult]);
        if (parseResult == snapshotInstance(E21Mode)) {  /* low-confidence predicate */
          final textResult20 = Text(seedNowResult, local20, local20, seedNowResult);
          final toStringResult2 =
            aot.invoke('_Smi.toString', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, 2]);
          final textResult21 = Text();
          final e23DynamicApplyResult =
            e23DynamicApply(aot.unresolvedValue('shared-code result'), aot.unresolvedValue('shared-code result'));
          final textResult22 = Text();
          if ((((local20 >> 0) & 0xffffffff) & 1) == 0) {  /* low-confidence predicate */
            if (seedNowResult == 0) {  /* low-confidence predicate */
              final e24KnotResult = e24Knot((local20 % 17), linearTextScalerResult, (local20 % 17), (local20 ~/ 17));
              final interpolateSingleResult16 =
                aot.invoke('String._interpolateSingle', <dynamic>[e24KnotResult, e24KnotResult]);
              final textResult23 = Text();
              final e25IntrinsicsResult = e25Intrinsics('${snapshotRef(870)}${seedNowResult}');
              if (!(e25IntrinsicsResult)) {  /* medium-confidence predicate */
                final textResult24 = Text(aot.unresolvedValue('shared-code result'));
                final columnResult = Column(aot.unresolvedValue('shared-code result'));
                final centerResult = Center(columnResult);
                return centerResult;
              } else {
              }
            } else {
            }
          } else {
          }
        } else {
        }
      } else {
      }
    }
    // Recovered source literals:
    //   line 22: 'US'
    //   line 22: 'clutter edge-case probe'
    final builderResult = Builder();
    final scaffoldResult = Scaffold(builderResult);
    final materialAppResult = MaterialApp(scaffoldResult, false);
    return materialAppResult;
  }

  /// Partially reconstructed from package:edge_probe/main.dart near line 68.
  String closureAtLine68() {
    return '${snapshotRef(427)}${aot.unresolvedValue('interpolated part')}';
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }

  /// Partially reconstructed from package:edge_probe/main.dart near line 51.
  Text closureAtLine51(BuildContext arg0, AsyncSnapshot<int> arg1) {
    if (x2 != null) {  /* low-confidence predicate */
      final interpolateSingleResult = aot.invoke('String._interpolateSingle', <dynamic>[]);
      final textResult = Text(interpolateSingleResult);
      return textResult;
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }
}
