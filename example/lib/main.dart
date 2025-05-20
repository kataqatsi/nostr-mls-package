import 'package:flutter/material.dart';
import 'package:nostr_mls_package/nostr_mls_package.dart';
import 'dart:convert';
import 'package:path_provider/path_provider.dart';
import 'dart:io';
import 'dart:typed_data';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Nostr MLS Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
        useMaterial3: true,
      ),
      home: const MLSDemo(),
    );
  }
}

class MLSDemo extends StatefulWidget {
  const MLSDemo({super.key});

  @override
  State<MLSDemo> createState() => _MLSDemoState();
}

class _MLSDemoState extends State<MLSDemo> {
  bool _initialized = false;
  String _mlsStatus = "Not initialized";
  String _ciphersuite = "";
  List<String> _extensions = [];
  String _keyPackage = "";
  String _groupInfo = "";
  List<int>? _currentGroupId;
  
  // Message related
  final _messageController = TextEditingController();
  final List<Map<String, dynamic>> _messages = [];
  final _receiveMessageController = TextEditingController();
  String _messageStatus = "";
  Uint8List? _lastSentMessage;
  
  final _publicKeyController = TextEditingController();
  final _groupNameController = TextEditingController(text: "Test Group");
  final _groupDescController = TextEditingController(text: "This is an MLS test group");
  
  @override
  void initState() {
    super.initState();
    _initMLS();
  }
  
  @override
  void dispose() {
    _publicKeyController.dispose();
    _groupNameController.dispose();
    _groupDescController.dispose();
    _messageController.dispose();
    _receiveMessageController.dispose();
    super.dispose();
  }
  
  Future<void> _initMLS() async {
    try {
      final directory = await getApplicationDocumentsDirectory();
      final path = '${directory.path}/nostr_mls_storage';
      
      // Ensure directory exists
      final dir = Directory(path);
      if (!await dir.exists()) {
        await dir.create(recursive: true);
      }
      
      // Sample identity, in a real app this should be the user's Nostr private key
      const sampleIdentity = "sample_identity_123";
      
      await initNostrMls(path: path, identity: sampleIdentity);
      
      // Get basic information
      final ciphersuite = getCiphersuite();
      final extensions = getExtensions();
      
      setState(() {
        _initialized = true;
        _mlsStatus = "Initialized";
        _ciphersuite = ciphersuite;
        _extensions = extensions;
      });
    } catch (e) {
      setState(() {
        _mlsStatus = "Initialization failed: $e";
      });
    }
  }
  
  Future<void> _createKeyPackage() async {
    if (!_initialized) {
      _showMessage("Please initialize MLS first");
      return;
    }
    
    final publicKey = _publicKeyController.text.trim();
    if (publicKey.isEmpty) {
      _showMessage("Please enter a public key");
      return;
    }
    
    try {
      final keyPackage = await createKeyPackageForEvent(publicKey: publicKey);
      setState(() {
        _keyPackage = keyPackage;
      });
      _showMessage("Key package created successfully");
    } catch (e) {
      _showMessage("Failed to create key package: $e");
    }
  }
  
  Future<void> _createGroup() async {
    if (!_initialized) {
      _showMessage("Please initialize MLS first");
      return;
    }
    
    if (_keyPackage.isEmpty) {
      _showMessage("Please create a key package first");
      return;
    }
    
    final publicKey = _publicKeyController.text.trim();
    final groupName = _groupNameController.text.trim();
    final groupDesc = _groupDescController.text.trim();
    
    if (publicKey.isEmpty || groupName.isEmpty) {
      _showMessage("Public key and group name cannot be empty");
      return;
    }
    
    try {
      final result = await createGroup(
        groupName: groupName,
        groupDescription: groupDesc,
        groupMembersKeyPackages: [_keyPackage],
        groupCreatorPublicKey: publicKey,
        groupAdminPublicKeys: [publicKey],
        relays: ["wss://relay.0xchat.com"],
      );
      
      setState(() {
        _groupInfo = result;
      });
      
      // Parse JSON, extract group ID
      final groupData = jsonDecode(result);
      _currentGroupId = List<int>.from(groupData['group_id']);
      
      _showMessage("Group created successfully: ${groupData['nostr_group_data']['nostr_group_id'] ?? 'Unknown ID'}");
    } catch (e) {
      _showMessage("Failed to create group: $e");
    }
  }
  
  // New: Create message method
  Future<void> _createMessage() async {
    if (_currentGroupId == null || _currentGroupId!.isEmpty) {
      _showMessage("Please create or join a group first");
      return;
    }
    
    final messageText = _messageController.text.trim();
    if (messageText.isEmpty) {
      _showMessage("Message content cannot be empty");
      return;
    }
    
    try {
      // Create a simple JSON message event
      final messageEvent = jsonEncode({
        "type": "message",
        "content": messageText,
        "sender": _publicKeyController.text.trim(),
        "timestamp": DateTime.now().millisecondsSinceEpoch,
      });
      
      // Use MLS API to create encrypted message
      final serializedMessage = await createMessageForGroup(
        groupId: _currentGroupId!,
        messageEvent: messageEvent,
      );
      
      // Save the last sent message for demonstration
      _lastSentMessage = serializedMessage;
      
      // Add to local message list
      setState(() {
        _messages.add({
          "type": "sent",
          "content": messageText,
          "timestamp": DateTime.now(),
        });
        _messageController.clear();
        _messageStatus = "Message created: ${_bytesToHex(serializedMessage, 20)}";
      });
      
      _showMessage("Message created");
    } catch (e) {
      _showMessage("Failed to create message: $e");
      setState(() {
        _messageStatus = "Error: $e";
      });
    }
  }
  
  // New: Process received message
  Future<void> _processReceivedMessage() async {
    if (_currentGroupId == null || _currentGroupId!.isEmpty) {
      _showMessage("Please create or join a group first");
      return;
    }
    
    // In a real application, this would receive messages from the network
    // For demonstration, we use the previously created message or hex input
    Uint8List serializedMessage;
    
    if (_receiveMessageController.text.trim().isNotEmpty) {
      try {
        // Try to convert hex string to byte array
        serializedMessage = _hexToBytes(_receiveMessageController.text.trim());
      } catch (e) {
        _showMessage("Invalid hex message format");
        return;
      }
    } else if (_lastSentMessage != null) {
      // Use the last sent message as an example
      serializedMessage = _lastSentMessage!;
    } else {
      _showMessage("No message to process");
      return;
    }
    
    try {
      // Process the received message
      final processedMessage = await processMessageForGroup(
        groupId: _currentGroupId!,
        serializedMessage: serializedMessage,
      );
      
      // Parse the processed message content
      final decodedMessage = utf8.decode(processedMessage);
      final messageData = jsonDecode(decodedMessage);
      
      // Add to local message list
      setState(() {
        _messages.add({
          "type": "received",
          "content": messageData['content'],
          "sender": messageData['sender'],
          "timestamp": DateTime.now(),
        });
        _receiveMessageController.clear();
        _messageStatus = "Message processed: ${messageData['content']}";
      });
      
      _showMessage("Message processed");
    } catch (e) {
      _showMessage("Failed to process message: $e");
      setState(() {
        _messageStatus = "Error: $e";
      });
    }
  }
  
  // Helper method: byte array to hex string
  String _bytesToHex(Uint8List bytes, [int? maxLength]) {
    final buffer = StringBuffer();
    final len = maxLength != null && maxLength < bytes.length ? maxLength : bytes.length;
    
    for (var i = 0; i < len; i++) {
      final byteHex = bytes[i].toRadixString(16).padLeft(2, '0');
      buffer.write(byteHex);
    }
    
    final result = buffer.toString();
    return maxLength != null && maxLength < bytes.length ? '$result...' : result;
  }
  
  // Helper method: hex string to byte array
  Uint8List _hexToBytes(String hex) {
    // Remove spaces and 0x prefix
    hex = hex.replaceAll(' ', '').replaceAll('0x', '');
    
    if (hex.length % 2 != 0) {
      hex = '0$hex'; // Ensure even length
    }
    
    final result = Uint8List(hex.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      final byteHex = hex.substring(i * 2, i * 2 + 2);
      result[i] = int.parse(byteHex, radix: 16);
    }
    
    return result;
  }
  
  void _showMessage(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Nostr MLS Demo'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('MLS Status: $_mlsStatus', style: const TextStyle(fontWeight: FontWeight.bold)),
                    const SizedBox(height: 8),
                    Text('Ciphersuite: $_ciphersuite'),
                    const SizedBox(height: 8),
                    Text('Extensions: ${_extensions.join(", ")}'),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Key Management', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _publicKeyController,
                      decoration: const InputDecoration(
                        labelText: 'Public Key',
                        hintText: 'Enter Nostr public key',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: _createKeyPackage,
                      child: const Text('Create Key Package'),
                    ),
                    if (_keyPackage.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      const Text('Generated Key Package:', style: TextStyle(fontWeight: FontWeight.bold)),
                      Container(
                        margin: const EdgeInsets.only(top: 8),
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: Colors.grey[200],
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          _keyPackage.length > 100 
                              ? '${_keyPackage.substring(0, 50)}...${_keyPackage.substring(_keyPackage.length - 50)}' 
                              : _keyPackage,
                          style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
            const SizedBox(height: 16),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Group Management', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _groupNameController,
                      decoration: const InputDecoration(
                        labelText: 'Group Name',
                        border: OutlineInputBorder(),
                      ),
                    ),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _groupDescController,
                      decoration: const InputDecoration(
                        labelText: 'Group Description',
                        border: OutlineInputBorder(),
                      ),
                      maxLines: 2,
                    ),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: _createGroup,
                      child: const Text('Create Group'),
                    ),
                    if (_groupInfo.isNotEmpty) ...[
                      const SizedBox(height: 16),
                      const Text('Group Information:', style: TextStyle(fontWeight: FontWeight.bold)),
                      Container(
                        margin: const EdgeInsets.only(top: 8),
                        padding: const EdgeInsets.all(8),
                        height: 200,
                        decoration: BoxDecoration(
                          color: Colors.grey[200],
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: SingleChildScrollView(
                          child: Text(
                            const JsonEncoder.withIndent('  ').convert(
                              jsonDecode(_groupInfo)
                            ),
                            style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
            // Add message functionality card
            if (_currentGroupId != null) ...[
              const SizedBox(height: 16),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Messaging', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                      const SizedBox(height: 16),
                      
                      // Message list
                      if (_messages.isNotEmpty) ...[
                        const Text('Message History:', style: TextStyle(fontWeight: FontWeight.bold)),
                        Container(
                          margin: const EdgeInsets.only(top: 8, bottom: 16),
                          padding: const EdgeInsets.all(8),
                          height: 150,
                          decoration: BoxDecoration(
                            color: Colors.grey[200],
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: ListView.builder(
                            itemCount: _messages.length,
                            itemBuilder: (context, index) {
                              final message = _messages[index];
                              final isReceived = message['type'] == 'received';
                              
                              return Align(
                                alignment: isReceived ? Alignment.centerLeft : Alignment.centerRight,
                                child: Container(
                                  margin: const EdgeInsets.symmetric(vertical: 4),
                                  padding: const EdgeInsets.all(8),
                                  decoration: BoxDecoration(
                                    color: isReceived ? Colors.blue[100] : Colors.green[100],
                                    borderRadius: BorderRadius.circular(8),
                                  ),
                                  child: Column(
                                    crossAxisAlignment: isReceived 
                                        ? CrossAxisAlignment.start 
                                        : CrossAxisAlignment.end,
                                    children: [
                                      Text(
                                        message['content'],
                                        style: const TextStyle(fontSize: 14),
                                      ),
                                      const SizedBox(height: 4),
                                      Text(
                                        isReceived 
                                            ? 'From: ${message['sender'] ?? 'Unknown'}'
                                            : 'Sent',
                                        style: TextStyle(
                                          fontSize: 10, 
                                          color: Colors.grey[700],
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              );
                            },
                          ),
                        ),
                      ],
                      
                      // Send message
                      const Text('Send Message:', style: TextStyle(fontWeight: FontWeight.bold)),
                      const SizedBox(height: 8),
                      TextField(
                        controller: _messageController,
                        decoration: const InputDecoration(
                          labelText: 'Message Content',
                          hintText: 'Enter message to send',
                          border: OutlineInputBorder(),
                        ),
                        maxLines: 2,
                      ),
                      const SizedBox(height: 8),
                      ElevatedButton(
                        onPressed: _createMessage,
                        child: const Text('Create Encrypted Message'),
                      ),
                      
                      const SizedBox(height: 16),
                      
                      // Receive message
                      const Text('Receive Message:', style: TextStyle(fontWeight: FontWeight.bold)),
                      const SizedBox(height: 8),
                      TextField(
                        controller: _receiveMessageController,
                        decoration: const InputDecoration(
                          labelText: 'Received Encrypted Message (Hex)',
                          hintText: 'Enter received encrypted message in hex format, or leave empty to use last created message',
                          border: OutlineInputBorder(),
                        ),
                        maxLines: 2,
                      ),
                      const SizedBox(height: 8),
                      ElevatedButton(
                        onPressed: _processReceivedMessage,
                        child: const Text('Process Received Message'),
                      ),
                      
                      if (_messageStatus.isNotEmpty) ...[
                        const SizedBox(height: 16),
                        Container(
                          padding: const EdgeInsets.all(8),
                          decoration: BoxDecoration(
                            color: Colors.grey[200],
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(_messageStatus),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
