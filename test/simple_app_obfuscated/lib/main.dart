import 'package:flutter/material.dart';

import 'models.dart';

void main() {
  runApp(const ShopDemoApp());
}

class ShopDemoApp extends StatelessWidget {
  const ShopDemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Clutter Shop Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.indigo),
      ),
      home: const CatalogPage(),
    );
  }
}

class CatalogPage extends StatefulWidget {
  const CatalogPage({super.key});

  @override
  State<CatalogPage> createState() => _CatalogPageState();
}

class _CatalogPageState extends State<CatalogPage> {
  final Cart _cart = Cart();
  String _query = '';
  String _status = 'Ready';

  List<Product> get _filteredProducts {
    if (_query.isEmpty) {
      return catalog;
    }
    final needle = _query.toLowerCase();
    return catalog.where((p) => p.name.toLowerCase().contains(needle)).toList();
  }

  void _add(Product product) {
    setState(() {
      _cart.add(product);
      _status = 'Added ${product.name}';
    });
  }

  Future<void> _checkout() async {
    if (_cart.isEmpty) {
      setState(() {
        _status = 'Cart is empty';
      });
      return;
    }
    setState(() {
      _status = 'Processing...';
    });
    await Future<void>.delayed(const Duration(milliseconds: 800));
    if (!mounted) {
      return;
    }
    final total = formatPrice(_cart.subtotal);
    setState(() {
      _status = 'Paid $total for ${_cart.itemCount} items';
      _cart.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    final products = _filteredProducts;
    final cheapest = firstWhereOrNull<Product>(
      products,
      (p) => !p.isExpensive,
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Shop Demo')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(8),
            child: TextField(
              decoration: const InputDecoration(labelText: 'Search'),
              onChanged: (value) {
                setState(() {
                  _query = value;
                });
              },
            ),
          ),
          Text(_status, style: Theme.of(context).textTheme.titleMedium),
          if (cheapest != null)
            Text('Deal: ${cheapest.name} at ${formatPrice(cheapest.price)}'),
          Expanded(
            child: ListView.builder(
              itemCount: products.length,
              itemBuilder: (context, index) {
                final product = products[index];
                return ListTile(
                  title: Text(product.toString()),
                  subtitle: Text(product.category.name),
                  trailing: IconButton(
                    icon: const Icon(Icons.add_shopping_cart),
                    onPressed: () => _add(product),
                  ),
                );
              },
            ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _checkout(),
        label: Text('Pay ${formatPrice(_cart.subtotal)}'),
        icon: const Icon(Icons.payment),
      ),
    );
  }
}
