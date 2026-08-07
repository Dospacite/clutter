import 'dart:async';

import '../models/edge_models.dart';

int recursiveChecksum(int value) {
  if (value <= 1) {
    return value;
  }
  return value + recursiveChecksum(value - 2);
}

String optionalPositional(
  int required, [
  int multiplier = 2,
  String tag = 'p',
]) {
  return '$tag:${required * multiplier}';
}

String optionalNamed({
  required String label,
  int count = 4,
  bool enabled = true,
}) {
  return enabled ? '$label:$count' : 'disabled';
}

T genericFirst<T extends Object>(List<T> values) => values.first;

String classifyRecord(EdgeResult result) => switch (result) {
  (score: > 20, label: final label, succeeded: true) => 'large:$label',
  (score: > 0, label: final label, succeeded: true) => 'small:$label',
  (succeeded: false, label: final label, score: _) => 'failed:$label',
  _ => 'empty',
};

int closurePipeline(int seed) {
  var mutableCapture = seed;
  int captureAndIncrement(int delta) {
    mutableCapture += delta;
    return mutableCapture;
  }

  final tearOff = captureAndIncrement;
  return <int>[1, 2, 3].map(tearOff).fold(0, (sum, item) => sum + item);
}

Stream<String> countedStream(int count) async* {
  for (var index = 0; index < count; index++) {
    await Future<void>.delayed(Duration.zero);
    yield 'stream:$index';
  }
}

Future<String> guardedAsyncValue(int seed) async {
  var finalMarker = 'not-finalized';
  try {
    await Future<void>.delayed(Duration.zero);
    if (seed.isNegative) {
      throw const EdgeFailure('negative seed');
    }
    return 'async:${seed ~/ 3}';
  } on EdgeFailure catch (error, stackTrace) {
    return 'caught:${error.message}:${stackTrace.hashCode.isEven}';
  } finally {
    finalMarker = 'finalized';
    assert(finalMarker.isNotEmpty);
  }
}

class EdgeCaseHarness {
  const EdgeCaseHarness();

  Future<List<String>> run(int runtimeSeed) async {
    final operation = runtimeSeed.isEven
        ? const AddOperation()
        : const MultiplyOperation();
    final vector = EdgeVector.parse('3,4') + EdgeVector(runtimeSeed % 5, 2);
    vector.recordEvent('created');

    final boxed = GenericBox<EdgeVector>(vector);
    final record = (
      label: boxed.map((item) => item.describe()),
      score: operation.apply(vector.magnitudeSquared, EdgeFlavor.spicy.weight),
      succeeded: true,
    );

    final streamValues = <String>[];
    await for (final value in countedStream(2)) {
      streamValues.add(value);
    }

    final positionalTearOff = optionalPositional;
    final namedTearOff = optionalNamed;
    final unicodeAndEscapes =
        'Καλημέρα\n${'Flutter'.bracketed(left: '<', right: '>')}';

    return <String>[
      classifyRecord(record),
      optionalPositional(runtimeSeed),
      optionalPositional(runtimeSeed, 3),
      optionalPositional(runtimeSeed, 3, 'positional'),
      optionalNamed(label: 'defaults'),
      optionalNamed(label: 'named', count: runtimeSeed % 7),
      optionalNamed(label: 'disabled', enabled: false),
      'apply:${Function.apply(positionalTearOff, <dynamic>[runtimeSeed])}',
      'applyNamed:${Function.apply(namedTearOff, const <dynamic>[], <Symbol, dynamic>{#label: 'applied'})}',
      'generic:${genericFirst<String>(<String>['first', 'second'])}',
      'closure:${closurePipeline(runtimeSeed % 9)}',
      'recursive:${recursiveChecksum(7)}',
      await guardedAsyncValue(runtimeSeed),
      ...streamValues,
      'vowels:${unicodeAndEscapes.vowelCount}',
      vector.auditSummary,
    ];
  }
}

String privateLibraryHelper(String input) => input.split('').reversed.join();
