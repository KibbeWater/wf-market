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

    pub fn to_vec(&self) -> Vec<State> {
        let mut orders = self.sell_orders.clone();
        orders.extend(self.buy_orders.clone());
        orders
    }
    pub fn total_orders(&self) -> usize {
        self.sell_orders.len() + self.buy_orders.len()
    }
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
    pub fn lowest_price(&self, order_type: OrderType) -> u32 {
        self.lowest_order(order_type)
            .map(|o| o.platinum())
            .unwrap_or(0)
    }
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
    pub fn highest_price(&self, order_type: OrderType) -> u32 {
        self.highest_order(order_type)
            .map(|o| o.platinum())
            .unwrap_or(0)
    }
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

    pub fn remove_by_id(&mut self, id: &str) {
        self.sell_orders
            .retain(|o| o.to_order().id != id);
        self.buy_orders
            .retain(|o| o.to_order().id != id);
    }

    pub fn update(&mut self, order_id: &str, args: UpdateOrderParams) {
        let mut orders = self.buy_orders
            .iter_mut()
            .chain(self.sell_orders.iter_mut());
        let index = orders.position(|o| o.to_order().id == order_id);
        if let Some(index) = index {
            if let Some(order) = orders.nth(index) {
                if let Some(platinum) = args.platinum {
                    order.to_order().platinum = platinum;
                }
                if let Some(subtype) = args.subtype {
                    order.to_order().subtype = subtype;
                }
                if let Some(order_type) = args.order_type {
                    order.to_order().order_type = order_type;
                }
            }
        }
    }

}

impl OrderList<OrderWithUser> {
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
