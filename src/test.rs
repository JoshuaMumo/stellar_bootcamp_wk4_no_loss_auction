#![cfg(test)]

use crate::{auction::{NoLossAuction, NoLossAuctionClient}, storage::DataKey};
use soroban_sdk::{testutils::{Address as _, Ledger}, token, Address, Env, String};

pub struct SetupResult<'a> {
    env: Env,
    client: NoLossAuctionClient<'a>,
    admin: Address,
    seller: Address,
    bidder_a: Address,
    bidder_b: Address,
    bidder_c: Address,
    token_client: token::StellarAssetClient<'a>,
}

fn setup<'a>() -> SetupResult<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_id.address());

    let seller   = Address::generate(&env);
    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let bidder_c = Address::generate(&env);

    token_client.mint(&bidder_a, &10_000);
    token_client.mint(&bidder_b, &10_000);
    token_client.mint(&bidder_c, &10_000);

    let contract_id = env.register(NoLossAuction, (&admin, &token_id.address()));
    let client = NoLossAuctionClient::new(&env, &contract_id);

    SetupResult {
        env,
        client,
        admin,
        seller,
        bidder_a,
        bidder_b,
        bidder_c,
        token_client,
    }
}

fn advance_ledger(env: &Env, seconds: u64) {
    let current = env.ledger().timestamp();
    env.ledger().set_timestamp(current + seconds);
}

#[test]
fn test_create_auction() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");

    let auction_id = setup_result
        .client
        .create_auction(&setup_result.seller, &item, &3600u64)
        ;

    let auction = setup_result.client.get_auction(&auction_id);

    assert_eq!(auction.auction_id, 1);
    assert_eq!(auction.highest_bid, 0);
    assert_eq!(auction.is_active, true);
    assert_eq!(auction.is_finalized, false);
    assert!(auction.highest_bidder.is_none());
}

#[test]
fn test_auction_count_increments() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");

    setup_result.client.create_auction(&setup_result.seller, &item, &3600u64);
    setup_result.client.create_auction(&setup_result.seller, &item, &3600u64);
    setup_result.client.create_auction(&setup_result.seller, &item, &3600u64);

    assert_eq!(setup_result.client.get_auction_count(), 3);
}

#[test]
fn test_place_bid() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");

    let auction_id = setup_result
        .client
        .create_auction(&setup_result.seller, &item, &3600u64)
        ;

    setup_result
        .client
        .place_bids(&auction_id, &setup_result.bidder_a, &500)
        ;

    let auction = setup_result.client.get_auction(&auction_id);

    assert_eq!(auction.highest_bid, 500);
    assert_eq!(auction.highest_bidder, Some(setup_result.bidder_a.clone()));
}

#[test]
fn test_seller_cannot_bid_returns_error() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");


    let auction_id = setup_result
        .client
        .create_auction(&setup_result.seller, &item, &3600u64)
        ;

    let result = setup_result
        .client
        .try_place_bids(&auction_id, &setup_result.seller, &500);

    assert!(result.is_err());
}

#[test]
fn test_finalize_auction() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");

    
    let auction_id = setup_result
        .client
        .create_auction(&setup_result.seller, &item, &3600u64)
        ;

    let token_addr: Address = setup_result.env.storage().instance().get(&DataKey::Token).unwrap();
    let token = token::Client::new(&setup_result.env, &token_addr);


    setup_result
        .client
        .place_bids(&auction_id, &setup_result.bidder_a, &2_000)
        ;

    advance_ledger(&setup_result.env, 7200);

    setup_result.client.finalize_bid(&auction_id);

    assert_eq!(token.balance(&setup_result.seller), 2_000);


    let auction = setup_result.client.get_auction(&auction_id);
    assert_eq!(auction.is_active, false);
    assert_eq!(auction.is_finalized, true);
}

#[test]
fn test_bid_history_recorded() {
    let setup_result = setup();

    let item = String::from_str(&setup_result.env, "Milk");

    let auction_id = setup_result
        .client
        .create_auction(&setup_result.seller, &item, &3600u64)
        ;

    setup_result
        .client
        .place_bids(&auction_id, &setup_result.bidder_a, &500)
        ;
    setup_result
        .client
        .place_bids(&auction_id, &setup_result.bidder_b, &1_000)
        ;

    let history = setup_result.client.get_bid_history(&auction_id);

    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().amount, 500);
    assert_eq!(history.get(1).unwrap().amount, 1_000);
}