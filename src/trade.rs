#[derive(Debug)]
pub struct Trade {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub price: u64,
    pub quantity: u64,
}

impl Trade {
    pub fn new(buy_order_id: u64, sell_order_id: u64, price: u64, quantity: u64) -> Self {
        Self {
            buy_order_id,
            sell_order_id,
            price,
            quantity,
        }
    }
}