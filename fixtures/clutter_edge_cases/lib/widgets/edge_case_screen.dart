import 'dart:async';

import 'package:flutter/material.dart';

import '../services/edge_case_harness.dart';

class EdgeCaseScreen extends StatefulWidget {
  const EdgeCaseScreen({super.key});

  @override
  State<EdgeCaseScreen> createState() => _EdgeCaseScreenState();
}

class _EdgeCaseScreenState extends State<EdgeCaseScreen> {
  final EdgeCaseHarness _harness = const EdgeCaseHarness();
  late Future<List<String>> _result;
  StreamSubscription<int>? _heartbeat;
  int _ticks = 0;

  @override
  void initState() {
    super.initState();
    _result = _runFixture();
    _heartbeat =
        Stream<int>.periodic(
          const Duration(seconds: 30),
          (index) => index,
        ).listen((value) {
          if (!mounted) {
            return;
          }
          setState(() => _ticks = value + 1);
        });
  }

  Future<List<String>> _runFixture() {
    final runtimeSeed = DateTime.now().microsecondsSinceEpoch;
    return _harness.run(runtimeSeed);
  }

  void _rerun() {
    setState(() {
      _result = _runFixture();
    });
  }

  @override
  void dispose() {
    unawaited(_heartbeat?.cancel());
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Clutter edge-case fixture')),
      body: FutureBuilder<List<String>>(
        future: _result,
        builder: (context, snapshot) {
          if (snapshot.hasError) {
            return Center(child: Text('error: ${snapshot.error}'));
          }
          if (!snapshot.hasData) {
            return const Center(child: CircularProgressIndicator());
          }
          return ListView(
            padding: const EdgeInsets.all(16),
            children: <Widget>[
              Text('heartbeat: $_ticks'),
              for (final line in snapshot.requireData)
                SelectableText(line, key: ValueKey<String>(line)),
            ],
          );
        },
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _rerun,
        tooltip: 'Run edge cases again',
        child: const Icon(Icons.refresh),
      ),
    );
  }
}
