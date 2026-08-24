// GENERATED AOT PSEUDOCODE — NOT ORIGINAL SOURCE.
// ignore_for_file: unused_element, unused_import, non_constant_identifier_names
// Recovered from: package:simple_app/main.dart

import '../support/aot_intrinsics.dart' as aot;

const String recoveredSourceUri = 'package:simple_app/main.dart';

dynamic recoveredEntry(List<dynamic> args) {
  return aot.unresolvedRegion(recoveredSourceUri, args);
}
/// Dart VM retained declaration object unknown; no distinct executable body survived.
void main() => throw UnsupportedError('AOT body unavailable');

/// Partially reconstructed `package:simple_app/main.dart._CatalogPageState`.
dynamic package_simple_app_main_dart__CatalogPageState(List<dynamic> args) {
  return aot.unresolvedRegion('Remaining behavior of package:simple_app/main.dart._CatalogPageState', <dynamic>[]);
}

/// Partially reconstructed from package:simple_app/main.dart near line 5.
/// Inlined by the optimizer (statements live inside this body):
///   main
void main_tearOff() {
  final runAppResult = runApp();
  return;
  // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
  // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
}

class CatalogPage extends StatefulWidget {

  /// Partially reconstructed from package:simple_app/main.dart near line 28.
  /// Inlined by the optimizer (statements live inside this body):
  ///   _CatalogPageState.
  ///   Cart.
  dynamic createState() {
    // Recovered source literals:
    //   line 28: 'Ready'
    final catalogPageStateResult = _CatalogPageState(aot.snapshotRef(23425));
    catalogPageStateResult._slot_18 = aot.snapshotRef(903); // AOT field store +0x18
    catalogPageStateResult._slot_1c = 'Ready'; // AOT field store +0x1c
    final fromLiteralResult = aot.invoke('Map._fromLiteral', <dynamic>[catalogPageStateResult, aot.snapshotRef(23225)]);
    final cartResult = Cart();
    cartResult._slot_8 = fromLiteralResult; // AOT field store +0x8
    catalogPageStateResult._slot_14 = cartResult; // AOT field store +0x14
    return catalogPageStateResult;
    // Control-flow evidence: 2 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}

class ShopDemoApp extends StatelessWidget {

  /// Partially reconstructed from package:simple_app/main.dart near line 17.
  dynamic build(dynamic context) {
    // Recovered source literals:
    //   line 12: 'Clutter Shop Demo'
    //   line 12: 'US'
    final colorScheme = ColorScheme.fromSeed(null, const MaterialColor(), aot.snapshotRef(48150));
    final themeData = ThemeData(null, aot.snapshotRef(48234), colorScheme);
    final materialAppResult = MaterialApp();
    return materialAppResult;
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}

class _CatalogPageState extends State<CatalogPage> {
  /// Dart VM retained declaration object unknown; no distinct executable body survived.
  static _CatalogPageState create_unknown(List<dynamic> args) => throw UnsupportedError('AOT body unavailable');

  /// Partially reconstructed from package:simple_app/main.dart near line 74.
  /// Inlined by the optimizer (statements live inside this body):
  ///   _GrowableList._literal2
  ///   add
  dynamic build(dynamic context) {

    /// Closure recovered from package:simple_app/main.dart near line 77.
    bool closureAtLine77(Product arg0) {
      return !(arg0._slot_10 >= 10.0);
    }

    /// Closure recovered from package:simple_app/main.dart near line 87.
    void closureAtLine87(String arg0) {
      final setStateResult =
        aot.invoke('State.setState', <dynamic>[local10, aot.unresolvedValue('shared-code result')]);
      return;
    }

    /// Closure recovered from package:simple_app/main.dart near line 88.
    void closureAtLine88() {
      if ((x16 & x28) == 0) {  /* low-confidence predicate */
        return;
      } else {
      }
    }

    /// Closure recovered from package:simple_app/main.dart near line 100.
    ListTile closureAtLine100(BuildContext arg0, int arg1) {
      final toStringResult = aot.invoke('Product.toString', <dynamic>[aot.unresolvedValue('shared-code result')]);
      final textResult = Text();
      final textResult2 = Text(textResult);
      final iconButtonResult = IconButton();
      final listTileResult = ListTile(iconButtonResult, const _IconButtonVariant());
      return listTileResult;
    }

    /// Closure recovered from package:simple_app/main.dart near line 107.
    void closureAtLine107() {
      final addResult = _add();
      return;
    }

    /// Closure recovered from package:simple_app/main.dart near line 116.
    void closureAtLine116() {
      final checkoutResult = _checkout();
      return checkoutResult;
    }
    // Recovered source literals:
    //   line 87: '•'
    //   line 96: ' at '
    //   line 96: 'Deal: '
    //   line 117: 'Pay '
    final get__filteredProductsResult = _filteredProducts;
    final firstWhereOrNullResult =
      firstWhereOrNull(aot.snapshotRef(55), aot.snapshotRef(23385), get__filteredProductsResult, aot.unresolvedValue('shared-code result'));
    final appBar = AppBar();
    final textFieldResult = TextField();
    final paddingResult = Padding(false, true, const TextInputType());
    final themeResult = Theme.of(arg1, paddingResult);
    final textResult = Text();
    if (firstWhereOrNullResult == null) {  /* low-confidence predicate */
      final listView = ListView.builder(aot.unresolvedValue('shared-code result'), local50);
      final expandedResult = Expanded(aot.snapshotRef(22966));
      if (x3 != x1) {  /* low-confidence predicate */
        final columnResult = Column(aot.unresolvedValue('shared-code result'), local50, this, appBar);
        final get_subtotalResult = aot.unresolvedValue('receiver').subtotal;
        final formatPriceResult = formatPrice();
        final textResult2 = Text();
        final floatingActionButtonResult = FloatingActionButton(textResult2);
        final scaffoldResult = Scaffold(true, false, textResult2);
        return scaffoldResult;
      } else {
        final growToNextCapacityResult =
          aot.invoke('List._growToNextCapacity', <dynamic>[aot.unresolvedValue('shared-code result'), expandedResult]);
      }
    } else {
      final formatPriceResult2 = formatPrice(firstWhereOrNullResult);
      final textResult3 = Text();
      if (x3 != x1) {  /* low-confidence predicate */
      } else {
        final growToNextCapacityResult2 =
          aot.invoke('List._growToNextCapacity', <dynamic>[aot.unresolvedValue('shared-code result'), textResult3]);
      }
    }
    // Dynamic-call evidence:
    //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
    // 3 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
  }

  /// Partially reconstructed from package:simple_app/main.dart near line 40.
  /// Inlined by the optimizer (statements live inside this body):
  ///   toList
  ///   List.of
  dynamic get _filteredProducts {

    /// Closure recovered from package:simple_app/main.dart near line 41.
    /// Inlined by the optimizer (statements live inside this body):
    ///   toLowerCase
    ///   _allocate
    bool closureAtLine41(Product arg0) {
      while (x5 < x3) {
        if (x6 != x7) {  /* low-confidence predicate */
        } else {
        }
      }
      final allocateOneByteStringResult =
        allocateOneByteString(aot.unresolvedValue('"\\0\\u{1}\\u{2}\\u{3}\\u{4}\\u{5}\\u{6}\\u{7}\\u{8}\\t\\n\\u{b}\\u{c}\\r\\u{e}\\u{f}\\u{10}\\u{11}\\u{12}\\u{13}\\u{14}\\u{15}\\u{16}\\u{17}\\u{18}\\u{19}\\u{1a}\\u{1b}\\u{1c}\\u{1d}\\u{1e}\\u{1f} !\\"#\$%&\'()*+,-./0123456789:;<=>?@abcdefghijklmnopqrstuvwxyz[\\\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\\u{7f}\\u{80}\\u{81}\\u{82}\\u{83}\\u{84}\\u{85}\\u{86}\\u{87}\\u{88}\\u{89}\\u{8a}\\u{8b}\\u{8c}\\u{8d}\\u{8e}\\u{8f}\\u{90}\\u{91}\\u{92}\\u{93}\\u{94}\\u{95}\\u{96}\\u{97}\\u{98}\\u{99}\\u{9a}\\u{9b}\\u{9c}\\u{9d}\\u{9e}\\u{9f}"…'));
      while (0 < local8) {
        if (0 >= local10) {  /* low-confidence predicate */
        } else {
        }
      }
      if (x4 >= x0) {  /* low-confidence predicate */
        final containsResult = aot.invoke('String.contains', <dynamic>[aot.snapshotRef(48150)]);
        return containsResult;
      } else {
      }
    }
    // Recovered source literals:
    //   line 40: 'Aged Cheese'
    //   line 40: 'Apple'
    //   line 40: 'Avocado'
    //   line 40: 'Sourdough Bread'
    //   line 40: 'Whole Milk'
    return aot.snapshotRef(48403);
    // Dynamic-call evidence:
    //   .<unknown selector>(...) at 1 site(s): candidate set unresolved.
    // Control-flow evidence: 2 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }

  /// Partially reconstructed from package:simple_app/main.dart near line 52.
  dynamic _checkout(List<dynamic> args) async {

    /// Closure recovered from package:simple_app/main.dart near line 53.
    void closureAtLine53() {
      // Recovered source literals:
      //   line 53: 'Cart is empty'
      return;
    }

    /// Closure recovered from package:simple_app/main.dart near line 58.
    void closureAtLine58() {
      // Recovered source literals:
      //   line 58: 'Processing...'
      return;
    }

    /// Closure recovered from package:simple_app/main.dart near line 66.
    void closureAtLine66() {
      // Recovered source literals:
      //   line 67: ' for '
      //   line 67: ' items'
      //   line 67: 'Paid '
      final get_itemCountResult = local10.itemCount;
      final clearResult = aot.invoke('Cart.clear', <dynamic>[]);
      return;
    }
    final get_isEmptyResult = aot.unresolvedValue('receiver').isEmpty;
    if (!(get_isEmptyResult)) {  /* medium-confidence predicate */
      final setStateResult = setState(aot.unresolvedValue('shared-code result'));
      final future = Future.delayed(aot.snapshotRef(23591), const Duration());
      if (x1 != null) {  /* low-confidence predicate */
        final get_subtotalResult = local20.subtotal;
        final formatPriceResult = formatPrice();
        final setStateResult2 = setState(aot.unresolvedValue('shared-code result'));
      } else {
      }
    } else {
      final setStateResult3 = setState(aot.unresolvedValue('shared-code result'));
    }
    // Body compiled to an async state machine; await boundaries are unnamed in this snapshot.
    // 2 branch region(s) and 0 loop(s) reconstructed; exact machine structure remains in reports/functions.json.
    return aot.unresolvedRegion('Remaining behavior of _CatalogPageState._checkout', <dynamic>[]);
  }

  /// Partially reconstructed from package:simple_app/main.dart near line 45.
  dynamic _add(List<dynamic> args) {

    /// Closure recovered from package:simple_app/main.dart near line 45.
    void closureAtLine45() {
      // Recovered source literals:
      //   line 47: 'Added '
      final addResult = aot.invoke('Cart.add', <dynamic>[]);
      return;
    }
    final setStateResult = setState(aot.unresolvedValue('shared-code result'));
    return;
    // Control-flow evidence: 1 conditional branch(es), 1 loop back-edge(s), 0 exception handler(s).
    // Remaining block structure is preserved in reports/functions.json and reports/assembly.s.
  }
}
