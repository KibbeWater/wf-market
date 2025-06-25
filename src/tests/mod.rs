use crate::{
    client::{Authenticated, Client},
    enums::*,
    errors::AuthError,
    types::{CreateOrderParams, UpdateOrderParams},
};
use dotenv::dotenv;
use std::env;

mod order;
mod user;
