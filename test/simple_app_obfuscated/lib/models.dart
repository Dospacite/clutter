// Shared domain model for the Clutter recovery test apps.
//
// The code is intentionally plain and idiomatic: it exercises classes,
// enums, fields, getters, methods with optional/named parameters, generics,
// closures, string interpolation, collections, and exceptions without any
// framework magic so decompiled output can be compared against this source.

enum Category { produce, dairy, bakery }

class Product {
  final String id;
  final String name;
  final double price;
  final Category category;

  const Product(this.id, this.name, this.price, this.category);

  Product copyWith({String? name, double? price, Category? category}) {
    return Product(
      id,
      name ?? this.name,
      price ?? this.price,
      category ?? this.category,
    );
  }

  bool get isExpensive => price >= 10.0;

  @override
  String toString() {
    return '$name (#$id) \$${price.toStringAsFixed(2)}';
  }
}

class CartLine {
  final Product product;
  final int quantity;

  const CartLine(this.product, this.quantity);

  double get subtotal => product.price * quantity;
}

class Cart {
  final Map<String, CartLine> _lines = {};

  bool get isEmpty => _lines.isEmpty;
  int get lineCount => _lines.length;

  int get itemCount {
    var total = 0;
    for (final line in _lines.values) {
      total += line.quantity;
    }
    return total;
  }

  double get subtotal {
    var total = 0.0;
    for (final line in _lines.values) {
      total += line.subtotal;
    }
    return total;
  }

  void add(Product product, {int quantity = 1}) {
    if (quantity <= 0) {
      throw ArgumentError.value(quantity, 'quantity', 'must be positive');
    }
    final existing = _lines[product.id];
    if (existing == null) {
      _lines[product.id] = CartLine(product, quantity);
    } else {
      _lines[product.id] = CartLine(product, existing.quantity + quantity);
    }
  }

  void remove(String productId) {
    _lines.remove(productId);
  }

  void clear() {
    _lines.clear();
  }

  List<CartLine> sortedLines() {
    final lines = _lines.values.toList();
    lines.sort((a, b) => a.product.name.compareTo(b.product.name));
    return lines;
  }
}

const List<Product> catalog = [
  Product('apple', 'Apple', 1.2, Category.produce),
  Product('bread', 'Sourdough Bread', 5.5, Category.bakery),
  Product('milk', 'Whole Milk', 2.3, Category.dairy),
  Product('cheese', 'Aged Cheese', 12.0, Category.dairy),
  Product('avocado', 'Avocado', 3.1, Category.produce),
];

T? firstWhereOrNull<T>(Iterable<T> source, bool Function(T) test) {
  for (final element in source) {
    if (test(element)) {
      return element;
    }
  }
  return null;
}

String formatPrice(double value, {String symbol = r'$', int decimals = 2}) {
  return '$symbol${value.toStringAsFixed(decimals)}';
}
