use lazy_static::lazy_static;
use std::sync::Mutex;
use nostr_openmls::NostrMls;
use std::path::PathBuf;
use anyhow::Result;
use serde_json::json;

lazy_static! {
    static ref NOSTR_MLS: Mutex<Option<NostrMls>> = Mutex::new(None);
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn init_nostr_mls(path: String, identity: Option<String>) {
    let mut mls = NOSTR_MLS.lock().expect("Failed to acquire NOSTR_MLS lock");

    if let Some(old_mls) = mls.take() {
        drop(old_mls);
    }

    let nostr_mls = NostrMls::new(PathBuf::from(path), identity);
    *mls = Some(nostr_mls);
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_ciphersuite() -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    format!("{:?}", nostr_mls.ciphersuite)
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_extensions() -> Vec<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    nostr_mls.extensions
        .iter()
        .map(|ext| format!("{:?}", ext))
        .collect()
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn create_key_package_for_event(public_key: String) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let encoded_key_package = nostr_openmls::key_packages::create_key_package_for_event(
        public_key,
        nostr_mls,
    ).expect("Failed to create key package");

    encoded_key_package
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn parse_key_package(encoded_key_package: String) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let key_package = nostr_openmls::key_packages::parse_key_package(
        encoded_key_package,
        nostr_mls,
    ).expect("Failed to parse key package");

    format!("{:?}", key_package)
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn delete_key_package_from_storage(encoded_key_package: String) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let key_package = nostr_openmls::key_packages::parse_key_package(
        encoded_key_package,
        nostr_mls,
    ).expect("Failed to parse key package");

    nostr_openmls::key_packages::delete_key_package_from_storage(
        key_package,
        nostr_mls,
    ).expect("Failed to parse key package");

    format!("Deleted!")
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn load_key_package_from_storage(encoded_key_package: String) -> String {
    // Lock the global NOSTR_MLS instance.
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    // Parse the encoded key package string into a key package object.
    let key_package = nostr_openmls::key_packages::parse_key_package(
        encoded_key_package,
        nostr_mls,
    )
    .expect("Failed to parse key package");

    // Load the key package from storage.
    let loaded_key_package = nostr_openmls::key_packages::load_key_package_from_storage(
        key_package,
        nostr_mls,
    )
    .expect("Failed to load key package from storage");

    // Serialize the loaded key package to a JSON string.
    serde_json::to_string(&loaded_key_package)
        .expect("Failed to serialize loaded key package")
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn create_group(
    group_name: String,
    group_description: String,
    group_members_key_packages: Vec<String>,
    group_creator_public_key: String,
    group_admin_public_keys: Vec<String>,
    relays: Vec<String>,
) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let member_key_packages: Vec<nostr_openmls::key_packages::KeyPackage> = group_members_key_packages
        .iter()
        .map(|key_package_str| {
            nostr_openmls::key_packages::parse_key_package(key_package_str.to_string(), nostr_mls)
                .expect("Failed to parse key package")
        })
        .collect();

    let group_create_result = nostr_mls.create_group(
        group_name,
        group_description,
        member_key_packages,
        group_admin_public_keys,
        group_creator_public_key,
        relays,
    );

    match group_create_result {
        Ok(result) => {
            let result_debug = format!("{:?}", result);

            let alice_mls_group = result.mls_group;
            let group_id = alice_mls_group.group_id();

            let members: Vec<String> = match nostr_mls.member_pubkeys(group_id.to_vec()) {
                Ok(members) => members,
                Err(e) => {
                    eprintln!("Failed to get members: {}", e);
                    vec![]
                }
            };

            let serialized_welcome_message = result.serialized_welcome_message;
            let nostr_data = result.nostr_group_data;
            let nostr_group_id = nostr_data.nostr_group_id;
            let name = nostr_data.name;
            let description = nostr_data.description;
            let admin_pubkeys = nostr_data.admin_pubkeys;
            let relays = nostr_data.relays;

            let output = json!({
                "group_id": group_id,
                "members": members,
                "serialized_welcome_message": serialized_welcome_message,
                "nostr_group_data": {
                    "nostr_group_id": nostr_group_id,
                    "name": name,
                    "description": description,
                    "admin_pubkeys": admin_pubkeys,
                    "relays": relays,
                }
            });

            serde_json::to_string_pretty(&output).unwrap_or_else(|_| result_debug)
        },
        Err(err) => {
            let error_message = format!("{}", err);
            let output = json!({
                "error": error_message,
            });
            serde_json::to_string_pretty(&output).unwrap_or_else(|_| error_message)
        }
    }
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn create_message_for_group(
    group_id: Vec<u8>,
    message_event: String
) -> Result<Vec<u8>> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let serialized_message = nostr_mls.create_message_for_group(group_id, message_event.into())?;

    Ok(serialized_message)
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn export_secret_as_hex_secret_key_and_epoch(
    group_id: Vec<u8>
) -> Result<(String, u64)> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let (export_secret_hex, epoch) = nostr_mls.export_secret_as_hex_secret_key_and_epoch(group_id)?;

    Ok((export_secret_hex, epoch))
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn process_message_for_group(
    group_id: Vec<u8>,
    serialized_message: Vec<u8>
) -> Result<Vec<u8>> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let processed_message = nostr_mls.process_message_for_group(group_id, serialized_message)?;

    Ok(processed_message)
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn preview_welcome_event(
    serialized_welcome_message: Vec<u8>
) -> anyhow::Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let welcome_preview = nostr_mls.preview_welcome_event(serialized_welcome_message.clone().into());

    let output_str = match welcome_preview {
        Ok(result) => {
            let result_debug = format!("{:?}", result);

            let nostr_data = result.nostr_group_data;
            let nostr_group_id = nostr_data.nostr_group_id;
            let name = nostr_data.name;
            let description = nostr_data.description;
            let admin_pubkeys = nostr_data.admin_pubkeys;
            let relays = nostr_data.relays;

            let output = json!({
                "nostr_group_data": {
                    "nostr_group_id": nostr_group_id,
                    "name": name,
                    "description": description,
                    "admin_pubkeys": admin_pubkeys,
                    "relays": relays,
                }
            });

            serde_json::to_string_pretty(&output).unwrap_or_else(|_| result_debug)
        },
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

#[flutter_rust_bridge::frb(dart_async)]
pub async fn join_group_from_welcome(
    serialized_welcome_message: Vec<u8>
) -> anyhow::Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let join_result = nostr_mls.join_group_from_welcome(serialized_welcome_message.clone().into());

    let output_str = match join_result {
        Ok(result) => {
            let result_debug = format!("{:?}", result);

            let alice_mls_group = result.mls_group;
            let group_id = alice_mls_group.group_id();

            let members: Vec<String> = match nostr_mls.member_pubkeys(group_id.to_vec()) {
                Ok(members) => members,
                Err(e) => {
                    eprintln!("Failed to get members: {}", e);
                    vec![]
                }
            };

            let nostr_data = result.nostr_group_data;
            let nostr_group_id = nostr_data.nostr_group_id;
            let name = nostr_data.name;
            let description = nostr_data.description;
            let admin_pubkeys = nostr_data.admin_pubkeys;
            let relays = nostr_data.relays;

            let output = json!({
                "group_id": group_id,
                "members": members,
                "serialized_welcome_message": serialized_welcome_message,
                "nostr_group_data": {
                    "nostr_group_id": nostr_group_id,
                    "name": name,
                    "description": description,
                    "admin_pubkeys": admin_pubkeys,
                    "relays": relays,
                }
            });

            serde_json::to_string_pretty(&output).unwrap_or_else(|_| result_debug)
        },
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

