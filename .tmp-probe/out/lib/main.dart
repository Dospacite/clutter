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

/// Partially reconstructed from package:edge_probe/main.dart near line 8.
/// Inlined by the optimizer (statements live inside this body):
///   main
void main_tearOff() {
  final runAppResult = runApp();
  return;
  // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

class ProbeApp extends StatelessWidget {

  /// Partially reconstructed from package:edge_probe/main.dart near line 20.
  dynamic build(dynamic context) {

    /// Closure recovered from package:edge_probe/main.dart near line 20.
    /// Inlined by the optimizer (statements live inside this body):
    ///   _GrowableList._literal2
    ///   e08LabeledLoops
    ///   toList
    ///   List.of
    ///   _interpolateSingle
    Center closureAtLine20(BuildContext arg0) {
      // Recovered source literals:
      //   line 28: 'beta-or-gamma'
      //   line 29: 'v v'
      final linearTextScalerResult = _LinearTextScaler(aot.unresolvedValue('shared-code result'));
      final e01InterpChainResult = e01InterpChain(aot.unresolvedValue('shared-code result'));
      final textResult = Text();
      final e02CascadeResult = e02Cascade(aot.unresolvedValue('shared-code result'), 4);
      final interpolateSingleResult = aot.invoke('String._interpolateSingle', <dynamic>[e02CascadeResult]);
      final textResult2 = Text();
      final textResult3 = Text(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final e04BitTwiddleResult = e04BitTwiddle(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final interpolateSingleResult2 =
        aot.invoke('String._interpolateSingle', <dynamic>[e04BitTwiddleResult, e04BitTwiddleResult]);
      final textResult4 = Text();
      final fromLiteralResult =
        aot.invoke('Map._fromLiteral', <dynamic>[aot.snapshotRef(17882), aot.unresolvedValue('shared-code result')]);
      final e05NullFlowResult = e05NullFlow(fromLiteralResult);
      final interpolateSingleResult3 = aot.invoke('String._interpolateSingle', <dynamic>[e05NullFlowResult]);
      final textResult5 = Text();
      final e06RecordDestructureResult = e06RecordDestructure(aot.unresolvedValue('shared-code result'));
      final interpolateSingleResult4 =
        aot.invoke('String._interpolateSingle', <dynamic>[e06RecordDestructureResult, e06RecordDestructureResult]);
      final textResult6 = Text();
      final fromLiteralResult2 =
        aot.invoke('Map._fromLiteral', <dynamic>[aot.snapshotRef(17935), aot.unresolvedValue('shared-code result')]);
      final e07GenericBoundResult = e07GenericBound(aot.snapshotRef(34410), aot.snapshotRef(17853), fromLiteralResult2);
      final interpolateSingleResult5 = aot.invoke('String._interpolateSingle', <dynamic>[e07GenericBoundResult]);
      final textResult7 = Text();
      while (0 < 4) {
        while (0 < 4) {
          if ((0 * 0) > 6) {  /* low-confidence predicate */
          } else {
            if ((0 + 0) == 4) {  /* low-confidence predicate */
            } else {
            }
          }
        }
      }
      final interpolateSingleResult6 =
        aot.invoke('String._interpolateSingle', <dynamic>[0, linearTextScalerResult, 0, 0]);
      final textResult8 = Text();
      final e09TryRethrowResult = e09TryRethrow(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final textResult9 = Text();
      final e10AsyncLoopResult = e10AsyncLoop(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final futureBuilderResult = FutureBuilder(aot.snapshotRef(18479));
      final e11SyncGenResult = e11SyncGen(3);
      final growableList = _GrowableList._of(e11SyncGenResult);
      final interpolateSingleResult7 = aot.invoke('String._interpolateSingle', <dynamic>[growableList]);
      final textResult10 = Text();
      final bumpResult =
        aot.invoke('E14Statics.bump', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
      final interpolateSingleResult8 = aot.invoke('String._interpolateSingle', <dynamic>[bumpResult, bumpResult]);
      final textResult11 = Text();
      final e15VecResult = E15Vec(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      e15VecResult._slot_10 = 4; // AOT field store +0x10
      final compareToResult = aot.invoke('E15Vec.compareTo', <dynamic>[const E15Vec(), e15VecResult]);
      final interpolateSingleResult9 =
        aot.invoke('String._interpolateSingle', <dynamic>[compareToResult, compareToResult]);
      final textResult12 = Text();
      final setResult = <dynamic>{};
      final addResult =
        aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, aot.snapshotRef(734), setResult]);
      final addResult2 =
        aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, aot.snapshotRef(458)]);
      final e16SortedCopyResult = e16SortedCopy(setResult);
      final interpolateSingleResult10 = aot.invoke('String._interpolateSingle', <dynamic>[e16SortedCopyResult]);
      final textResult13 = Text();
      final e17JsonRoundTripResult =
        e17JsonRoundTrip(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final toStringResult = aot.invoke('_Smi.toString', <dynamic>[]);
      final textResult14 = Text();
      final e18NumericEdgesResult = e18NumericEdges(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final textResult15 = Text();
      final e19AckermannResult =
        e19Ackermann(aot.unresolvedValue('shared-code result'), linearTextScalerResult, aot.snapshotRef(23), 4, 4);
      final interpolateSingleResult11 = aot.invoke('String._interpolateSingle', <dynamic>[e19AckermannResult]);
      final textResult16 = Text();
      final e20ComboResult = E20Combo(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
      final greetResult = aot.invoke('E20Combo.greet', <dynamic>[e20ComboResult]);
      final textResult17 = Text();
      final parseResult =
        aot.invoke('E21Mode.parse', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
      if (parseResult == snapshotInstance(E21Mode)) {  /* low-confidence predicate */
        final textResult18 = Text();
        final toStringResult2 =
          aot.invoke('_Smi.toString', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, 2]);
        final textResult19 = Text();
        final growableList2 = <dynamic>[];
        final e23DynamicApplyResult = e23DynamicApply(aot.unresolvedValue('shared-code result'), growableList2);
        final textResult20 = Text();
        final e13DynamicResult = E13Dynamic(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        final e13DynamicResult2 = E13Dynamic();
        final probeResult = aot.invoke('E13Dynamic.probe', <dynamic>[e13DynamicResult2, e13DynamicResult2]);
        final textResult21 = Text();
        final textResult22 = Text(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        final e25IntrinsicsResult = e25Intrinsics(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
        if (!(e25IntrinsicsResult)) {  /* medium-confidence predicate */
          final textResult23 = Text(aot.unresolvedValue('shared-code result'));
          final columnResult = Column(aot.unresolvedValue('shared-code result'));
          final centerResult = Center(columnResult);
          return centerResult;
        } else {
        }
      } else {
      }
    }
    // Recovered source literals:
    //   line 17: 'US'
    //   line 17: 'clutter edge-case probe'
    final builderResult = Builder();
    final scaffoldResult = Scaffold(builderResult);
    final materialAppResult = MaterialApp(scaffoldResult, false);
    return materialAppResult;
  }

  /// Partially reconstructed from package:edge_probe/main.dart near line 52.
  String closureAtLine52() {
    return aot.snapshotRef(427);
  }

  /// Partially reconstructed from package:edge_probe/main.dart near line 38.
  Text closureAtLine38(BuildContext arg0, AsyncSnapshot<int> arg1) {
    if (x2 != null) {  /* low-confidence predicate */
      final interpolateSingleResult = aot.invoke('String._interpolateSingle', <dynamic>[]);
      final textResult = Text(interpolateSingleResult);
      return textResult;
    } else {
    }
    // 1 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }
}
