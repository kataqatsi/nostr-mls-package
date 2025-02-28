import 'dart:convert';

import 'package:integration_test/integration_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nostr_mls_package/nostr_mls_package.dart';
import 'package:path_provider/path_provider.dart';

String toHexString(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join('');
}

class MlsGroup {
  final List<int> groupId;
  final List<String> groupMembers;
  final List<int> serializedWelcomeMessage;
  final NostrGroupData nostrGroupData;

  MlsGroup({
    required this.groupId,
    required this.groupMembers,
    required this.serializedWelcomeMessage,
    required this.nostrGroupData,
  });

  static MlsGroup fromJson(Map<String, dynamic> json) {
    if (json['group_id'] == null) {
      throw Exception("error: group_id does not exit");
    }

    List<dynamic> groupIdVec = json['group_id']['value']['vec'];
    List<int> groupId = groupIdVec.map((e) => e as int).toList();

    List<String> members = [];
    if (json['members'] != null) {
      members = List<String>.from(json['members']);
    }

    List<int> serializedWelcomeMessage = List<int>.from(json['serialized_welcome_message']);

    NostrGroupData nostrGroupData = NostrGroupData.fromJson(json['nostr_group_data']);

    return MlsGroup(
      groupId: groupId,
      groupMembers: members,
      serializedWelcomeMessage: serializedWelcomeMessage,
      nostrGroupData: nostrGroupData,
    );
  }
}

class NostrGroupData {
  final String nostrGroupId;
  final String name;
  final String description;
  final List<String> adminPubkeys;
  final List<String> relays;

  NostrGroupData({
    required this.nostrGroupId,
    required this.name,
    required this.description,
    required this.adminPubkeys,
    required this.relays,
  });

  static NostrGroupData fromJson(Map<String, dynamic> json) {
    List<int> nostrGroupIdInts = List<int>.from(json['nostr_group_id']);
    String nostrGroupId = toHexString(nostrGroupIdInts);

    List<int> nameInts = List<int>.from(json['name']);
    String name = utf8.decode(nameInts);

    List<int> descriptionInts = List<int>.from(json['description']);
    String description = utf8.decode(descriptionInts);

    List<String> adminPubkeys = (json['admin_pubkeys'] as List)
        .map((item) => utf8.decode(List<int>.from(item))).toList();

    List<String> relays = (json['relays'] as List)
        .map((item) => utf8.decode(List<int>.from(item))).toList();

    return NostrGroupData(
      nostrGroupId: nostrGroupId,
      name: name,
      description: description,
      adminPubkeys: adminPubkeys,
      relays: relays,
    );
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  test('mls', () async {
    final directory = await getApplicationDocumentsDirectory();
    await initNostrMls(path: directory.path);

    String alice_pubkey = '3b88ecd9164822437aa8723ebaf224ebda13768cc82bb05785d6a1c8b36a0337';
    String alice_privkey = '233a778afd756800801a619904e62e99e93fc4f2e4df0343f63d4c02dabc0a9e';
    String bob_pubkey = 'aa1c02218a8b920d42844cfbf959f3a65d7842a991a709e1d462b1ff3f511769';
    String bob_privkey = '8434f08f7a8e49c1b934d52988940ccacfcb938292313ff96d581b941ccddcf4';

    String encodedKeyPackage = await createKeyPackageForEvent(
      publicKey:bob_pubkey,
    );

    String createGroupResult = await createGroup(
      groupName: 'group name',
      groupDescription: 'group descriptions',
      groupMembersKeyPackages: [encodedKeyPackage],
      groupCreatorPublicKey:alice_pubkey,
      groupAdminPublicKeys: [alice_pubkey],
      relays: ['wss://example.com'],
    );

    MlsGroup mlsGroup = MlsGroup.fromJson(jsonDecode(createGroupResult));
    // print("Group ID: ${mlsGroup.groupId}");
    // print("Group Members: ${mlsGroup.groupMembers}");
    // print("Serialized Welcome Message: ${mlsGroup.serializedWelcomeMessage}");
    // print("Nostr Group Data:${mlsGroup.nostrGroupData.nostrGroupId}, ${mlsGroup.nostrGroupData.name}, ${mlsGroup.nostrGroupData.description}, ${mlsGroup.nostrGroupData.adminPubkeys}, ${mlsGroup.nostrGroupData.relays}");

    String preview = await previewWelcomeEvent(serializedWelcomeMessage: mlsGroup.serializedWelcomeMessage);
    print(preview);

    return;

    String joinGroupResult = await joinGroupFromWelcome(serializedWelcomeMessage: mlsGroup.serializedWelcomeMessage);
    MlsGroup joinmlsGroup = MlsGroup.fromJson(jsonDecode(joinGroupResult));
    // print("Group Members: ${joinmlsGroup.groupMembers}");
    // print("Nostr Group Data:${joinmlsGroup.nostrGroupData.nostrGroupId}, ${joinmlsGroup.nostrGroupData.name}, ${joinmlsGroup.nostrGroupData.description}, ${joinmlsGroup.nostrGroupData.adminPubkeys}, ${joinmlsGroup.nostrGroupData.relays}");

    List<int> serializedMessage = await createMessageForGroup(groupId: mlsGroup.groupId, messageEvent: 'hello');
    List<int> message = await processMessageForGroup(groupId: mlsGroup.groupId, serializedMessage: serializedMessage);
    print(utf8.decode(message));
  });
}