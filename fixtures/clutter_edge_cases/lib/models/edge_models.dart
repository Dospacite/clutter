sealed class ArithmeticOperation {
  const ArithmeticOperation();

  int apply(int left, int right);
}

final class AddOperation extends ArithmeticOperation {
  const AddOperation();

  @override
  int apply(int left, int right) => left + right;
}

final class MultiplyOperation extends ArithmeticOperation {
  const MultiplyOperation();

  @override
  int apply(int left, int right) => left * right;
}

enum EdgeFlavor {
  plain('plain', 1),
  spicy('spicy', 3);

  const EdgeFlavor(this.label, this.weight);

  final String label;
  final int weight;
}

class GenericBox<T extends Object> {
  const GenericBox(this.value);

  final T value;

  R map<R extends Object>(R Function(T value) convert) => convert(value);
}

mixin AuditTrail {
  final List<String> _events = <String>[];

  void recordEvent(String event) => _events.add(event);

  String get auditSummary => _events.join('|');
}

abstract interface class Describable {
  String describe();
}

class EdgeVector with AuditTrail implements Describable {
  EdgeVector(this.x, this.y);

  factory EdgeVector.parse(String source) {
    final parts = source.split(',');
    return EdgeVector(int.parse(parts.first), int.parse(parts.last));
  }

  final int x;
  final int y;

  EdgeVector operator +(EdgeVector other) =>
      EdgeVector(x + other.x, y + other.y);

  int get magnitudeSquared => (x * x) + (y * y);

  @override
  String describe() => 'vector($x,$y)';
}

extension WordMetrics on String {
  int get vowelCount {
    const vowels = <String>{'a', 'e', 'i', 'o', 'u'};
    return toLowerCase().split('').where(vowels.contains).length;
  }

  String bracketed({String left = '[', String right = ']'}) =>
      '$left$this$right';
}

typedef EdgeResult = ({String label, int score, bool succeeded});

class EdgeFailure implements Exception {
  const EdgeFailure(this.message);

  final String message;

  @override
  String toString() => 'EdgeFailure: $message';
}
