use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use nostr_mls::prelude::*;
use nostr_mls::NostrMls;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use serde_json::{json};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

lazy_static! {
    static ref NOSTR_MLS: Mutex<Option<NostrMls<NostrMlsSqliteStorage>>> = Mutex::new(None);
}

/// Initialize the NostrMls instance
/// Returns: JSON {"status": "success"} on success, or error message on failure
pub fn init_nostr_mls(path: String, identity: Option<String>) -> Result<String> {
    let mut mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;

    if let Some(old_mls) = mls.take() {
        drop(old_mls);
    }
    
    let db_path =
        PathBuf::from(path).join(identity.as_deref().unwrap_or("default").to_owned() + "-mls.db");

    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path)
        .map_err(|e| anyhow!("Failed to initialize storage: {}", e))?);
    
    *mls = Some(nostr_mls);
    
    Ok(json!({"status": "success"}).to_string())
}

/// Get the current ciphersuite
/// Returns: JSON formatted ciphersuite information
pub fn get_ciphersuite() -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let ciphersuite = format!("{:?}", nostr_mls.ciphersuite);
    Ok(json!({"ciphersuite": ciphersuite}).to_string())
}

/// Get the list of enabled extensions
/// Returns: JSON formatted list of extensions
pub fn get_extensions() -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let extensions: Vec<String> = nostr_mls
        .extensions
        .iter()
        .map(|ext| format!("{:?}", ext))
        .collect();
        
    Ok(json!({"extensions": extensions}).to_string())
}

/// Create a key package for an event
/// Returns: JSON formatted key package information, including encoded key package and tags
pub fn create_key_package_for_event(public_key: String, relay: Option<Vec<String>>) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let public_key = PublicKey::from_str(&public_key)
        .map_err(|e| anyhow!("Invalid public key: {}", e))?;
        
    let relay = relay
        .map(|relays| {
            relays
                .into_iter()
                .map(|r| RelayUrl::from_str(&r).map_err(|_| anyhow!("Invalid relay url: {}", r)))
                .collect::<Result<Vec<RelayUrl>>>()
        })
        .unwrap_or(Ok(vec![]))?;
        
    let (encoded_key_package, tags) = nostr_mls
        .create_key_package_for_event(&public_key, relay)
        .map_err(|e| anyhow!("Failed to create key package: {}", e))?;

    let tags_str: Vec<String> = tags
        .iter()
        .map(|tag| format!("{:?}", tag))
        .collect();

    Ok(json!({
        "encoded_key_package": encoded_key_package,
        "tags": tags_str
    }).to_string())
}

/// Parse a key package from serialized key package
/// Returns: JSON formatted key package information
pub fn parse_serialized_key_package(serialized_key_package: String) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;
        
    let key_package = nostr_mls
        .parse_serialized_key_package(&serialized_key_package)
        .map_err(|e| anyhow!("Failed to parse key package: {}", e))?;

    Ok(json!({
        "key_package": format!("{:?}", key_package)
    }).to_string())
}

/// Create a group
/// Returns: JSON formatted group information
pub fn create_group(
    group_name: String,
    group_description: String,
    group_members_serialized_key_packages: Vec<String>,
    group_members_pubkeys: Vec<String>,
    group_creator_public_key: String,
    group_admin_public_keys: Vec<String>,
    relays: Vec<String>,
) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let member_pubkeys: Result<Vec<PublicKey>> = group_members_pubkeys
        .into_iter()
        .map(|k| PublicKey::from_str(&k).map_err(|e| anyhow!("Invalid member pubkey: {}", e)))
        .collect();
    let member_pubkeys = member_pubkeys?;

    let mut member_key_packages = Vec::new();
    for serialized_key_package in &group_members_serialized_key_packages {
        let key_package = nostr_mls
            .parse_serialized_key_package(&serialized_key_package)
            .map_err(|e| anyhow!("Failed to parse key package: {}", e))?;
        member_key_packages.push(key_package);
    }

    let group_admin_public_keys: Result<Vec<PublicKey>> = group_admin_public_keys
        .into_iter()
        .map(|k| PublicKey::from_str(&k).map_err(|e| anyhow!("Invalid admin pubkey: {}", e)))
        .collect();
    let group_admin_public_keys = group_admin_public_keys?;

    let group_creator_public_key =
        PublicKey::from_str(&group_creator_public_key)
            .map_err(|e| anyhow!("Invalid creator pubkey: {}", e))?;

    let relays: Result<Vec<RelayUrl>> = relays
        .into_iter()
        .map(|r| RelayUrl::from_str(&r).map_err(|e| anyhow!("Invalid relay url: {}", e)))
        .collect();
    let relays = relays?;

    let group_create_result = nostr_mls.create_group(
        group_name,
        group_description,
        &group_creator_public_key,
        &member_pubkeys,
        &member_key_packages,
        group_admin_public_keys,
        relays,
    ).map_err(|e| anyhow!("Failed to create group: {}", e))?;

    let alice_mls_group = group_create_result.group;
    let group_id = alice_mls_group.mls_group_id;

    let members: Vec<String> = match nostr_mls.get_members(&group_id) {
        Ok(members) => members.iter().map(|pk| pk.to_string()).collect(),
        Err(e) => return Err(anyhow!("Failed to get members: {}", e)),
    };

    let serialized_welcome_message = group_create_result.serialized_welcome_message;
    let nostr_group_id = alice_mls_group.nostr_group_id;
    let name = alice_mls_group.name;
    let description = alice_mls_group.description;
    let admin_pubkeys = alice_mls_group.admin_pubkeys;

    let output = json!({
        "mls_group_id": group_id,
        "members": members,
        "serialized_welcome_message": serialized_welcome_message,
        "nostr_group_data": {
            "nostr_group_id": nostr_group_id,
            "name": name,
            "description": description,
            "admin_pubkeys": admin_pubkeys,
        }
    });

    Ok(output.to_string())
}

/// Create a message for a group
/// Parameters: group_id - byte array of group ID, rumor_event_string - JSON string of the event
/// Returns: JSON formatted message information
pub fn create_message_for_group(group_id: Vec<u8>, rumor_event_string: String) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let event: Event = serde_json::from_str(&rumor_event_string)
        .map_err(|e| anyhow!("Failed to deserialize event: {}", e))?;
        
    let mut unsigned_event = UnsignedEvent::from(event);
    let group_id = GroupId::from_slice(&group_id);

    let mut mls_group = nostr_mls.load_mls_group(&group_id)
        .map_err(|e| anyhow!("Failed to load MLS group: {}", e))?
        .ok_or_else(|| anyhow!("Group not found"))?;
        
    let message = nostr_mls.create_message_for_event(&mut mls_group, &mut unsigned_event)
        .map_err(|e| anyhow!("Failed to create message: {}", e))?;
        
    let message_json = serde_json::to_value(&message)
        .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;

    Ok(json!({
        "message": message_json
    }).to_string())
}

/// Export group secret
/// Parameters: group_id - byte array of group ID
/// Returns: JSON formatted secret information, including secret key and epoch
pub fn export_secret(group_id: Vec<u8>) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;
    
    let group_id = GroupId::from_slice(&group_id);

    let export_secret = nostr_mls.exporter_secret(&group_id)
        .map_err(|e| anyhow!("Failed to export secret: {}", e))?;
        
    let secret_hex = ::hex::encode(export_secret.secret);
    let epoch = export_secret.epoch;

    Ok(json!({
        "secret": secret_hex,
        "epoch": epoch
    }).to_string())
}

/// Process a message for a group
/// Parameters: group_id - byte array of group ID, serialized_message - serialized message
/// Returns: JSON formatted processing result
pub fn process_message_for_group(group_id: Vec<u8>, serialized_message: String) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;
    
    let group_id = GroupId::from_slice(&group_id);
    
    let mut mls_group = nostr_mls.load_mls_group(&group_id)
        .map_err(|e| anyhow!("Failed to load MLS group: {}", e))?
        .ok_or_else(|| anyhow!("Group not found"))?;

    let event = nostr_mls.process_message_for_group(&mut mls_group, serialized_message.as_bytes())
        .map_err(|e| anyhow!("Failed to process message: {}", e))?
        .ok_or_else(|| anyhow!("No event returned"))?;
    
    let event_json = serde_json::to_value(&event)
        .map_err(|e| anyhow!("Failed to serialize event: {}", e))?;
    
    Ok(json!({
        "event": event_json
    }).to_string())
}

/// Preview a group from a welcome message without joining it
/// Parameters: wrapper_event_id - byte array of event ID, rumor_event_string - JSON string of the event
/// Returns: JSON formatted group preview information
pub fn preview_group_from_welcome(
    wrapper_event_id: Vec<u8>,
    rumor_event_string: String,
) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let event: Event = serde_json::from_str(&rumor_event_string)
        .map_err(|e| anyhow!("Failed to deserialize event: {}", e))?;

    let rumor_event = UnsignedEvent::from(event);

    let event_id = EventId::from_slice(&wrapper_event_id)
        .map_err(|e| anyhow!("Invalid event ID: {}", e))?;

    let welcome_preview = nostr_mls.preview_welcome(&event_id, &rumor_event)
        .map_err(|e| anyhow!("Failed to process welcome: {}", e))?;

    let nostr_group_id = welcome_preview.nostr_group_data.nostr_group_id;
    let name = welcome_preview.nostr_group_data.name;
    let description = welcome_preview.nostr_group_data.description;
    let admin_pubkeys: Vec<String> = welcome_preview.nostr_group_data.admins.iter().map(|pk| pk.to_string()).collect();

    let output = json!({
        "nostr_group_data": {
            "nostr_group_id": nostr_group_id,
            "name": name,
            "description": description,
            "admin_pubkeys": admin_pubkeys,
        }
    });

    Ok(output.to_string())
}

/// Join a group from a welcome message
/// Parameters: wrapper_event_id - byte array of event ID, rumor_event_string - JSON string of the event
/// Returns: JSON formatted join result
pub fn join_group_from_welcome(
    wrapper_event_id: Vec<u8>,
    rumor_event_string: String,
) -> Result<String> {
    let mls = NOSTR_MLS.lock().map_err(|_| anyhow!("Failed to acquire NOSTR_MLS lock"))?;
    let nostr_mls = mls.as_ref().ok_or_else(|| anyhow!("NostrMls is not initialized"))?;

    let event: Event = serde_json::from_str(&rumor_event_string)
        .map_err(|e| anyhow!("Failed to deserialize event: {}", e))?;

    let rumor_event = UnsignedEvent::from(event);

    let event_id = EventId::from_slice(&wrapper_event_id)
        .map_err(|e| anyhow!("Invalid event ID: {}", e))?;

    let join_result = nostr_mls.process_welcome(&event_id, &rumor_event)
        .map_err(|e| anyhow!("Failed to process welcome: {}", e))?;

    let group_id = join_result.mls_group_id;

    let members: Vec<String> = match nostr_mls.get_members(&group_id) {
        Ok(members) => members.iter().map(|pk| pk.to_string()).collect(),
        Err(e) => return Err(anyhow!("Failed to get members: {}", e)),
    };

    let nostr_group_id = join_result.nostr_group_id;
    let name = join_result.group_name;
    let description = join_result.group_description;
    let admin_pubkeys = join_result.group_admin_pubkeys;

    let output = json!({
        "mls_group_id": group_id,
        "members": members,
        "nostr_group_data": {
            "nostr_group_id": nostr_group_id,
            "name": name,
            "description": description,
            "admin_pubkeys": admin_pubkeys,
        }
    });

    Ok(output.to_string())
}
