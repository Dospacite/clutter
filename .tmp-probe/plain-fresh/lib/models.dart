// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.
// ignore_for_file: unused_element, unused_import, non_constant_identifier_names
// Recovered from: package:simple_app/models.dart

import '../support/aot_intrinsics.dart' as aot;

const String recoveredSourceUri = 'package:simple_app/models.dart';

/// Partially reconstructed from package:simple_app/models.dart near line 111.
dynamic formatPrice(List<dynamic> args) {
  final toStringAsFixedResult = aot.invoke('_Double.toStringAsFixed', <dynamic>[local10, 2]);
  return '${snapshotRef(371)}${toStringAsFixedResult}';
  // Control-flow evidence: 4 conditional branch(es), 2 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

/// Partially reconstructed from package:simple_app/models.dart near line 102.
dynamic firstWhereOrNull(List<dynamic> args) {
  while (x0 != true) {
  }
  return local10;
  // Dynamic-call evidence:
  //   .<unknown selector>(...) at 3 site(s): candidate set unresolved.
  //   1 register-indirect call site(s) remain unresolved.
  // 1 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
}

/// Partially reconstructed `package:simple_app/models.dart.CartLine`.
dynamic package_simple_app_models_dart_CartLine(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:simple_app/models.dart.CartLine', <dynamic>[]);
}

/// Partially reconstructed `package:simple_app/models.dart.Cart`.
dynamic package_simple_app_models_dart_Cart(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:simple_app/models.dart.Cart', <dynamic>[]);
}

class Cart {
  /// Dart VM retained declaration object unknown; no distinct executable body survived.
  static Cart create_unknown(List<dynamic> args) => throw UnsupportedError('AOT body unavailable');

  /// Partially reconstructed from package:simple_app/models.dart near line 60.
  /// Inlined by the optimizer (statements live inside this body):
  ///   values
  ///   current
  dynamic get subtotal {
    final compactValuesIterableResult =
      aot.invoke('dart:_compact_hash._CompactValuesIterable', <dynamic>[aot.snapshotRef(23048)]);
    final get_iteratorResult = compactValuesIterableResult.iterator;
    while (moveNextResult) {
      final moveNextResult = aot.invoke('_CompactIterator.moveNext', <dynamic>[get_iteratorResult]);
      if (x4 != null) {  /* low-confidence predicate */
      } else {
        if (local8 == null) {  /* low-confidence predicate */
        } else {
        }
      }
    }
    return moveNextResult;
    // Dynamic-call evidence:
    //   1 register-indirect call site(s) remain unresolved.
    // 3 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed `Cart.get:isEmpty`.
  dynamic get isEmpty {
    // Control-flow evidence: 1 conditional branch(es), 0 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
    return aot.unresolvedRegion('Remaining behavior of Cart.get:isEmpty', <dynamic>[]);
  }

  /// Partially reconstructed from package:simple_app/models.dart near line 83.
  dynamic clear(List<dynamic> args) {
    final clearResult =
      aot.invoke('__Map&_HashVMBase&MapMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashMapMixin.clear', <dynamic>[]);
    return;
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }

  /// Partially reconstructed from package:simple_app/models.dart near line 52.
  /// Inlined by the optimizer (statements live inside this body):
  ///   values
  ///   current
  dynamic get itemCount {
    final compactValuesIterableResult =
      aot.invoke('dart:_compact_hash._CompactValuesIterable', <dynamic>[aot.snapshotRef(23048)]);
    final get_iteratorResult = compactValuesIterableResult.iterator;
    while (moveNextResult) {
      final moveNextResult = aot.invoke('_CompactIterator.moveNext', <dynamic>[get_iteratorResult]);
      if (x4 != null) {  /* low-confidence predicate */
      } else {
        if (local8 == null) {  /* low-confidence predicate */
        } else {
        }
      }
    }
    return local10;
    // Dynamic-call evidence:
    //   1 register-indirect call site(s) remain unresolved.
    // 3 branch region(s) and 1 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed from package:simple_app/models.dart near line 70.
  /// Inlined by the optimizer (statements live inside this body):
  ///   []
  dynamic add(List<dynamic> args) {
    final lookupResult = aot.invoke('Map.lookup', <dynamic>[]);
    if (x2 != lookupResult) {  /* low-confidence predicate */
      if (x0 != null) {  /* low-confidence predicate */
        final cartLineResult = CartLine(arg1);
        cartLineResult._slot_8 = arg1; // AOT field store +0x8
        cartLineResult._slot_c = local20; // AOT field store +0xc
        final updatedItemResult =
          aot.invoke('__Map&_HashVMBase&MapMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashMapMixin.[]=', <dynamic>[local10, local8, cartLineResult]);
        return;
      } else {
        final cartLineResult2 = CartLine(local10);
        cartLineResult2._slot_8 = arg1; // AOT field store +0x8
        cartLineResult2._slot_c = 1; // AOT field store +0xc
        final updatedItemResult2 =
          aot.invoke('__Map&_HashVMBase&MapMixin&_HashBase&_OperatorEqualsAndHashCode&_LinkedHashMapMixin.[]=', <dynamic>[local10, local8, cartLineResult2]);
      }
    } else {
    }
    // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }
}

class CartLine {
}

/// Snapshot class flags identify this declaration as an enum; values may be tree-shaken.
abstract class Category extends _Enum {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  num _slot_8; // AOT slot +0x8; unboxed_field
  num _slot_c; // AOT slot +0xc; unboxed_field

  /// Partially reconstructed `Category._enumToString`.
  dynamic _enumToString(List<dynamic> args) {
    // Recovered source literals:
    //   'Category.'
    return 'Category.${aot.unresolvedValue('interpolated part')}';
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}

class Product {
  /// AOT instance slots whose original Field declaration was tree-shaken.
  num _slot_10; // AOT slot +0x10; unboxed_field
  num _slot_14; // AOT slot +0x14; unboxed_field

  /// Partially reconstructed from package:simple_app/models.dart near line 31.
  String toString() {
    // Recovered source literals:
    //   line 31: ' (#'
    //   line 31: ') \$'
    final toStringAsFixedResult =
      aot.invoke('_Double.toStringAsFixed', <dynamic>[this._slot_10, 2, aot.unresolvedValue('shared-code result')]);
    return '${local8} (#${aot.unresolvedValue('interpolated part')}) \$${toStringAsFixedResult}';
    // Control-flow evidence: 4 conditional branch(es), 2 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}
