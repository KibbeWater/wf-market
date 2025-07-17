use std::marker::PhantomData;

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
    fn platinum(&self) -> u32;
    fn to_order(&self) -> Order;
    fn user(&self) -> Option<UserShort>;
    fn sub_type(&self) -> &SubType;
}

// Implement trait for Order
impl OrderLike for Order {
    fn order_type(&self) -> OrderType {
        self.order_type
    }

    fn platinum(&self) -> u32 {
        self.platinum
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
}

// Implement trait for OrderWithUser
impl OrderLike for OrderWithUser {
    fn order_type(&self) -> OrderType {
        self.order.order_type
    }

    fn platinum(&self) -> u32 {
        self.order.platinum
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
    Filter orders by subtype.
    # Arguments
    - sub_type: Option<SubType>: The subtype to filter by. If None, it defaults to SubType::default().
    */
    pub fn filter_by_sub_type(&mut self, sub_type: Option<SubType>, exclude: bool) {
        let sub_type = match sub_type {
            Some(st) => st,
            None => SubType::default(), // If no subtype is provided, use default
        };

        if exclude {
            self.sell_orders.retain(|o| *o.sub_type() != sub_type);
            self.buy_orders.retain(|o| *o.sub_type() != sub_type);
            return;
        }
        self.sell_orders.retain(|o| *o.sub_type() == sub_type);

        self.buy_orders.retain(|o| *o.sub_type() == sub_type);
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
    - u32: The lowest price of the specified order type. If no orders exist, returns 0.
    */
    pub fn lowest_price(&self, order_type: OrderType) -> u32 {
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
    - u32: The highest price of the specified order type. If no orders exist, returns 0.
    */
    pub fn highest_price(&self, order_type: OrderType) -> u32 {
        self.highest_order(order_type)
            .map(|o| o.platinum())
            .unwrap_or(0)
    }
    /*
    Get the price range for a specific order type.
    # Arguments
    - order_type: OrderType: The type of order to get the price range for (Sell or Buy).
    # Returns
    - u32: The price range for the specified order type. If no orders exist, returns 0.
    */
    pub fn price_range(&self, order_type: OrderType) -> u32 {
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
    pub fn remove_by_id(&mut self, id: &str) {
        self.sell_orders.retain(|o| o.to_order().id != id);
        self.buy_orders.retain(|o| o.to_order().id != id);
    }

    /*
    Update an order by its ID.
    # Arguments
    - order_id: &str: The ID of the order to update.
    - args: UpdateOrderParams: The parameters to update the order with.
    */
    pub fn update(&mut self, order_id: &str, args: UpdateOrderParams) {
        // let mut orders = self.buy_orders
        //     .iter_mut()
        //     .chain(self.sell_orders.iter_mut());
        // let index = orders.position(|o| o.to_order().id == order_id);
        // if let Some(index) = index {
        //     if let Some(order) = orders.nth(index) {
        //         if let Some(platinum) = args.platinum {
        //             order.to_order().platinum = platinum;
        //         }
        //         if let Some(subtype) = args.subtype {
        //             order.to_order().subtype = subtype;
        //         }
        //         if let Some(order_type) = args.order_type {
        //             order.to_order().order_type = order_type;
        //         }
        //     }
        // }
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
    pub fn filter_username(&mut self, name: &str, exclude: bool) {
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
