mod order;
mod order_book;
mod trade;

use order::{Order, Side};
use order_book::{OrderBook};

fn main() {
    let mut orderbook = OrderBook::new();
    // Do work.
}
