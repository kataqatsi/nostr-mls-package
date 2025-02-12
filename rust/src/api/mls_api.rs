use lazy_static::lazy_static;
use std::sync::Mutex;
use nostr_openmls::NostrMls;
use std::path::PathBuf;
use anyhow::Result;

lazy_static! {
    static ref NOSTR_MLS: Mutex<Option<NostrMls>> = Mutex::new(None);
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn init_nostr_mls(path: String, identity: Option<String>) {
    let nostr_mls = NostrMls::new(PathBuf::from(path), identity);
    let mut mls = NOSTR_MLS.lock().unwrap();
    *mls = Some(nostr_mls);
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
pub async fn create_group(
    group_name: String,
    group_description: String,
    bob_key_package: String,
    alice_public_key: String,
    bob_public_key: String,
    group_admin_public_keys: Vec<String>,
    relays: Vec<String>,
) -> String {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let bob_key_package_parsed = nostr_openmls::key_packages::parse_key_package(
        bob_key_package,
        nostr_mls,
    ).expect("Failed to parse Bob's key package");

    let group_create_result = nostr_mls.create_group(
        group_name,
        group_description,
        vec![bob_key_package_parsed],
        group_admin_public_keys,
        alice_public_key,
        relays,
    ).expect("Failed to create group");

    format!("Group created: {:?}", group_create_result)
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
pub async fn preview_welcome_event(
    serialized_welcome_message: String
) -> Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let welcome_preview = nostr_mls.preview_welcome_event(serialized_welcome_message.into())?;

    Ok(format!("{:?}", welcome_preview))
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn join_group_from_welcome(
    serialized_welcome_message: String
) -> Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let join_result = nostr_mls.join_group_from_welcome(serialized_welcome_message.into())?;

    Ok(format!("{:?}", join_result))
}

#[flutter_rust_bridge::frb(dart_async)]
pub async fn process_message_for_group(
    group_id: Vec<u8>,
    serialized_message: String
) -> Result<String> {
    let mls = NOSTR_MLS.lock().unwrap();
    let nostr_mls = mls.as_ref().expect("NostrMls is not initialized");

    let processed_message = nostr_mls.process_message_for_group(group_id, serialized_message.into())?;

    Ok(format!("{:?}", processed_message))
}

