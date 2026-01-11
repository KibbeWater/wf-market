//! Data models for the warframe.market API.
//!
//! This module contains all the data structures used to represent
//! API requests and responses.

pub mod common;
pub mod credentials;
pub mod item;
pub mod item_index;
pub mod order;
pub mod request;
pub mod riven;
pub mod transaction;
pub mod user;

// V1 API models (deprecated, feature-gated)
#[cfg(feature = "v1-api")]
pub mod statistics;

// Re-export commonly used types
pub use common::{
    Activity, ActivityType, Language, OrderType, Platform, Rarity, RivenType, SubscriptionTier,
    Theme, UserRole, UserStatus,
};
pub use credentials::{Credentials, WithPassword};
pub use item::{Item, ItemSet, ItemTranslation, ModView, SculptureView};
pub use item_index::ItemIndex;
pub use order::{Order, OrderListing, OwnedOrder, OwnedOrderId, TopOrders};
pub use request::{CreateOrder, TopOrderFilters, UpdateOrder};
pub use riven::{Riven, RivenAttribute, RivenAttributeTranslation, RivenTranslation};
pub use transaction::Transaction;
pub use user::{
    Achievement, AchievementType, FullUser, UpdateProfile, User, UserPrivate, UserProfile,
};

// V1 API model re-exports (deprecated, feature-gated)
#[cfg(feature = "v1-api")]
pub use statistics::{
    ItemStatistics, ItemStatisticsView, StatisticEntry, TimeframedStatistics,
    TimeframedStatisticsView,
};
