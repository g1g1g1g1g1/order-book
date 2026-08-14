mod order;
mod order_book;

use order::{Order, Side};
use order_book::{OrderBook};

fn main() {
    let buy = Order::new(1, Side::Buy, 100, 10);
    let buy2 = Order::new(2, Side::Buy, 90, 5);
    let buy3 = Order::new(3, Side::Buy, 90, 50);

    let sell = Order::new(4, Side::Sell, 50, 10);
    let sell2 = Order::new(5, Side::Sell, 90, 4);
    let sell3 = Order::new(6, Side::Sell, 50, 3);

    let mut orderbook = OrderBook::new();

    orderbook.add_order(buy);
    orderbook.add_order(buy2);
    orderbook.add_order(buy3);
    
    orderbook.add_order(sell);
    orderbook.add_order(sell2);
    orderbook.add_order(sell3);

    println!{"{:?}", orderbook};
}
