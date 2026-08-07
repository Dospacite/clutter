import 'package:clutter_edge_cases/main.dart';
import 'package:clutter_edge_cases/models/edge_models.dart';
import 'package:clutter_edge_cases/services/edge_case_harness.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('pure edge cases produce stable values', () {
    expect(recursiveChecksum(7), 16);
    expect(optionalPositional(5), 'p:10');
    expect(optionalNamed(label: 'edge', enabled: false), 'disabled');
    expect(genericFirst<int>(<int>[9, 4]), 9);
    expect(closurePipeline(1), 13);
    expect(
      classifyRecord((label: 'ok', score: 21, succeeded: true)),
      'large:ok',
    );
    expect(EdgeVector.parse('3,4').magnitudeSquared, 25);
  });

  testWidgets('fixture renders recovered edge-case output', (tester) async {
    await tester.pumpWidget(const EdgeCaseApp());
    await tester.pumpAndSettle();

    expect(find.text('Clutter edge-case fixture'), findsOneWidget);
    expect(find.textContaining('recursive:16'), findsOneWidget);
    expect(find.textContaining('stream:0'), findsOneWidget);
  });
}
