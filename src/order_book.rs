use std::{collections::{BTreeMap, VecDeque}, thread::current};
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

    pub fn cancel_order(&mut self, order_id: u64) -> bool {
        let mut removed = false;
        let mut empty_price = None;

        for (price, orders) in &mut self.bids {
            if let Some(index) = orders.iter().position(|order| order.id == order_id) {
                removed = true;
                orders.remove(index);

                if orders.is_empty() {
                    empty_price = Some(*price);
                }

                break;
            }
        }

        if let Some(price) = empty_price {
            self.bids.remove(&price);
        }

        if removed {
            return true;
        }

        for (price, orders) in &mut self.asks {
            if let Some(index) = orders.iter().position(|order| order.id == order_id) {
                removed = true;
                orders.remove(index);
                
                if orders.is_empty() {
                    empty_price = Some(*price);
                }

                break;
            }
        }

        if let Some(price) = empty_price {
            self.asks.remove(&price);
        }

        removed
    }

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.asks.keys().next().copied()
    }

    pub fn crosses(&self, order: &Order) -> bool {
        let current_best_bid = self.best_bid();
        let current_best_ask = self.best_ask();

        match order.side {
            Side::Buy => {
                if let Some(best_ask) = current_best_ask {
                    if order.price >= best_ask {
                        return true;
                    }
                }
            }
            Side::Sell => {
                if let Some(best_bid) = current_best_bid {
                    if order.price <= best_bid {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{Order, Side};

    #[test]
    fn empty_book_test() {
        let book = OrderBook::new();

        assert_eq!(book.best_bid(), None);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn best_bid_returns_highest_bid() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 5));
        book.add_order(Order::new(2, Side::Buy, 70, 10));
        book.add_order(Order::new(3, Side::Buy, 110, 30));

        assert_eq!(book.best_bid(), Some(110))
    }

    #[test]
    fn best_ask_returns_lowest_ask() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 5));
        book.add_order(Order::new(2, Side::Sell, 70, 10));
        book.add_order(Order::new(3, Side::Sell, 110, 30));

        assert_eq!(book.best_ask(), Some(70))
    }

    #[test]
    fn cancel_nonexistent_order() {
        let mut book = OrderBook::new();

        assert_eq!(book.cancel_order(1), false);
    }

    #[test]
    fn cancel_bid_order() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 5));
        book.add_order(Order::new(2, Side::Buy, 100, 5));

        assert!(book.cancel_order(1));

        let orders = book.bids.get(&100).unwrap();

        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].id, 2);
    }

    #[test]
    fn cancel_ask_order() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 5));
        book.add_order(Order::new(2, Side::Sell, 100, 5));

        assert!(book.cancel_order(1));

        let orders = book.asks.get(&100).unwrap();

        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].id, 2);
    }

    #[test]
    fn cancel_last_order_at_price() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 5));
        book.add_order(Order::new(2, Side::Sell, 100, 5));

        book.cancel_order(1);
        book.cancel_order(2);

        assert!(!book.bids.contains_key(&100));
        assert!(!book.asks.contains_key(&100));
    }

    #[test]
    fn crosses_empty_book() {
        let book = OrderBook::new();
        let order = Order::new(1, Side::Buy, 20, 1);

        assert!(!book.crosses(&order));
    }

    #[test]
    fn crosses_buy_below_best_ask() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Sell, 100, 1));

        let buy = Order::new(2, Side::Buy, 70, 1);

        assert!(!book.crosses(&buy));
    }

    #[test]
    fn crosses_buy_equal_to_best_ask() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Sell, 100, 1));

        let buy = Order::new(2, Side::Buy, 100, 1);

        assert!(book.crosses(&buy));
    }

    #[test]
    fn crosses_buy_above_best_ask() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Sell, 100, 1));

        let buy = Order::new(2, Side::Buy, 110, 1);

        assert!(book.crosses(&buy));
    }

    #[test]
    fn crosses_sell_above_best_bid() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Buy, 100, 1));

        let sell = Order::new(2, Side::Sell, 200, 1);

        assert!(!book.crosses(&sell));
    }

    #[test]
    fn crosses_sell_equal_to_best_bid() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Buy, 100, 1));

        let sell = Order::new(2, Side::Sell, 100, 1);

        assert!(book.crosses(&sell));
    }

    #[test]
    fn crosses_sell_below_best_bid() {
        let mut book = OrderBook::new();
        book.add_order(Order::new(1, Side::Buy, 100, 1));

        let sell = Order::new(2, Side::Sell, 50, 1);

        assert!(book.crosses(&sell));
    }
}