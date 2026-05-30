use soroban_sdk::{contracttype, Address, String};

#[derive(Clone)]
#[contracttype]
pub struct Auction {
    pub auction_id: u64,
    pub item_name: String,
    pub seller: Address,
    pub highest_bidder: Option<Address>,
    pub highest_bid: i128,
    pub end_time: u64,
    pub is_active: bool,
    pub is_finalized: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Bid {
    pub bidder: Address,
    pub amount: i128,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin, 
    Token,
    AuctionCount,
    Auction(u64),
    Bids(u64),
}