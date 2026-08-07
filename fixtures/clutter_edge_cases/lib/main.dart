import 'package:flutter/material.dart';

// Importing the library makes its VM entry-point pragmas visible to the AOT
// compiler without calling those functions from Dart.
// ignore: unused_import
import 'entry_points.dart';
import 'widgets/edge_case_screen.dart';

void main() {
  runApp(const EdgeCaseApp());
}

class EdgeCaseApp extends StatelessWidget {
  const EdgeCaseApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Clutter recovery fixture',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.teal),
      ),
      home: const EdgeCaseScreen(),
    );
  }
}
