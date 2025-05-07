use anyhow::Result;
use lazy_static::lazy_static;
use nostr_mls::prelude::*;
use nostr_mls::NostrMls;
use nostr_mls_sqlite_storage::NostrMlsSqliteStorage;
use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

lazy_static! {
    static ref NOSTR_MLS: Mutex<Option<NostrMls<NostrMlsSqliteStorage>>> = Mutex::new(None);
}

pub fn init_nostr_mls(path: String, identity: Option<String>) {
    let mut mls = NOSTR_MLS.lock().expect("Failed to acquire NOSTR_MLS lock");

    if let Some(old_mls) = mls.take() {
        drop(old_mls);
    }
    let db_path =
        PathBuf::from(path).join(identity.as_deref().unwrap_or("default").to_owned() + "mls.db");

    let nostr_mls = NostrMls::new(NostrMlsSqliteStorage::new(db_path).unwrap());
    *mls = Some(nostr_mls);
}

pub fn get_ciphersuite() -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    format!("{:?}", nostr_mls.ciphersuite)
}

pub fn get_extensions() -> Vec<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    nostr_mls
        .extensions
        .iter()
        .map(|ext| format!("{:?}", ext))
        .collect()
}

pub fn create_key_package_for_event(public_key: String, relay: Option<Vec<String>>) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let public_key = PublicKey::from_str(&public_key).expect("Invalid public key");
    let relay = relay
        .map(|relays| {
            relays
                .into_iter()
                .map(|r| RelayUrl::from_str(&r).expect("Invalid relay url"))
                .collect::<Vec<RelayUrl>>()
        })
        .unwrap_or_default();
    let (encoded_key_package, _) = nostr_mls
        .create_key_package_for_event(&public_key, relay)
        .expect("Failed to create key package");

    encoded_key_package
}

pub fn parse_key_package(event: String) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");
    let event: Event = serde_json::from_str(&event).expect("Failed to deserialize event");
    let key_package = nostr_mls
        .parse_key_package(&event)
        .expect("Failed to parse key package");

    format!("{:?}", key_package)
}

pub fn create_group(
    group_name: String,
    group_description: String,
    group_members_key_package_events: Vec<String>,
    group_members_pubkeys: Vec<String>,
    group_creator_public_key: String,
    group_admin_public_keys: Vec<String>,
    relays: Vec<String>,
) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let member_pubkeys: Vec<PublicKey> = group_members_pubkeys
        .into_iter()
        .map(|k| PublicKey::from_str(&k).expect("Invalid member pubkey"))
        .collect();

    let mut member_key_packages = Vec::new();
    for event_str in &group_members_key_package_events {
        let event: Event = serde_json::from_str(event_str).expect("Failed to deserialize event");
        let key_package = nostr_mls
            .parse_key_package(&event)
            .expect("Failed to parse key package");
        member_key_packages.push(key_package);
    }

    let group_admin_public_keys: Vec<PublicKey> = group_admin_public_keys
        .into_iter()
        .map(|k| PublicKey::from_str(&k).expect("Invalid admin pubkey"))
        .collect();

    let group_creator_public_key =
        PublicKey::from_str(&group_creator_public_key).expect("Invalid creator pubkey");

    let relays: Vec<RelayUrl> = relays
        .into_iter()
        .map(|r| RelayUrl::from_str(&r).expect("Invalid relay url"))
        .collect();

    let group_create_result = nostr_mls.create_group(
        group_name,
        group_description,
        &group_creator_public_key,
        member_pubkeys,
        member_key_packages,
        group_admin_public_keys,
        relays,
    );

    match group_create_result {
        Ok(result) => {
            let result_debug = format!("{:?}", result);

            let alice_mls_group = result.group;
            let group_id = alice_mls_group.mls_group_id;

            let members: Vec<String> = match nostr_mls.get_members(&group_id) {
                Ok(members) => members.iter().map(|pk| pk.to_string()).collect(),
                Err(e) => {
                    eprintln!("Failed to get members: {}", e);
                    vec![]
                }
            };

            let serialized_welcome_message = result.serialized_welcome_message;
            let nostr_group_id = alice_mls_group.nostr_group_id;
            let name = alice_mls_group.name;
            let description = alice_mls_group.description;
            let admin_pubkeys = alice_mls_group.admin_pubkeys;

            let output = json!({
                "group_id": group_id,
                "members": members,
                "serialized_welcome_message": serialized_welcome_message,
                "nostr_group_data": {
                    "nostr_group_id": nostr_group_id,
                    "name": name,
                    "description": description,
                    "admin_pubkeys": admin_pubkeys,
                }
            });

            serde_json::to_string_pretty(&output).unwrap_or_else(|_| result_debug)
        }
        Err(err) => {
            let error_message = format!("{}", err);
            let output = json!({
                "error": error_message,
            });
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| error_message)
        }
    }
}

pub fn create_message_for_group(group_id: Vec<u8>, message_event: String) -> Result<Vec<u8>> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let event: Event = serde_json::from_str(&message_event).expect("Failed to deserialize event");

    let group_id = GroupId::from_slice(&group_id);

    let unsigned_event = UnsignedEvent::from(event);
    let event = nostr_mls.create_message(&group_id, unsigned_event)?;
    let serialized_message = serde_json::to_vec(&event).expect("Failed to serialize event");

    Ok(serialized_message)
}

pub fn export_secret(group_id: Vec<u8>) -> Result<(String, u64)> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");
    let group_id = GroupId::from_slice(&group_id);

    let export_secret = nostr_mls.exporter_secret(&group_id)?;
    let secret_hex = ::hex::encode(export_secret.secret);
    let epoch = export_secret.epoch;

    Ok((secret_hex, epoch))
}

pub fn process_message_for_group(message_event: String) -> Result<Vec<u8>> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let event: Event = serde_json::from_str(&message_event).expect("Failed to deserialize event");

    if let Some(processed_message) = nostr_mls.process_message(&event)? {
        let bytes = serde_json::to_vec(&processed_message).expect("Failed to serialize message");
        Ok(bytes)
    } else {
        Err(anyhow::anyhow!("No message processed"))
    }
}

pub fn join_group_from_welcome(
    wrapper_event_id: Vec<u8>,
    message_event: String,
) -> anyhow::Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let event: Event = serde_json::from_str(&message_event).expect("Failed to deserialize event");

    let rumor_event = UnsignedEvent::from(event);

    let event_id = EventId::from_slice(&wrapper_event_id)?;

    let join_result = nostr_mls.process_welcome(&event_id, &rumor_event);

    let output_str = match join_result {
        Ok(result) => {
            let result_debug = format!("{:?}", result);

            let group_id = result.mls_group_id;

            let members: Vec<String> = match nostr_mls.get_members(&group_id) {
                Ok(members) => members.iter().map(|pk| pk.to_string()).collect(),
                Err(e) => {
                    eprintln!("Failed to get members: {}", e);
                    vec![]
                }
            };

            let nostr_group_id = result.nostr_group_id;
            let name = result.group_name;
            let description = result.group_description;
            let admin_pubkeys = result.group_admin_pubkeys;

            let output = json!({
                "group_id": group_id,
                "members": members,
                "nostr_group_data": {
                    "nostr_group_id": nostr_group_id,
                    "name": name,
                    "description": description,
                    "admin_pubkeys": admin_pubkeys,
                }
            });

            serde_json::to_string_pretty(&output).unwrap_or_else(|_| result_debug)
        }
        Err(err) => {
            let error_message = format!("{}", err);
            let output = json!({
                "error": error_message,
            });
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| error_message)
        }
    };

    Ok(output_str)
}
