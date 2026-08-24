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

/// Partially reconstructed `seedNow`.
int seedNow() {
  final getCurrentMicrosResult = _getCurrentMicros();
  // Control-flow evidence: 1 conditional branch(es), 0 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  return aot.unresolvedRegion('Remaining behavior of seedNow', <dynamic>[]);
}

/// Partially reconstructed `main`.
final main_tearOff = main; // implicit closure forwarding to `main`

class ProbeApp extends StatelessWidget {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  dynamic _slot_4; // AOT slot +0x4; reference

  /// Partially reconstructed `ProbeApp.build`.
  dynamic build(dynamic context) {
    // Recovered source literals:
    //   'US'
    //   'clutter edge-case probe'
    final builderResult = Builder();
    final scaffoldResult = Scaffold(builderResult);
    final materialAppResult = MaterialApp(scaffoldResult);
    return materialAppResult;
  }

  /// Partially reconstructed `ProbeApp.anonymous closure`.
  Center closure_30679(BuildContext arg0) {
    // Recovered source literals:
    //   '[{"a":'
    //   'alpha'
    //   'beta-or-gamma'
    //   'other'
    //   'v v'
    //   '},{"b":null}]'
    final seedNowResult = seedNow(aot.unresolvedValue('shared-code result'));
    if (seedNowResult == (seedNowResult << 1)) {  /* low-confidence predicate */
      if ((aot.unresolvedRegister('r0') & 1) == 0) {  /* low-confidence predicate */
        final linearTextScalerResult = _LinearTextScaler(aot.unresolvedValue('shared-code result'));
        if (aot.unresolvedValue('slot 0x10') < 0) {  /* low-confidence predicate */
          if (aot.unresolvedRegister('r6') == aot.unresolvedRegister('r0')) {  /* low-confidence predicate */
            final _Result = aot.invoke('_IntegerImplementation.%', <dynamic>[6]);
            final fromCharCodeResult = aot.invoke('String.fromCharCode', <dynamic>[]);
            if ((seedNowResult & 1) != 0) {  /* low-confidence predicate */
              final fromLiteralResult =
                aot.invoke('Map._fromLiteral', <dynamic>[aot.unresolvedValue('(sub_318cac_result + 15)'), aot.unresolvedValue('slot 0x10'), linearTextScalerResult]);
              final _Result2 = aot.invoke('_IntegerImplementation.%', <dynamic>[]);
              final _Result3 = aot.invoke('_IntegerImplementation.%', <dynamic>[]);
              final double = _Double.fromInteger();
              final double2 = _Double.fromInteger();
              final e01InterpChainResult = e01InterpChain('${aot.snapshotRef(744)}${aot.unresolvedValue('slot 0x14')}');
              final textResult = Text();
              final interpolateSingleResult =
                aot.invoke('String._interpolateSingle', <dynamic>[aot.unresolvedValue('shared-code result'), aot.unresolvedValue('(sub_318cac_result + 11)'), aot.unresolvedValue('slot 0x14')]);
              final e02CascadeResult = e02Cascade(aot.unresolvedValue('shared-code result'), 4);
              final interpolateSingleResult2 = aot.invoke('String._interpolateSingle', <dynamic>[e02CascadeResult]);
              final textResult2 = Text();
              if ((textResult2 & 1) == 0) {  /* low-confidence predicate */
                final equalsResult =
                  aot.invoke('String.==', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                if (equalsResult != aot.unresolvedRegister('r0')) {  /* low-confidence predicate */
                  final equalsResult2 = aot.invoke('String.==', <dynamic>[equalsResult]);
                  if (equalsResult2 == aot.unresolvedRegister('r0')) {  /* low-confidence predicate */
                    final textResult3 = Text(linearTextScalerResult, aot.unresolvedValue('slot 0x4'));
                    if ((textResult3 & 1) == 0) {  /* low-confidence predicate */
                      final e04BitTwiddleResult = e04BitTwiddle(seedNowResult, aot.unresolvedValue('slot 0x10'), linearTextScalerResult);
                      if (e04BitTwiddleResult == (e04BitTwiddleResult << 1)) {  /* low-confidence predicate */
                        final interpolateSingleResult3 = aot.invoke('String._interpolateSingle', <dynamic>[]);
                        final textResult4 = Text();
                        if ((textResult4 & 1) == 0) {  /* low-confidence predicate */
                          final e05NullFlowResult = e05NullFlow(fromLiteralResult, linearTextScalerResult);
                          final interpolateSingleResult4 =
                            aot.invoke('String._interpolateSingle', <dynamic>[e05NullFlowResult]);
                          final textResult5 = Text();
                          if ((textResult5 & 1) == 0) {  /* low-confidence predicate */
                            final _Result4 =
                              aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                            final _Result5 = aot.invoke('_IntegerImplementation.%', <dynamic>[]);
                            final e06RecordDestructureResult =
                              e06RecordDestructure(aot.unresolvedValue('shared-code result'));
                            if (e06RecordDestructureResult == (e06RecordDestructureResult << 1)) {  /* low-confidence predicate */
                              final interpolateSingleResult5 = aot.invoke('String._interpolateSingle', <dynamic>[]);
                              final textResult6 = Text();
                              if ((textResult6 & 1) == 0) {  /* low-confidence predicate */
                                final _Result6 =
                                  aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('(sub_318cac_result + 11)')]);
                                if ((_Result6 & 1) == 0) {  /* low-confidence predicate */
                                  final fromLiteralResult2 =
                                    aot.invoke('Map._fromLiteral', <dynamic>[aot.unresolvedValue('shared-code result')]);
                                  final e07GenericBoundResult = e07GenericBound(aot.unresolvedValue('argument'));
                                  final interpolateSingleResult6 =
                                    aot.invoke('String._interpolateSingle', <dynamic>[e07GenericBoundResult]);
                                  final textResult7 = Text();
                                  if ((textResult7 & 1) == 0) {  /* low-confidence predicate */
                                    final _Result7 =
                                      aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                                    while (aot.unresolvedRegister('r3') < aot.unresolvedRegister('r2')) {
                                      if (aot.unresolvedRegister('r0') < aot.unresolvedRegister('r1')) {  /* low-confidence predicate */
                                      }
                                      while (aot.unresolvedRegister('r6') < aot.unresolvedRegister('r2')) {
                                        if (aot.unresolvedRegister('r4') < aot.unresolvedRegister('r1')) {  /* low-confidence predicate */
                                        }
                                        // unresolved predicate: r8 > 0 /* low-confidence predicate */
                                      }
                                    }
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
                        } else {
                        }
                      } else {
                      }
                    } else {
                    }
                  } else {
                    final equalsResult3 = aot.invoke('String.==', <dynamic>[equalsResult2]);
                    // unresolved predicate: equalsResult3 != r0 /* low-confidence predicate */
                  }
                } else {
                }
              } else {
              }
            } else {
            }
          } else {
          }
        } else {
          // unresolved predicate: seedNowResult >= 0 /* low-confidence predicate */
        }
      } else {
      }
    } else {
    }
    if (aot.unresolvedRegister('r6') == aot.unresolvedRegister('r0')) {  /* low-confidence predicate */
      final interpolateSingleResult7 = aot.invoke('String._interpolateSingle', <dynamic>[]);
      final textResult8 = Text();
      if ((textResult8 & 1) == 0) {  /* low-confidence predicate */
        if ((seedNowResult & 1) != 0) {  /* low-confidence predicate */
          final interpolateSingleResult8 =
            aot.invoke('String._interpolateSingle', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, aot.unresolvedValue('slot 0x14')]);
          final e09TryRethrowResult = e09TryRethrow(aot.unresolvedValue('slot 0x14'), aot.unresolvedValue('slot 0x10'));
          final textResult9 = Text();
          if ((textResult9 & 1) == 0) {  /* low-confidence predicate */
            final _Result8 =
              aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
            final e10AsyncLoopResult = e10AsyncLoop();
            final futureBuilderResult = FutureBuilder(aot.snapshotRef(18555));
            if ((futureBuilderResult & 1) == 0) {  /* low-confidence predicate */
              final e11SyncGenResult = e11SyncGen((seedNowResult & 3), (seedNowResult ^ seedNowResult));
              final growableList = _GrowableList._of(e11SyncGenResult);
              final interpolateSingleResult9 = aot.invoke('String._interpolateSingle', <dynamic>[growableList]);
              final textResult10 = Text();
              if ((textResult10 & 1) == 0) {  /* low-confidence predicate */
                final e12TearOffsResult =
                  e12TearOffs(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
                final interpolateSingleResult10 = aot.invoke('String._interpolateSingle', <dynamic>[e12TearOffsResult]);
                final textResult11 = Text();
                if ((textResult11 & 1) == 0) {  /* low-confidence predicate */
                  final e13DynamicResult =
                    E13Dynamic(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
                  final e13DynamicResult2 = E13Dynamic();
                  final probeResult = aot.invoke('E13Dynamic.probe', <dynamic>[e13DynamicResult2, e13DynamicResult2]);
                  final textResult12 = Text();
                  if ((textResult12 & 1) == 0) {  /* low-confidence predicate */
                    final bumpResult =
                      aot.invoke('E14Statics.bump', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                    if (bumpResult == (bumpResult << 1)) {  /* low-confidence predicate */
                      final interpolateSingleResult11 = aot.invoke('String._interpolateSingle', <dynamic>[]);
                      final textResult13 = Text();
                      if ((textResult13 & 1) == 0) {  /* low-confidence predicate */
                        final _Result9 =
                          aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                        final e15VecResult = E15Vec();
                        e15VecResult._slot_4 = aot.unresolvedValue('slot 0x10'); // AOT field store +0x4
                        e15VecResult._slot_8 = aot.unresolvedValue('slot 0x34'); // AOT field store +0x8
                        e15VecResult._slot_c = 4; // AOT field store +0xc
                        e15VecResult._slot_10 = 0; // AOT field store +0x10
                        final compareToResult = aot.invoke('E15Vec.compareTo', <dynamic>[const E15Vec(), e15VecResult]);
                        if (compareToResult == (compareToResult << 1)) {  /* low-confidence predicate */
                          final interpolateSingleResult12 = aot.invoke('String._interpolateSingle', <dynamic>[]);
                          final textResult14 = Text();
                          if ((textResult14 & 1) == 0) {  /* low-confidence predicate */
                            final setResult = <dynamic>{};
                            final interpolateSingleResult13 =
                              aot.invoke('String._interpolateSingle', <dynamic>[setResult, aot.snapshotRef(47653), aot.unresolvedValue('slot 0x14')]);
                            final addResult =
                              aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, interpolateSingleResult13]);
                            final addResult2 =
                              aot.invoke('__Set&_HashVMBase&SetMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashSetMixin.add', <dynamic>[setResult, aot.snapshotRef(545)]);
                            final e16SortedCopyResult = e16SortedCopy(setResult);
                            final interpolateSingleResult14 =
                              aot.invoke('String._interpolateSingle', <dynamic>[e16SortedCopyResult]);
                            final textResult15 = Text();
                            if ((textResult15 & 1) == 0) {  /* low-confidence predicate */
                              final e17JsonRoundTripResult = e17JsonRoundTrip('[{"a":${aot.unresolvedValue('slot 0x14')}},{"b":null}]');
                              final toStringResult = aot.invoke('_Smi.toString', <dynamic>[]);
                              final textResult16 = Text();
                              if ((textResult16 & 1) == 0) {  /* low-confidence predicate */
                                final _Result10 =
                                  aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                                final double3 = _Double.fromInteger();
                                final double4 = _Double.fromInteger();
                                final e18NumericEdgesResult = e18NumericEdges(double4);
                                final textResult17 = Text();
                                if ((textResult17 & 1) == 0) {  /* low-confidence predicate */
                                  final _Result11 =
                                    aot.invoke('_IntegerImplementation.%', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult]);
                                  final e19AckermannResult =
                                    e19Ackermann(_Result11, ((seedNowResult & 3) << 1), _Result11, ((seedNowResult & 3) << 1));
                                  final interpolateSingleResult15 =
                                    aot.invoke('String._interpolateSingle', <dynamic>[e19AckermannResult]);
                                  final textResult18 = Text();
                                  if ((textResult18 & 1) == 0) {  /* low-confidence predicate */
                                    final e20ComboResult =
                                      E20Combo(aot.unresolvedValue('shared-code result'), linearTextScalerResult);
                                    final greetResult = aot.invoke('E20Combo.greet', <dynamic>[e20ComboResult]);
                                    final textResult19 = Text();
                                    if ((textResult19 & 1) == 0) {  /* low-confidence predicate */
                                      final parseResult =
                                        aot.invoke('E21Mode.parse', <dynamic>[fromCharCodeResult, linearTextScalerResult]);
                                      if (parseResult == aot.snapshotInstance(E21Mode)) {  /* low-confidence predicate */
                                        final textResult20 = Text(aot.unresolvedValue('slot 0x14'), (seedNowResult & 1), aot.unresolvedValue('slot 0x4'));
                                        if ((textResult20 & 1) == 0) {  /* low-confidence predicate */
                                          final toStringResult2 =
                                            aot.invoke('_Smi.toString', <dynamic>[aot.unresolvedValue('shared-code result'), linearTextScalerResult, 2]);
                                          final textResult21 = Text();
                                          if ((textResult21 & 1) == 0) {  /* low-confidence predicate */
                                            final e23DynamicApplyResult =
                                              e23DynamicApply(aot.unresolvedValue('shared-code result'), aot.unresolvedValue('shared-code result'));
                                            final textResult22 = Text();
                                            if ((textResult22 & 1) == 0) {  /* low-confidence predicate */
                                              final _Result12 =
                                                aot.invoke('_IntegerImplementation.%', <dynamic>[linearTextScalerResult]);
                                              final _Result13 =
                                                aot.invoke('_IntegerImplementation.%', <dynamic>[_Result12]);
                                              final e24KnotResult =
                                                e24Knot(e23DynamicApplyResult, aot.unresolvedValue('shared-code result'));
                                              if (e24KnotResult == (e24KnotResult << 1)) {  /* low-confidence predicate */
                                                final interpolateSingleResult16 =
                                                  aot.invoke('String._interpolateSingle', <dynamic>[]);
                                                final textResult23 = Text();
                                                if ((textResult23 & 1) == 0) {  /* low-confidence predicate */
                                                  final e25IntrinsicsResult =
                                                    e25Intrinsics('${aot.snapshotRef(130)}${aot.unresolvedValue('slot 0x4')}');
                                                  if (e25IntrinsicsResult != aot.unresolvedRegister('r0')) {  /* low-confidence predicate */
                                                    final textResult24 =
                                                      Text(aot.unresolvedValue('shared-code result'));
                                                    if ((textResult24 & 1) == 0) {  /* low-confidence predicate */
                                                      final columnResult =
                                                        Column(aot.unresolvedValue('shared-code result'));
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
                                          } else {
                                          }
                                        } else {
                                        }
                                      } else {
                                        if (aot.unresolvedRegister('r1') < 0) {  /* low-confidence predicate */
                                        } else {
                                          // unresolved predicate: r2 >= 0 /* low-confidence predicate */
                                        }
                                      }
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
      } else {
      }
    } else {
    }
    // 48 branch region(s) and 2 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed `ProbeApp.anonymous closure`.
  String closure_31254() {
    return '${aot.snapshotRef(576)}${aot.unresolvedValue('interpolated part')}';
  }

  /// Partially reconstructed `ProbeApp.anonymous closure`.
  Text closure_31255(BuildContext arg0, AsyncSnapshot<int> arg1) {
    final interpolateSingleResult = aot.invoke('String._interpolateSingle', <dynamic>[arg0._slot_14, arg1._slot_c]);
    final textResult = Text(interpolateSingleResult);
    return textResult;
    // Control-flow evidence: 1 conditional branch(es), 0 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}
