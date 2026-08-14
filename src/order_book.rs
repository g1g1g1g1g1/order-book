use std::collections::{BTreeMap, VecDeque};
use crate::order::{Order, Side};

#[derive(Debug)]
pub struct OrderBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bids
                    .entry(order.price)
                    .or_default()
                    .push_back(order);
            }
            Side::Sell => {
                self.asks
                    .entry(order.price)
                    .or_default()
                    .push_back(order);
            }
        }
    }
}