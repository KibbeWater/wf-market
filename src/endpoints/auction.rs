use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde::de::Error;
use serde_json::{Value, json};

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

#[derive(Debug)]
pub struct AuctionRoute<State> {
    auctions_cache: Mutex<AuctionList<Auction>>,
    limit: Mutex<usize>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> AuctionRoute<State> {
    /**
     * Creates a new `AuctionRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            auctions_cache: Mutex::new(AuctionList::new(vec![])),
            limit: Mutex::new(50), // Default limit
            client: Arc::downgrade(&client),
        })
    }
    /**
    Get the current auctions from the cache.
    # Returns
    - `AuctionList<Auction>`: A clone of the cached auctions.
    */
    pub fn cache_auctions(&self) -> AuctionList<Auction> {
        let ca_auctions = self.auctions_cache.lock().unwrap();
        ca_auctions.clone()
    }

    /**
    Get a mutable reference to the current auctions from the cache.
    # Returns
    - `std::sync::MutexGuard<AuctionList<Auction>>`: A mutable reference to the cached auctions.
    */
    pub fn cache_auctions_mut(&'_ self) -> std::sync::MutexGuard<'_, AuctionList<Auction>> {
        self.auctions_cache.lock().unwrap()
    }

    /**
     * Returns the most recent auctions.
     * This method fetches the latest auctions from the server and caches them.
     * - Returns a `Result` containing a vector of `AuctionWithOwner` on success.
     */
    pub async fn get_recent_auctions(&self) -> Result<AuctionList<AuctionWithOwner>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(ApiVersion::V1, Method::GET, "/auctions", None, None)
            .await
        {
            Ok((data, _, err)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::missing_field("auctions"),
                    )
                })?;
                let mut auctions = serde_json::from_value::<Vec<AuctionWithOwner>>(value.clone())
                    .map_err(|e| ApiError::ParsingError(err, e))?;
                for auction in &mut auctions {
                    auction.apply_uuid();
                }
                Ok(AuctionList::new(auctions))
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
    ) -> Result<AuctionList<AuctionWithOwner>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        let query = serde_urlencoded::to_string(&filter).map_err(|e| {
            ApiError::Unknown(format!("Failed to serialize auctions filter: {}", e))
        })?;

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
            Ok((data, _, err)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::missing_field("auctions"),
                    )
                })?;
                let auctions = serde_json::from_value::<Vec<AuctionWithOwner>>(value.clone())
                    .map_err(|e| ApiError::ParsingError(err, e))?;
                let mut list = AuctionList::new(auctions);

                if filter.user_activity.as_ref().is_some() {
                    list.filter_user_status(filter.user_activity.unwrap(), false);
                }
                if filter.similarity.as_ref().is_some()
                    && filter.similarity_attributes.as_ref().is_some()
                {
                    list.filter_similarity(
                        filter.similarity.unwrap(),
                        filter.similarity_attributes.clone().unwrap(),
                    );
                }

                Ok(list)
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
            limit: Mutex::new(old.limit.lock().unwrap().clone()),
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
     * - Returns `Result<AuctionList<Auction>, ApiError>`
     */
    pub async fn my_auctions(&self) -> Result<AuctionList<Auction>, ApiError> {
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
            Ok((data, _, err)) => {
                let value = data.payload.get("auctions").ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::missing_field("auctions"),
                    )
                })?;
                let mut auctions = serde_json::from_value::<Vec<Auction>>(value.clone())
                    .map_err(|e| ApiError::ParsingError(err, e))?;
                for auction in &mut auctions {
                    auction.apply_uuid();
                }
                let auctions = AuctionList::new(auctions.clone());
                let mut ca_auctions = self.auctions_cache.lock().unwrap();
                *ca_auctions = auctions.clone();
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
        if !self.can_create_auction() {
            return Err(ApiError::AuctionLimitExceeded(RequestError::new(
                ApiVersion::V1,
                "POST".to_string(),
                "/auctions/create".to_string(),
                Some(json!(args)),
            )));
        }
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
            Ok((data, _, err)) => {
                let value = data.payload.get("auction").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("auction"))
                })?;
                let mut auction = serde_json::from_value::<Auction>(value.clone())
                    .map_err(|e| ApiError::ParsingError(err, e))?;
                auction.properties = args.properties; // Set properties if any
                auction.apply_uuid();

                let mut cache = self.auctions_cache.lock().unwrap();
                cache.add(auction.clone());
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
            Ok((data, _, err)) => {
                let value = data.payload.get("auction").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("auction"))
                })?;
                let mut auction = serde_json::from_value::<Auction>(value.clone())
                    .map_err(|e| ApiError::ParsingError(err, e))?;
                if let Some(properties) = args.properties.clone() {
                    auction.properties = Some(properties);
                }
                auction.apply_uuid();
                let mut cache = self.auctions_cache.lock().unwrap();
                cache.update(auction.clone());
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
            Ok((data, _, err)) => {
                let id = data.payload.get("auction_id").ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::missing_field("auction_id"),
                    )
                })?;
                let id_str = id.as_str().ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::custom("Auction ID is not a string"),
                    )
                })?;
                let mut cache = self.auctions_cache.lock().unwrap();
                cache.remove_by_id(id_str);
                return Ok(id_str.to_string());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Set the limit for active auctions
     * # Arguments
     * - `limit`: The new limit for active auctions
     * # Returns
     * - `Ok(())` if the limit was successfully set
     * - `Err(ApiError)` if there was an error setting the limit
     */
    pub fn set_auction_limit(&self, limit: usize) {
        let mut ca_auctions = self.limit.lock().unwrap();
        *ca_auctions = limit;
    }

    /**
     * Get the current limit for active auctions
     * # Returns
     * - `Ok(usize)` if the limit was successfully retrieved
     * - `Err(ApiError)` if there was an error retrieving the limit
     */
    pub fn get_auction_limit(&self) -> usize {
        let ca_auctions = self.limit.lock().unwrap();
        *ca_auctions
    }
    /**
     * Check if a new auction can be created
     * # Returns
     * - `true` if a new auction can be created
     * - `false` if the auction limit has been reached
     */
    pub fn can_create_auction(&self) -> bool {
        let ca_auctions = self.auctions_cache.lock().unwrap();
        let auction_limit = self.get_auction_limit();
        ca_auctions.total_auctions() < auction_limit
    }
}
