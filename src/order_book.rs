use std::{collections::{BTreeMap, VecDeque}};
use crate::order::{Order, Side};
use crate::trade::Trade;

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

    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let trades = self.match_order(&mut order);

        if order.quantity > 0 {
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

        trades
    }

    pub fn match_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side {
            Side::Buy => {
                while self.crosses(order) {
                    let curr_best_ask = self.best_ask();
                    
                    if let Some(best_ask) = curr_best_ask {
                        let curr_orders = self.asks.get_mut(&best_ask).unwrap();
                        
                        if let Some(resting_order) = curr_orders.front_mut() {
                            let fulfilled_quantity = std::cmp::min(order.quantity, resting_order.quantity);

                            let curr_trade = Trade::new(order.id, resting_order.id, best_ask, fulfilled_quantity);
                            trades.push(curr_trade);

                            order.quantity -= fulfilled_quantity;
                            resting_order.quantity -= fulfilled_quantity;

                            if resting_order.quantity == 0 {
                                curr_orders.pop_front();

                                if curr_orders.is_empty() {
                                    self.asks.remove(&best_ask);
                                }
                            }

                            if order.quantity == 0 {
                                break;
                            }
                        }
                    }
                }
            }
            Side::Sell => {
                while self.crosses(order) {
                    let curr_best_bid = self.best_bid();

                    if let Some(best_bid) = curr_best_bid {
                        let curr_orders = self.bids.get_mut(&best_bid).unwrap();

                        if let Some(resting_order) = curr_orders.front_mut() {
                            let fulfilled_quantity = std::cmp::min(order.quantity, resting_order.quantity);

                            let curr_trade = Trade::new(resting_order.id, order.id, best_bid, fulfilled_quantity);
                            trades.push(curr_trade);

                            order.quantity -= fulfilled_quantity;
                            resting_order.quantity -= fulfilled_quantity;

                            if resting_order.quantity == 0 {
                                curr_orders.pop_front();

                                if curr_orders.is_empty() {
                                    self.bids.remove(&best_bid);
                                }
                            }

                            if order.quantity == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }

        trades
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

    #[test]
    fn sell_exact_fill() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 1));
        book.add_order(Order::new(2, Side::Sell, 100, 1));

        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn buy_exact_fill() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 1));
        book.add_order(Order::new(2, Side::Buy, 100, 1));

        assert!(book.asks.is_empty());
        assert!(book.bids.is_empty());
    }

    #[test]
    fn sell_partially_fills_resting_buy() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 5));
        book.add_order(Order::new(2, Side::Sell, 100, 1));

        assert!(book.asks.is_empty());
        assert_eq!(book.bids.get(&100).unwrap()[0].quantity, 4);
    }

    #[test]
    fn buy_partially_fills_resting_sell() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 5));
        book.add_order(Order::new(2, Side::Buy, 100, 1));

        assert!(book.bids.is_empty());
        assert_eq!(book.asks.get(&100).unwrap()[0].quantity, 4);
    }

    #[test]
    fn sell_fully_fills_resting_buy_and_rests_remainder() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 1));
        book.add_order(Order::new(2, Side::Sell, 100, 5));

        assert!(book.bids.is_empty());
        assert_eq!(book.asks.get(&100).unwrap()[0].quantity, 4);
    }

    #[test]
    fn buy_fully_fills_resting_sell_and_rests_remainder() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 1));
        book.add_order(Order::new(2, Side::Buy, 100, 5));

        assert!(book.asks.is_empty());
        assert_eq!(book.bids.get(&100).unwrap()[0].quantity, 4);
    }

    #[test]
    fn sell_matches_resting_buys_fifo() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 2));
        book.add_order(Order::new(2, Side::Buy, 100, 3));
        book.add_order(Order::new(3, Side::Sell, 100, 4));

        assert!(book.asks.is_empty());

        let remaining_order = &book.bids.get(&100).unwrap()[0];

        assert_eq!(remaining_order.id, 2);
        assert_eq!(remaining_order.quantity, 1);
    }

    #[test]
    fn buy_matches_resting_sells_fifo() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 2));
        book.add_order(Order::new(2, Side::Sell, 100, 3));
        book.add_order(Order::new(3, Side::Buy, 100, 4));

        assert!(book.bids.is_empty());

        let remaining_order = &book.asks.get(&100).unwrap()[0];

        assert_eq!(remaining_order.id, 2);
        assert_eq!(remaining_order.quantity, 1);
    }

    #[test]
    fn sell_matches_across_multiple_price_levels() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 110, 2));
        book.add_order(Order::new(2, Side::Buy, 105, 3));
        book.add_order(Order::new(3, Side::Buy, 100, 4));

        book.add_order(Order::new(4, Side::Sell, 105, 4));

        assert!(!book.bids.contains_key(&110));

        let orders_at_105 = book.bids.get(&105).unwrap();
        assert_eq!(orders_at_105[0].id, 2);
        assert_eq!(orders_at_105[0].quantity, 1);

        assert_eq!(book.bids.get(&100).unwrap()[0].quantity, 4);

        assert!(book.asks.is_empty());
    }

    #[test]
    fn buy_matches_across_multiple_price_levels() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 2));
        book.add_order(Order::new(2, Side::Sell, 105, 3));
        book.add_order(Order::new(3, Side::Sell, 110, 4));

        book.add_order(Order::new(4, Side::Buy, 105, 4));

        assert!(!book.asks.contains_key(&100));

        let orders_at_105 = book.asks.get(&105).unwrap();
        assert_eq!(orders_at_105[0].id, 2);
        assert_eq!(orders_at_105[0].quantity, 1);

        assert_eq!(book.asks.get(&110).unwrap()[0].quantity, 4);

        assert!(book.bids.is_empty());
    }

    #[test]
    fn exact_fill_returns_single_trade() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 2));

        let trades = book.add_order(Order::new(2, Side::Buy, 105, 2));

        assert_eq!(trades.len(), 1);

        let trade = &trades[0];

        assert_eq!(trade.buy_order_id, 2);
        assert_eq!(trade.sell_order_id, 1);
        assert_eq!(trade.price, 100);
        assert_eq!(trade.quantity, 2);
    }

    #[test]
    fn incoming_sell_returns_correct_trade() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Buy, 100, 3));

        let trades = book.add_order(Order::new(2, Side::Sell, 95, 2));

        assert_eq!(trades.len(), 1);

        let trade = &trades[0];

        assert_eq!(trade.buy_order_id, 1);
        assert_eq!(trade.sell_order_id, 2);
        assert_eq!(trade.price, 100);
        assert_eq!(trade.quantity, 2);
    }

    #[test]
    fn partial_fill_returns_correct_quantity() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 5));

        let trades = book.add_order(Order::new(2, Side::Buy, 100, 2));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 2);
    }

    #[test]
    fn one_order_can_generate_multiple_trades() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 2));
        book.add_order(Order::new(2, Side::Sell, 101, 3));

        let trades = book.add_order(Order::new(3, Side::Buy, 105, 4));

        assert_eq!(trades.len(), 2);

        assert_eq!(trades[0].buy_order_id, 3);
        assert_eq!(trades[0].sell_order_id, 1);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 2);

        assert_eq!(trades[1].buy_order_id, 3);
        assert_eq!(trades[1].sell_order_id, 2);
        assert_eq!(trades[1].price, 101);
        assert_eq!(trades[1].quantity, 2);
    }

    #[test]
    fn non_crossing_order_returns_no_trades() {
        let mut book = OrderBook::new();

        book.add_order(Order::new(1, Side::Sell, 100, 5));

        let trades = book.add_order(Order::new(2, Side::Buy, 90, 2));

        assert!(trades.is_empty());
    }
}