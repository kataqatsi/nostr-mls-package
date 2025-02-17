import 'dart:convert';

import 'package:integration_test/integration_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:nostr_mls_package/nostr_mls_package.dart';
import 'package:path_provider/path_provider.dart';

String toHexString(List<int> bytes) {
  return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join('');
}

class MlsGroup {
  final String groupId;
  final List<String> groupMembers;
  final String serializedWelcomeMessage;
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
    List<int> groupIdInts = groupIdVec.map((e) => e as int).toList();
    String groupId = toHexString(groupIdInts);

    List<String> members = [];
    if (json['members'] != null) {
      members = List<String>.from(json['members']);
    }

    List<int> welcomeMessageInts = List<int>.from(json['serialized_welcome_message']);
    String serializedWelcomeMessage = toHexString(welcomeMessageInts);

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

    String encodedKeyPackage = await createKeyPackageForEvent(
      publicKey:
      'b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f',
    );

    String createGroupResult = await createGroup(
      groupName: 'group name',
      groupDescription: 'group descriptions',
      groupMembersKeyPackages: [encodedKeyPackage],
      groupCreatorPublicKey:
      'b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f',
      groupAdminPublicKeys: [
        'b3e43e8cc7e6dff23a33d9213a3e912d895b1c3e4250240e0c99dbefe3068b5f'
      ],
      relays: ['wss://example.com'],
    );

    MlsGroup mlsGroup = MlsGroup.fromJson(jsonDecode(createGroupResult));
    print("Group ID: ${mlsGroup.groupId}");
    print("Group Members: ${mlsGroup.groupMembers}");
    print("Serialized Welcome Message: ${mlsGroup.serializedWelcomeMessage}");
    print("Nostr Group Data:${mlsGroup.nostrGroupData.nostrGroupId}, ${mlsGroup.nostrGroupData.name}, ${mlsGroup.nostrGroupData.description}, ${mlsGroup.nostrGroupData.adminPubkeys}, ${mlsGroup.nostrGroupData.relays}");
  });
}