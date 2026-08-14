#[derive(Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
}

impl Order {
    pub fn new(id: u64, side: Side, price: u64, quantity: u64) -> Self {
        Self {
            id,
            side,
            price,
            quantity,
        }
    }
}