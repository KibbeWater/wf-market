use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{enums::*, types::*};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AuctionList<State = Auction> {
    #[serde(rename = "auctions")]
    pub auctions: Vec<State>,
    #[serde(skip)]
    _state: PhantomData<State>,
}

// Trait to abstract over Order and AuctionWithOwner
pub trait AuctionLike {
    fn platinum(&self) -> i64;
    fn owner(&self) -> Option<UserShort>;
    fn to_auction(&self) -> Auction;
    fn to_auction_mut(&mut self) -> &mut Auction;
}

// Implement trait for Auction
impl AuctionLike for Auction {
    fn platinum(&self) -> i64 {
        if self.is_direct_sell {
            self.starting_price as i64
        } else {
            self.top_bid.unwrap_or(self.starting_price) as i64
        }
    }

    fn owner(&self) -> Option<UserShort> {
        None
    }

    fn to_auction(&self) -> Auction {
        self.clone()
    }
    fn to_auction_mut(&mut self) -> &mut Auction {
        self
    }
}

// Implement trait for AuctionWithOwner
impl AuctionLike for AuctionWithOwner {
    fn platinum(&self) -> i64 {
        if self.auction.is_direct_sell {
            self.auction.starting_price as i64
        } else {
            self.auction.top_bid.unwrap_or(self.auction.starting_price) as i64
        }
    }

    fn owner(&self) -> Option<UserShort> {
        Some(self.owner.clone())
    }

    fn to_auction(&self) -> Auction {
        self.auction.clone()
    }
    fn to_auction_mut(&mut self) -> &mut Auction {
        &mut self.auction
    }
}

impl<State: AuctionLike + Clone> AuctionList<State> {
    pub fn new(auctions: Vec<State>) -> Self {
        AuctionList {
            auctions,
            _state: PhantomData,
        }
    }

    /*
    Sort the orders by platinum price.
    This method sorts the sell orders in ascending order and the buy orders in descending order.
    */
    pub fn sort_by_platinum(&mut self) {
        self.auctions
            .sort_by(|a, b| a.platinum().cmp(&b.platinum()));
    }

    /*
    Get the total number of auctions in the list.
    # Returns
     usize: The total number of auctions.
    */
    pub fn total_auctions(&self) -> usize {
        self.auctions.len()
    }

    /*
       Add a new auction to the list.
    */
    pub fn add(&mut self, auction: State) {
        self.auctions.push(auction);
    }
    /*
       Updates an existing auction or adds a new one.
       This method allows updating the details of an existing auction.
    */
    pub fn update(&mut self, auction: State) {
        if let Some(index) = self
            .auctions
            .iter()
            .position(|o| o.to_auction().id == auction.to_auction().id)
        {
            self.auctions[index] = auction;
        } else {
            self.auctions.push(auction);
        }
    }
    /*
    Remove an auction by its ID.
    # Arguments
    - id: &str: The ID of the auction to remove.
    */
    pub fn remove_by_id(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.auctions.retain(|o| o.to_auction().id != id);
    }
    /*
        Get an auction by its ID.
        # Arguments
        - id: &str: The ID of the auction to retrieve.
        # Returns
        - Option<State>: The auction with the specified ID, if it exists.
    */
    pub fn get_by_id(&self, id: impl Into<String>) -> Option<State> {
        let id = id.into();
        self.auctions
            .iter()
            .find(|auction| auction.to_auction().id == id)
            .cloned()
    }
    /*
       Get an auction by its UUID.
       # Arguments
       - uuid: &str: The UUID of the auction to retrieve.
       # Returns
       - Option<State>: The auction with the specified UUID, if it exists.
    */
    pub fn get_by_uuid(&self, uuid: impl Into<String>) -> Option<State> {
        let uuid = uuid.into();
        self.auctions
            .iter()
            .find(|auction| auction.to_auction().uuid == uuid)
            .cloned()
    }

    /*
       Filter auctions by auction type.
       This method retains auctions that match the specified type.
    */
    pub fn filter_type(&mut self, auction_type: AuctionType, exclude: bool) {
        if exclude {
            self.auctions
                .retain(|o| o.to_auction().item.item_type != auction_type);
            return;
        }
        self.auctions
            .retain(|o| o.to_auction().item.item_type == auction_type);
    }

    /*
       Get the lowest auction.
       # Returns
       - Option<&State>: The lowest auction, if it exists.
    */
    pub fn lowest_auction(&self) -> Option<State> {
        self.auctions
            .iter()
            .min_by(|a, b| a.platinum().cmp(&b.platinum()))
            .cloned()
    }
    /*
       Get the lowest price.
       # Returns
       - i64: The lowest price, if it exists.
    */
    pub fn lowest_price(&self) -> i64 {
        self.lowest_auction().map(|o| o.platinum()).unwrap_or(0)
    }
    /*
       Get the highest auction.
       # Returns
       - Option<&State>: The highest auction, if it exists.
    */
    pub fn highest_auction(&self) -> Option<State> {
        self.auctions
            .iter()
            .max_by(|a, b| a.platinum().cmp(&b.platinum()))
            .cloned()
    }
    /*
       Get the highest price.
       # Returns
       - i64: The highest price, if it exists.
    */
    pub fn highest_price(&self) -> i64 {
        self.highest_auction().map(|o| o.platinum()).unwrap_or(0)
    }
    /*
       Get all auction prices.
       # Returns
       - Vec<i64>: A vector of all auction prices.
    */
    pub fn prices(&self) -> Vec<i64> {
        self.auctions
            .iter()
            .map(|auction| auction.to_auction().platinum() as i64)
            .collect()
    }

    /*
    Filter auctions by similarity.
    # Arguments
    - similarity: i64: The similarity score to filter by.
    - attributes: Vec<ItemAttribute>: The item attributes to filter by.
    - exclude: bool: If true, excludes auctions with the specified similarity; otherwise, includes
    */
    pub fn filter_similarity(&mut self, similarity: i64, attributes: &[ItemAttribute]) {
        let auctions = self
            .auctions
            .iter_mut()
            .filter_map(|o| {
                let sim = o.to_auction_mut().item.apply_similarity(&attributes);
                if sim.score >= (similarity as f32 / 100.0) {
                    Some(o.clone())
                } else {
                    None
                }
                // o.to_auction().apply_similarity(&attributes);
            })
            .collect::<Vec<State>>();
        self.auctions = auctions;
    }

    // Sort by similarity score descending or ascending

    pub fn sort_by_similarity(&mut self, descending: bool) {
        if descending {
            self.auctions.sort_by(|a, b| {
                let sim_a = a.to_auction().item.similarity.score;
                let sim_b = b.to_auction().item.similarity.score;
                sim_a
                    .partial_cmp(&sim_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return;
        }

        self.auctions.sort_by(|a, b| {
            let sim_a = a.to_auction().item.similarity.score;
            let sim_b = b.to_auction().item.similarity.score;
            sim_b
                .partial_cmp(&sim_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /*
       Returns the top `size` results auctions
    */
    pub fn take_top(&self, size: usize) -> Vec<State> {
        self.auctions.iter().take(size).cloned().collect()
    }
    pub fn to_vec(&self) -> Vec<State> {
        self.auctions.clone()
    }
}

impl AuctionList<AuctionWithOwner> {
    /*
    Filter auctions by user status.
    # Arguments
    - status: StatusType: The status to filter by.
    - exclude: bool: If true, excludes auctions with the specified status; otherwise, includes
    */
    pub fn filter_user_status(&mut self, status: StatusType, exclude: bool) {
        if exclude {
            self.auctions
                .retain(|o| o.owner().map_or(true, |u| u.status != status));
            return;
        }
        self.auctions
            .retain(|o| o.owner().map_or(false, |u| u.status == status));
    }
    /*
    Filter auctions by username.
    # Arguments
    - name: &str: The username to filter by.
    - exclude: bool: If true, excludes auctions with the specified username; otherwise, includes
    */
    pub fn filter_username(&mut self, name: impl Into<String>, exclude: bool) {
        let name = name.into();
        if exclude {
            self.auctions
                .retain(|o| o.owner().map_or(true, |u| u.name != name));
            return;
        }
        self.auctions
            .retain(|o| o.owner().map_or(false, |u| u.name == name));
    }
}
