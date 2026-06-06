#  No-Loss Auction 

A "No-Loss" Auction smart contract built on the Stellar network using Soroban (Rust). 

##  What does this project do?

These auctions require the winner to exchange their capital for an item, while losers get their capital back. 

This contract handles the core logic of accepting deposits, tracking the highest bidder, managing timeframes and securely refunding the principal balances once the auction concludes.

##  Design Decisions

In building this contract for the Stellar ecosystem, several key design decisions were made to ensure security, efficiency, and adherence to Soroban best practices:

* **Storage Optimization**: 
  * `Instance` storage is used for global auction state (e.g., `auction_end_time`, `highest_bidder`, `item_name`, `admin`) because these values need to be loaded together and share the contract's lifecycle.
  * `Persistent` storage is used for mapping individual users to their deposited bid amounts, ensuring user data cannot be arbitrarily archived out of state before they withdraw their funds.
* **Authentication & Security**: The contract heavily utilizes Soroban's `.require_auth()` to guarantee that only the owner of a wallet can place a bid or increase a bid 
* **Token Interface Interoperability**: The contract interacts with the standard `soroban-token-sdk`. This ensures the auction can accept any Stellar-issued asset or standard Soroban token seamlessly.


## Deployed
the project is deployed in testnet using the contract address as **[CDZLU4GYVMIDBVXAK3ZDJYZRTYJFQDWV6RCYFSWCHSVCAYAKM66Y4MQJ](https://stellar.expert/explorer/testnet/contract/CDZLU4GYVMIDBVXAK3ZDJYZRTYJFQDWV6RCYFSWCHSVCAYAKM66Y4MQJ)**