use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde_json::{Value, json};

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

#[derive(Debug)]
pub struct AuctionRoute<State> {
    auctions_cache: Mutex<Vec<Auction>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> AuctionRoute<State> {
    /**
     * Creates a new `AuctionRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            auctions_cache: Mutex::new(Vec::new()),
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Returns the most recent auctions.
     * This method fetches the latest auctions from the server and caches them.
     * - Returns a `Result` containing a vector of `AuctionWithOwner` on success.
     */
    pub async fn get_recent_auctions(&self) -> Result<Vec<AuctionWithOwner>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(ApiVersion::V1, Method::GET, "/auctions", None, None)
            .await
        {
            Ok((data, _headers)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auctions' field in response".to_string())
                })?;
                let auctions = serde_json::from_value::<Vec<AuctionWithOwner>>(value.clone())
                    .map_err(|e| {
                        ApiError::ParsingError(format!("Failed to parse auctions data: {}", e))
                    })?;
                Ok(auctions)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Searches for auctions based on the provided filter.
     * This method allows filtering auctions by various criteria such as item type, polarity, etc.
     * - `filter`: The [`AuctionFilter`] to apply when searching for auctions.
     * - Returns a `Result` containing a vector of `AuctionWithOwner` on success.
     */
    pub async fn search_auctions(
        &self,
        filter: AuctionFilter,
    ) -> Result<Vec<AuctionWithOwner>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        let query = serde_urlencoded::to_string(filter)
            .map_err(|_| ApiError::ParsingError("Unable to serialize filters".to_string()))?;

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::GET,
                &format!("/auctions/search?{}", query),
                None,
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auctions' field in response".to_string())
                })?;
                let auctions = serde_json::from_value::<Vec<AuctionWithOwner>>(value.clone())
                    .map_err(|e| {
                        ApiError::ParsingError(format!("Failed to parse auctions data: {}", e))
                    })?;
                Ok(auctions)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Creates a new `AuctionRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &AuctionRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            auctions_cache: Mutex::new(old.auctions_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> AuctionRoute<State>
where
    State: IsAuthenticated + Clone + 'static,
{
    /**
     * Returns the cached auctions.
     * This method retrieves the cached auctions from the route.
     * - Returns a `Result` containing a vector of `Auction` on success.
     */
    pub async fn my_auctions(&self) -> Result<Vec<Auction>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        let user = client.user().get_user()?;
        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::GET,
                &format!("/profile/{}/auctions", user.ingame_name),
                None,
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auctions' field in response".to_string())
                })?;
                let auctions =
                    serde_json::from_value::<Vec<Auction>>(value.clone()).map_err(|e| {
                        ApiError::ParsingError(format!("Failed to parse auctions data: {}", e))
                    })?;
                let mut cache = self.auctions_cache.lock().unwrap();
                *cache = auctions.clone(); // Cache the auctions
                Ok(auctions)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Creates a new auction.
     * This method allows creating a new auction with the specified parameters.
     * - `args`: The [`CreateAuctionParams`] containing the details of the auction to create.
     * - Returns a `Result` containing the created `Auction` on success.
     */
    pub async fn create(&self, args: CreateAuctionParams) -> Result<Auction, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");
        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::POST,
                "/auctions/create",
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let value = data.payload.get("auction").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auction' field in response".to_string())
                })?;
                let auction = serde_json::from_value::<Auction>(value.clone()).map_err(|e| {
                    ApiError::ParsingError(format!("Failed to parse auction data: {}", e))
                })?;

                let mut cache = self.auctions_cache.lock().unwrap();
                cache.push(auction.clone());
                Ok(auction)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Updates an existing auction.
     * This method allows updating the details of an existing auction.
     * - `auction_id`: The ID of the auction to update.
     * - `args`: The [`UpdateAuctionParams`] containing the updated details of the auction.
     * - Returns a `Result` containing the updated `Auction` on success.
     */
    pub async fn update(
        &self,
        auction_id: &str,
        args: UpdateAuctionParams,
    ) -> Result<Auction, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");
        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::PUT,
                format!("/auctions/entry/{}", auction_id).as_str(),
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let value = data.payload.get("auction").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auction' field in response".to_string())
                })?;
                let auction = serde_json::from_value::<Auction>(value.clone()).map_err(|e| {
                    ApiError::ParsingError(format!("Failed to parse auction data: {}", e))
                })?;
                let mut cache = self.auctions_cache.lock().unwrap();
                if let Some(index) = cache.iter().position(|o| o.id == auction.id) {
                    cache[index] = auction.clone();
                } else {
                    cache.push(auction.clone());
                }
                return Ok(auction);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Deletes an auction by its ID.
     * This method allows closing an auction, removing it from the cache.
     * - `order_id`: The ID of the auction to close.
     * - Returns a `Result` containing the auction ID on success.
     */
    pub async fn delete(&self, order_id: &str) -> Result<String, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::PUT,
                format!("/auctions/entry/{}/close", order_id).as_str(),
                None,
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let id = data.payload.get("auction_id").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'auction_id' field in response".to_string())
                })?;
                let id_str = id.as_str().ok_or_else(|| {
                    ApiError::ParsingError("Auction ID is not a string".to_string())
                })?;
                let mut cache = self.auctions_cache.lock().unwrap();
                cache.retain(|o| o.id != id_str);
                return Ok(id_str.to_string());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
