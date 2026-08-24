import 'package:flutter/material.dart';

import 'probe_code.dart';

// Anchors every probe function into the retained graph. All inputs are
// derived from the wall clock so the AOT optimizer cannot constant-fold
// probe bodies against their call sites: what survives is the GENERAL
// body, and any remaining failure belongs to the decompiler, not to
// cross-function constant propagation.

int seedNow() => DateTime.now().microsecondsSinceEpoch;

void main() {
  runApp(const ProbeApp());
}

class ProbeApp extends StatelessWidget {
  const ProbeApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'clutter edge-case probe',
      home: Scaffold(
        body: Builder(builder: (context) {
          final s = seedNow();
          final text = TextScaler.linear(1);
          final sa = s.abs();
          final code = String.fromCharCode(97 + (sa % 3)); // a|b|c
          final mapArg = <String, String?>{
            'k': sa.isEven ? 'v v' : null,
          };
          return Center(
            child: Column(
              children: <Widget>[
                Text(e01InterpChain('u$sa', sa % 97, (sa % 100) / 100),
                    textScaler: text),
                Text('${e02Cascade(['$sa', 'a'])}', textScaler: text),
                Text(e03StringSwitch(code), textScaler: text),
                Text('${e04BitTwiddle(sa)}', textScaler: text),
                Text('${e05NullFlow(mapArg, 'k')}', textScaler: text),
                Text('${e06RecordDestructure((sa % 9, sa % 7, tag: "t$s"))}',
                    textScaler: text),
                Text('${e07GenericBound<int, int>({'x': sa % 11})}',
                    textScaler: text),
                Text('${e08LabeledLoops(sa % 5)}', textScaler: text),
                Text(e09TryRethrow(sa % 2 == 0 ? sa : '$sa'),
                    textScaler: text),
                FutureBuilder<int>(
                  future: e10AsyncLoop(sa % 5),
                  builder: (c, snap) =>
                      Text('${snap.data ?? 0}', textScaler: text),
                ),
                Text('${e11SyncGen(sa % 4).toList()}', textScaler: text),
                Text('${e12TearOffs()}', textScaler: text),
                Text('${E13Dynamic().probe(E13Dynamic())}', textScaler: text),
                Text('${E14Statics.bump()}', textScaler: text),
                Text('${const E15Vec(1, -2).compareTo(E15Vec(sa % 5, 4))}',
                    textScaler: text),
                Text('${e16SortedCopy({'$sa', 'a'}, null)}', textScaler: text),
                Text('${e17JsonRoundTrip('[{"a":$sa},{"b":null}]').length}',
                    textScaler: text),
                Text(e18NumericEdges((sa % 1000) / 8), textScaler: text),
                Text('${e19Ackermann(sa % 3, sa % 4)}', textScaler: text),
                Text(E20Combo().greet(), textScaler: text),
                Text('${E21Mode.parse(code).isBad}', textScaler: text),
                Text('${<int>[1].safeLen}', textScaler: text),
                Text(e23DynamicApply(() => 'z$s', [sa]), textScaler: text),
                Text('${e24Knot(sa.isEven, sa % 3 == 0, sa % 17)}',
                    textScaler: text),
                Text('${e25Intrinsics('k$s', 'ey')}', textScaler: text),
              ],
            ),
          );
        }),
      ),
    );
  }
}
