use std::{fmt::Display, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{enums::*, types::*};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OrderList<State = Order> {
    #[serde(rename = "sell_orders")]
    pub sell_orders: Vec<State>,
    #[serde(rename = "buy_orders")]
    pub buy_orders: Vec<State>,
    #[serde(skip)]
    _state: PhantomData<State>,
}

// Trait to abstract over Order and OrderWithUser
pub trait OrderLike {
    fn order_type(&self) -> OrderType;
    fn platinum(&self) -> i64;
    fn to_order(&self) -> Order;
    fn user(&self) -> Option<UserShort>;
    fn sub_type(&self) -> &SubType;
    fn update(&mut self, args: UpdateOrderParams);
}

// Implement trait for Order
impl OrderLike for Order {
    fn order_type(&self) -> OrderType {
        self.order_type
    }

    fn platinum(&self) -> i64 {
        let per_trade = self.per_trade.unwrap_or(1);
        ((self.platinum as i64) / per_trade as i64).into()
    }

    fn to_order(&self) -> Order {
        self.clone()
    }

    fn user(&self) -> Option<UserShort> {
        None // Orders do not have a user field
    }

    fn sub_type(&self) -> &SubType {
        &self.subtype
    }
    fn update(&mut self, args: UpdateOrderParams) {
        if let Some(platinum) = args.platinum {
            self.platinum = platinum;
        }
        if let Some(quantity) = args.quantity {
            self.quantity = quantity;
        }

        if let Some(per_trade) = args.per_trade {
            self.per_trade = Some(per_trade as u8);
        }
        if let Some(rank) = args.rank {
            self.subtype.rank = Some(rank as i64);
        }
        if let Some(visible) = args.visible {
            self.visible = visible;
        }
        if let Some(properties) = args.properties {
            self.properties = Properties::from(properties);
        }
    }
}

// Implement trait for OrderWithUser
impl OrderLike for OrderWithUser {
    fn order_type(&self) -> OrderType {
        self.order.order_type
    }

    fn platinum(&self) -> i64 {
        let per_trade = self.order.per_trade.unwrap_or(1);
        ((self.order.platinum as i64) / per_trade as i64).into()
    }

    fn to_order(&self) -> Order {
        self.downgrade()
    }
    fn user(&self) -> Option<UserShort> {
        Some(self.user.clone()) // OrdersWithUser have a user field
    }
    fn sub_type(&self) -> &SubType {
        &self.order.subtype
    }
    fn update(&mut self, args: UpdateOrderParams) {
        self.order.update(args);
    }
}

impl<State: OrderLike + Clone> OrderList<State> {
    pub fn new(orders: Vec<State>) -> Self {
        let mut buy_orders: Vec<State> = orders
            .iter()
            .filter(|o| o.order_type() == OrderType::Buy)
            .cloned()
            .collect();
        buy_orders.sort_by(|a, b| b.platinum().cmp(&a.platinum()));

        let mut sell_orders: Vec<State> = orders
            .iter()
            .filter(|o| o.order_type() == OrderType::Sell)
            .cloned()
            .collect();
        sell_orders.sort_by(|a, b| a.platinum().cmp(&b.platinum()));
        OrderList {
            sell_orders,
            buy_orders,
            _state: PhantomData,
        }
    }

    /*
    Sort the orders by platinum price.
    This method sorts the sell orders in ascending order and the buy orders in descending order.
    */
    pub fn sort_by_platinum(&mut self) {
        self.sell_orders
            .sort_by(|a, b| a.platinum().cmp(&b.platinum()));
        self.buy_orders
            .sort_by(|a, b| b.platinum().cmp(&a.platinum()));
    }

    /*
    Get a vector of all orders in the list.
    # Returns
     Vec<State>: A vector containing all orders, both sell and buy.
    */
    pub fn to_vec(&self) -> Vec<State> {
        let mut orders = self.sell_orders.clone();
        orders.extend(self.buy_orders.clone());
        orders
    }
    /*
    Get the total number of orders in the list.
    # Returns
     usize: The total number of orders, both sell and buy.
    */
    pub fn total_orders(&self) -> usize {
        self.sell_orders.len() + self.buy_orders.len()
    }

    /*
    Find an order by its ID and subtype.
    # Arguments
    - id: impl Into<String>: The item ID to search for.
    - sub_type: Option<SubType>: The subtype of the order to find. If None, defaults to SubType::default().
    - order_type: OrderType: The type of order to find (Sell or Buy).
    */
    pub fn find_order(
        &self,
        wfm_id: impl Into<String>,
        sub_type: &SubType,
        order_type: OrderType,
    ) -> Option<State> {
        let id = wfm_id.into();
        match order_type {
            OrderType::Sell => self
                .sell_orders
                .iter()
                .find(|o| o.to_order().item_id == id && o.to_order().sub_type() == sub_type)
                .cloned(),
            OrderType::Buy => self
                .buy_orders
                .iter()
                .find(|o| o.to_order().item_id == id && o.to_order().sub_type() == sub_type)
                .cloned(),
        }
    }

    /*
    Filter orders by subtype.
    # Arguments
    - sub_type: Option<SubType>: The subtype to filter by. If None, it defaults to SubType::default().
    */
    pub fn filter_by_sub_type(&mut self, sub_type: SubType, exclude: bool) {
        if exclude {
            self.sell_orders.retain(|o| *o.sub_type() != sub_type);
            self.buy_orders.retain(|o| *o.sub_type() != sub_type);
            return;
        }
        self.sell_orders.retain(|o| *o.sub_type() == sub_type);

        self.buy_orders.retain(|o| *o.sub_type() == sub_type);
    }
    /*
    Filter orders based on a provided filter function.
    # Arguments
    - filter: F: A closure that takes a reference to a State and returns a boolean
    indicating whether the order should be retained (true) or removed (false).
    */
    pub fn filter<F>(&mut self, filter: F)
    where
        F: Fn(&State) -> bool,
    {
        self.sell_orders.retain(|order| filter(order));
        self.buy_orders.retain(|order| filter(order));
    }
    /*
    Get the lowest order of a specific type.
    # Arguments
    - order_type: OrderType: The type of order to get (Sell or Buy).
    # Returns
    - Option<State>: An optional State representing the lowest order of the specified type. If no orders exist, returns None.
    */
    pub fn lowest_order(&self, order_type: OrderType) -> Option<State> {
        let orders = match order_type {
            OrderType::Sell => &self.sell_orders,
            OrderType::Buy => &self.buy_orders,
        };

        if orders.is_empty() {
            return None;
        }
        orders
            .iter()
            .min_by(|a, b| a.platinum().cmp(&b.platinum()))
            .cloned()
    }
    /*
    Get the lowest price of a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get the lowest price for (Sell or Buy).
    # Returns
    - i64: The lowest price of the specified order type. If no orders exist, returns 0.
    */
    pub fn lowest_price(&self, order_type: OrderType) -> i64 {
        self.lowest_order(order_type)
            .map(|o| o.platinum())
            .unwrap_or(0)
    }
    /*
    Get the highest order of a specific type.
    # Arguments
    - order_type: OrderType: The type of order to get (Sell or Buy).
    # Returns
    - Option<State>: An optional State representing the highest order of the specified type. If no orders exist, returns None.
    */
    pub fn highest_order(&self, order_type: OrderType) -> Option<State> {
        let orders = match order_type {
            OrderType::Sell => &self.sell_orders,
            OrderType::Buy => &self.buy_orders,
        };

        if orders.is_empty() {
            return None;
        }
        orders
            .iter()
            .max_by(|a, b| a.platinum().cmp(&b.platinum()))
            .cloned()
    }
    /*
    Get the highest price of a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get the highest price for (Sell or Buy).
    # Returns
    - i64: The highest price of the specified order type. If no orders exist, returns 0.
    */
    pub fn highest_price(&self, order_type: OrderType) -> i64 {
        self.highest_order(order_type)
            .map(|o| o.platinum())
            .unwrap_or(0)
    }
    /*
    Get the price range for a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get the price range for (Sell or Buy).
    # Returns
    - i64: The price range for the specified order type. If no orders exist, returns 0.
    */
    pub fn price_range(&self, order_type: OrderType) -> i64 {
        let lowest_price = self.lowest_price(OrderType::Sell);
        let highest_price = self.highest_price(OrderType::Buy);
        if order_type == OrderType::Sell {
            return highest_price - lowest_price;
        } else if order_type == OrderType::Buy {
            return lowest_price - highest_price;
        }
        return 0;
    }
    /*
    Add an order to the list.
    # Arguments
    - order: State: The order to add to the list. It can be either a sell or buy order.
    */
    pub fn add(&mut self, order: State) {
        match order.order_type() {
            OrderType::Sell => {
                self.sell_orders.push(order);
            }
            OrderType::Buy => {
                self.buy_orders.push(order);
            }
        }
    }
    /*
    Remove an order by its ID.
    # Arguments
    - id: &str: The ID of the order to remove.
    */
    pub fn remove_by_id(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.sell_orders.retain(|o| o.to_order().id != id);
        self.buy_orders.retain(|o| o.to_order().id != id);
    }

    /*
    Update an order by its ID.
    # Arguments
    - order_id: &str: The ID of the order to update.
    - args: UpdateOrderParams: The parameters to update the order with.
    */
    pub fn update(&mut self, order_id: impl Into<String>, args: UpdateOrderParams) {
        let order_id = order_id.into();

        // First check buy orders
        for order in &mut self.buy_orders {
            if order.to_order().id == order_id {
                order.update(args);
                return;
            }
        }

        // Then check sell orders
        for order in &mut self.sell_orders {
            if order.to_order().id == order_id {
                order.update(args);
                return;
            }
        }

        println!("Order with ID {} not found", order_id);
    }
    /*
    Close an order by its ID.
    # Arguments
    - order_id: &str: The ID of the order to close.
    - quantity: u32: The quantity to close. If 0, closes the entire order.
     */
    pub fn close_order(&mut self, order_id: impl Into<String>, quantity: u32) {
        let order_id = order_id.into();
        let orders = self
            .buy_orders
            .iter_mut()
            .chain(self.sell_orders.iter_mut())
            .collect::<Vec<_>>();
        // First check buy orders
        for order in orders {
            if order.to_order().id == order_id {
                let new_quantity = order.to_order().quantity - quantity;
                if new_quantity <= 0 {
                    self.remove_by_id(order_id);
                } else {
                    order.update(UpdateOrderParams::new().with_quantity(new_quantity));
                }
                break;
            }
        }
    }

    /*
    Get a list of order IDs for a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get IDs for (Sell or Buy).
    # Returns
    - Vec<String>: A vector of order IDs for the specified order type.
     */
    pub fn order_ids(&self, order_type: OrderType) -> Vec<String> {
        match order_type {
            OrderType::Sell => self
                .sell_orders
                .iter()
                .map(|o| o.to_order().id.clone())
                .collect(),
            OrderType::Buy => self
                .buy_orders
                .iter()
                .map(|o| o.to_order().id.clone())
                .collect(),
        }
    }

    /*
       Returns the top `size` results orders
    */
    pub fn take_top(&self, size: usize, order_type: OrderType) -> Vec<State> {
        match order_type {
            OrderType::Sell => self.sell_orders.iter().take(size).cloned().collect(),
            OrderType::Buy => self.buy_orders.iter().take(size).cloned().collect(),
        }
    }
    /*
       Get Order by ID
       # Arguments
       - order_id: &str: The ID of the order to get.
       # Returns
       - Option<Order>: The order with the specified ID, if it exists.
    */
    pub fn get_by_id(&self, order_id: impl Into<String>) -> Option<Order> {
        let order_id = order_id.into();
        self.sell_orders
            .iter()
            .chain(self.buy_orders.iter())
            .find(|o| o.to_order().id == order_id)
            .map(|o| o.to_order().clone())
    }
    /*
    Get a list of prices for a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get prices for (Sell or Buy).
    # Returns
    - Vec<i64>: A vector of prices for the specified order type.
     */
    pub fn get_price_list(
        &self,
        order_type: OrderType,
        filter: Option<fn(&State) -> bool>,
    ) -> Vec<i64> {
        let orders = match order_type {
            OrderType::Sell => &self.sell_orders,
            OrderType::Buy => &self.buy_orders,
        };
        let filtered_orders = if let Some(filter_fn) = filter {
            orders.iter().filter(|o| filter_fn(o)).collect::<Vec<_>>()
        } else {
            orders.iter().collect::<Vec<_>>()
        };
        filtered_orders.iter().map(|o| o.platinum()).collect()
    }
}

impl OrderList<OrderWithUser> {
    /*
    Filter orders by user status.
    # Arguments
    - status: StatusType: The status to filter by.
    - exclude: bool: If true, excludes orders with the specified status; otherwise, includes
    */
    pub fn filter_user_status(&mut self, status: StatusType, exclude: bool) {
        if exclude {
            self.sell_orders
                .retain(|o| o.user().map_or(true, |u| u.status != status));
            self.buy_orders
                .retain(|o| o.user().map_or(true, |u| u.status != status));
            return;
        }
        self.sell_orders
            .retain(|o| o.user().map_or(false, |u| u.status == status));
        self.buy_orders
            .retain(|o| o.user().map_or(false, |u| u.status == status));
    }
    /*
    Filter orders by username.
    # Arguments
    - name: &str: The username to filter by.
    - exclude: bool: If true, excludes orders with the specified username; otherwise, includes
    */
    pub fn filter_username(&mut self, name: impl Into<String>, exclude: bool) {
        let name = name.into();
        if exclude {
            self.sell_orders
                .retain(|o| o.user().map_or(true, |u| u.name != name));
            self.buy_orders
                .retain(|o| o.user().map_or(true, |u| u.name != name));
            return;
        }
        self.sell_orders
            .retain(|o| o.user().map_or(false, |u| u.name == name));
        self.buy_orders
            .retain(|o| o.user().map_or(false, |u| u.name == name));
    }
}

impl Display for OrderList<Order> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!(
            "Sell Orders: {}, Buy Orders: {}",
            self.sell_orders.len(),
            self.buy_orders.len()
        ));
        write!(f, "{}", output)
    }
}
impl Display for OrderList<OrderWithUser> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        output.push_str(&format!(
            "Sell Orders: {}, Buy Orders: {}",
            self.sell_orders.len(),
            self.buy_orders.len()
        ));
        write!(f, "{}", output)
    }
}
