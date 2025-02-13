import 'package:integration_test/integration_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nostr_mls_package/nostr_mls_package.dart';
import 'package:path_provider/path_provider.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  test('mls', () async {
    final directory = await getApplicationDocumentsDirectory();
    await initNostrMls(path: directory.path);
    String encodedKeyPackage = await createKeyPackageForEvent(publicKey: 'b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f');
    String decodedKeyPackage = await parseKeyPackage(encodedKeyPackage: encodedKeyPackage);
    String result = await deleteKeyPackageFromStorage(encodedKeyPackage: encodedKeyPackage);
    print(encodedKeyPackage);
    print(decodedKeyPackage);
    print(result);

    String createGroupReslt = await createGroup(groupName: 'group name', groupDescription: 'group descriptions', groupMembersKeyPackages: [encodedKeyPackage], groupCreatorPublicKey: 'b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f', groupAdminPublicKeys: ['b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f'], relays: ['wss://example.com']);
    print(createGroupReslt);
  });
}
