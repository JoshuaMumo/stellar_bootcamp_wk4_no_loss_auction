use soroban_sdk::{Address, Env, String, Vec, contract, contractimpl, token};

use crate::{
    error::ContractError,
    events::{AuctionCreatedEvent, BidPlacedEvent, FinalizedAuction, OutBidEvent},
    storage::{Auction, Bid, DataKey},
};

#[contract]
pub struct NoLossAuction;

#[contractimpl]
impl NoLossAuction {

    pub fn __constructor(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::AuctionCount, &0u64);
    }

    pub fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(ContractError::AuctionNotFound)
    }

    pub fn get_auction_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AuctionCount)
            .unwrap_or(0)
    }

    pub fn get_highest_bidder(env: Env, auction_id: u64) -> i128 {
        let auction: Option<Auction> = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id));
        match auction {
            Some(a) => a.highest_bid,
            None => 0,
        }
    }

    pub fn get_bid_history(env: Env, auction_id: u64) -> Vec<Bid> {
        env.storage()
            .persistent()
            .get(&DataKey::Bids(auction_id))
            .unwrap_or(Vec::new(&env))
    }

    pub fn create_auction(
        env: Env,
        seller: Address,
        item_name: String,
        duration: u64,
    ) -> Result<u64, ContractError> {
        seller.require_auth();

        if duration == 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AuctionCount)
            .unwrap_or(0);

        count += 1;

        let end_time = env.ledger().timestamp() + duration;

        let auction = Auction {
            auction_id: count,
            item_name: item_name.clone(),
            seller: seller.clone(),
            highest_bidder: None,
            highest_bid: 0,
            end_time,
            is_active: true,
            is_finalized: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Auction(count), &auction);

        let bids: Vec<Bid> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Bids(count), &bids);

        env.storage()
            .instance()
            .set(&DataKey::AuctionCount, &count);

        AuctionCreatedEvent {
            auction_id: count,
            item_name,
            seller,
            end_time,
        }
        .publish(&env);

        Ok(count)
    }

    pub fn place_bids(
        env: Env,
        auction_id: u64,
        bidder: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        bidder.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(ContractError::AuctionNotFound)?;

        if !auction.is_active {
            return Err(ContractError::AuctionNotActive);
        }

        if env.ledger().timestamp() >= auction.end_time {
            return Err(ContractError::AuctionEnded);
        }

        if bidder == auction.seller {
            return Err(ContractError::SellerCannotBid);
        }

        if amount <= auction.highest_bid {
            return Err(ContractError::BidTooLow);
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // Refund the previous highest bidder if one exists
        if let Some(previous_bidder) = auction.highest_bidder.clone() {
            let contract_address = env.current_contract_address();
            token_client.transfer(
                &contract_address,
                &previous_bidder,
                &auction.highest_bid,
            );

            OutBidEvent {
                auction_id,
                refunded_bidder: previous_bidder,
                refunded_amount: auction.highest_bid,
            }
            .publish(&env);
        }

        token_client.transfer(
            &bidder,
            &env.current_contract_address(),
            &amount,
        );

        auction.highest_bidder = Some(bidder.clone());
        auction.highest_bid = amount;

        env.storage()
            .persistent()
            .set(&DataKey::Auction(auction_id), &auction);

        let mut bids: Vec<Bid> = env
            .storage()
            .persistent()
            .get(&DataKey::Bids(auction_id))
            .unwrap_or(Vec::new(&env));

        bids.push_back(Bid {
            bidder: bidder.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
        });

        env.storage()
            .persistent()
            .set(&DataKey::Bids(auction_id), &bids);

        BidPlacedEvent {
            auction_id,
            bidder,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    pub fn finalize_bid(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .ok_or(ContractError::AuctionNotFound)?;

        if !auction.is_active {
            return Err(ContractError::AuctionNotActive);
        }

        if env.ledger().timestamp() < auction.end_time {
            return Err(ContractError::AuctionStillOngoing);
        }

        if auction.is_finalized {
            return Err(ContractError::AlreadyFinalized);
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap();
        let token_client = token::Client::new(&env, &token_address);

        if let Some(winner) = auction.highest_bidder.clone() {
            token_client.transfer(
                &env.current_contract_address(),
                &auction.seller,
                &auction.highest_bid,
            );

            FinalizedAuction {
                auction_id,
                winner,
                winning_bid: auction.highest_bid,
            }
            .publish(&env);
        }

        auction.is_active = false;
        auction.is_finalized = true;

        env.storage()
            .persistent()
            .set(&DataKey::Auction(auction_id), &auction);

        Ok(())
    }
}