use soroban_sdk::{contractevent, Address, String};

#[derive(Clone)]
#[contractevent]
pub struct AuctionCreatedEvent {
    pub auction_id: u64,
    pub item_name: String,
    pub seller: Address,
    pub end_time: u64,
}

#[derive(Clone)]
#[contractevent]
pub struct BidPlacedEvent {
    pub auction_id: u64,
    pub bidder: Address,
    pub amount: i128,
}

#[derive(Clone)]
#[contractevent]
pub struct OutBidEvent {
    pub auction_id:u64,
    pub refunded_bidder: Address,
    pub refunded_amount: i128,
}


#[derive(Clone)]
#[contractevent]
pub struct FinalizedAuction {
    pub auction_id: u64,
    pub winner: Address,
    pub winning_bid: i128,
}
