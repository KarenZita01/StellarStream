//! Tests for Stream Metadata System and Batch Withdrawal
#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

fn setup() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 100);

    let contract_id = env.register(StellarStreamContract, ());
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let client = StellarStreamContractClient::new(&env, &contract_id);

    // Set admin role
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::Role(admin.clone(), Role::SuperAdmin), &true);
    });

    // Create token
    let token_id = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = StellarAssetClient::new(&env, &token_id);
    token_client.mint(&sender, &100_000_000);

    (env, contract_id, admin, sender, receiver, token_id)
}

fn create_stream_with_receiver(
    env: &Env,
    contract: &Address,
    sender: &Address,
    receiver: &Address,
    token: &Address,
    total: i128,
) -> u64 {
    let client = StellarStreamContractClient::new(env, contract);
    client.create_stream(
        sender,
        receiver,
        token,
        &total,
        &100u64,
        &1100u64,
        &CurveType::Linear,
        &false,
    )
}

// ==================== Metadata Tests ====================

#[test]
fn test_update_stream_metadata_success() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let label = String::from_str(&env, "Salary Payment");
    let mut tags = soroban_sdk::Vec::new(&env);
    tags.push_back(String::from_str(&env, "salary"));
    tags.push_back(String::from_str(&env, "monthly"));
    let external_ref = Some(String::from_str(&env, "https://example.com/invoice/123"));

    let result = client.update_stream_metadata(
        &stream_id,
        &sender,
        &label,
        &tags,
        &external_ref,
    );

    assert!(result.is_ok());

    // Verify metadata was stored
    let stream = client.get_stream(&stream_id);
    assert!(stream.stream_metadata.is_some());
    let metadata = stream.stream_metadata.unwrap();
    assert_eq!(metadata.label, String::from_str(&env, "Salary Payment"));
    assert_eq!(metadata.tags.len(), 2);
    assert_eq!(
        metadata.external_ref,
        Some(String::from_str(&env, "https://example.com/invoice/123"))
    );
}

#[test]
fn test_update_stream_metadata_without_auth() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let unauthorized = Address::generate(&env);
    let label = String::from_str(&env, "Unauthorized");
    let tags = soroban_sdk::Vec::new(&env);

    let result = client.try_update_stream_metadata(
        &stream_id,
        &unauthorized,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_wrong_sender() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let wrong_sender = Address::generate(&env);
    let label = String::from_str(&env, "Wrong Sender");
    let tags = soroban_sdk::Vec::new(&env);

    let result = client.try_update_stream_metadata(
        &stream_id,
        &wrong_sender,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_label_too_long() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    // Create a label longer than 64 characters
    let long_label = String::from_str(
        &env,
        "This is a very long label that exceeds the sixty-four character limit for stream labels",
    );
    let tags = soroban_sdk::Vec::new(&env);

    let result = client.try_update_stream_metadata(
        &stream_id,
        &sender,
        &long_label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_too_many_tags() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let label = String::from_str(&env, "Test");
    let mut tags = soroban_sdk::Vec::new(&env);
    // Add 6 tags (exceeds max of 5)
    tags.push_back(String::from_str(&env, "tag1"));
    tags.push_back(String::from_str(&env, "tag2"));
    tags.push_back(String::from_str(&env, "tag3"));
    tags.push_back(String::from_str(&env, "tag4"));
    tags.push_back(String::from_str(&env, "tag5"));
    tags.push_back(String::from_str(&env, "tag6"));

    let result = client.try_update_stream_metadata(
        &stream_id,
        &sender,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_tag_too_long() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let label = String::from_str(&env, "Test");
    let mut tags = soroban_sdk::Vec::new(&env);
    // Tag longer than 32 characters
    tags.push_back(String::from_str(
        &env,
        "this_tag_is_way_too_long_for_the_limit",
    ));

    let result = client.try_update_stream_metadata(
        &stream_id,
        &sender,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_on_closed_stream() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    // Close the stream first
    client.cancel_stream(&stream_id, &sender);

    let label = String::from_str(&env, "Should Fail");
    let tags = soroban_sdk::Vec::new(&env);

    let result = client.try_update_stream_metadata(
        &stream_id,
        &sender,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_err());
}

#[test]
fn test_update_stream_metadata_empty() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let label = String::from_str(&env, "");
    let tags = soroban_sdk::Vec::new(&env);

    let result = client.update_stream_metadata(
        &stream_id,
        &sender,
        &label,
        &tags,
        &None,
    );

    assert!(result.is_ok());

    // Verify metadata was stored
    let stream = client.get_stream(&stream_id);
    assert!(stream.stream_metadata.is_some());
    let metadata = stream.stream_metadata.unwrap();
    assert_eq!(metadata.label, String::from_str(&env, ""));
    assert_eq!(metadata.tags.len(), 0);
    assert_eq!(metadata.external_ref, None);
}

// ==================== Batch Withdrawal Tests ====================

#[test]
fn test_batch_withdraw_two_streams() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    // Create two streams
    let stream1 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);
    let stream2 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 2000);

    // Advance time to allow some vesting
    env.ledger().with_mut(|li| li.timestamp = 600);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(stream1);
    stream_ids.push_back(stream2);

    let result = client.batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_ok());

    let amounts = result.unwrap();
    assert_eq!(amounts.len(), 2);
    // Both streams should have some withdrawable amount
    assert!(amounts.get(0).unwrap() > 0);
    assert!(amounts.get(1).unwrap() > 0);
}

#[test]
fn test_batch_withdraw_exceeds_max_size() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    // Create 21 streams
    let mut stream_ids = soroban_sdk::Vec::new(&env);
    for _ in 0..21 {
        let id = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);
        stream_ids.push_back(id);
    }

    let result = client.try_batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_err());
}

#[test]
fn test_batch_withdraw_empty_batch() {
    let (env, contract, _admin, _sender, receiver, _token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream_ids = soroban_sdk::Vec::new(&env);

    let result = client.try_batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_err());
}

#[test]
fn test_batch_withdraw_unauthorized() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream1 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(stream1);

    let unauthorized = Address::generate(&env);

    let result = client.try_batch_withdraw(&stream_ids, &unauthorized);
    assert!(result.is_err());
}

#[test]
fn test_batch_withdraw_partial_failure_reverts() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let stream1 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(stream1);
    // Add non-existent stream ID
    stream_ids.push_back(999);

    let result = client.try_batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_err());
}

#[test]
fn test_batch_withdraw_stream_not_found() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(999); // Non-existent stream

    let result = client.try_batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_err());
}

#[test]
fn test_batch_withdraw_mixed_amounts() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    // Create streams with different amounts
    let stream1 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);
    let stream2 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 5000);
    let stream3 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 10000);

    // Advance time
    env.ledger().with_mut(|li| li.timestamp = 600);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(stream1);
    stream_ids.push_back(stream2);
    stream_ids.push_back(stream3);

    let result = client.batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_ok());

    let amounts = result.unwrap();
    assert_eq!(amounts.len(), 3);
    // Verify proportional amounts (linear vesting, 50% elapsed)
    assert!(amounts.get(0).unwrap() > 0);
    assert!(amounts.get(1).unwrap() > amounts.get(0).unwrap());
    assert!(amounts.get(2).unwrap() > amounts.get(1).unwrap());
}

#[test]
fn test_batch_withdraw_all_or_nothing() {
    let (env, contract, _admin, sender, receiver, token) = setup();
    let client = StellarStreamContractClient::new(&env, &contract);

    // Create one valid stream and one paused stream
    let stream1 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 1000);
    let stream2 = create_stream_with_receiver(&env, &contract, &sender, &receiver, &token, 2000);

    // Pause the second stream
    client.pause_stream(&stream2, &sender);

    // Advance time
    env.ledger().with_mut(|li| li.timestamp = 600);

    let mut stream_ids = soroban_sdk::Vec::new(&env);
    stream_ids.push_back(stream1);
    stream_ids.push_back(stream2);

    // Should fail because one stream is paused
    let result = client.try_batch_withdraw(&stream_ids, &receiver);
    assert!(result.is_err());
}
