use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvalidAmount     = 1,
    AuctionNotFound   = 2,
    AuctionNotActive  = 3,
    AuctionStillOngoing = 4,
    AuctionEnded      = 5,
    SellerCannotBid   = 6,
    BidTooLow         = 7,
    AlreadyFinalized  = 8,  
}