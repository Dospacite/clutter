// GENERATED SUPPORT FOR CONSERVATIVE AOT PSEUDOCODE.

Never _unresolved(String kind, Object? evidence) =>
    throw UnsupportedError('Unresolved AOT operation: $kind ($evidence)');

dynamic unresolvedRegion(String sourceUri, List<dynamic> args) =>
    _unresolved('region', <Object?>[sourceUri, args]);

dynamic unresolvedValue(String description) =>
    _unresolved('value', description);

dynamic unresolvedRegister(String register) =>
    _unresolved('register', register);

dynamic invoke(String target, List<dynamic> arguments) =>
    _unresolved('call', <Object?>[target, arguments]);

dynamic unknownOperation(
  String address,
  String bytes,
  List<dynamic> inputs,
) =>
    _unresolved('instruction', <Object?>[address, bytes, inputs]);

dynamic snapshotRef(int reference) =>
    _unresolved('snapshot-object', reference);

dynamic nativePoolEntry(int index) =>
    _unresolved('native-pool-entry', index);

dynamic resetPoolEntry(int index) =>
    _unresolved('reset-pool-entry', index);
