use serde::Deserialize;

use crate::types::*;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct OrdersTop {
    pub buy: Vec<OrderWithUser>,
    pub sell: Vec<OrderWithUser>,
}
