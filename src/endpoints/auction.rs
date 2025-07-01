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
    auctions_cache: Mutex<Option<Vec<Auction>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> AuctionRoute<State> {
    /**
     * Creates a new `AuctionRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            auctions_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }

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
                *cache = Some(auctions.clone()); // Cache the auctions
                Ok(auctions)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
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
                if let Some(ref mut auctions) = *cache {
                    auctions.push(auction.clone()); // Cache the new auction
                } else {
                    *cache = Some(vec![auction.clone()]); // Initialize cache with the new auction
                }
                Ok(auction)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
