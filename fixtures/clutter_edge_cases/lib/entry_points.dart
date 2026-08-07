@pragma('vm:entry-point')
String retainedTopLevelEntrypoint(int value) => 'retained:${value * 13}';

abstract final class NativeEntryPoints {
  @pragma('vm:entry-point')
  static int retainedStaticEntrypoint(int left, int right) => left ^ right;
}

// This is intentionally unreachable in the release application. The recovery
// evaluator expects Dart's tree shaker to remove it from the AOT snapshot.
String treeShakenSentinel() => 'this function should not survive';
