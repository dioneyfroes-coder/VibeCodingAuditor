use domain::Order;
use infrastructure::Database;

pub struct CreateOrder;

pub const ORDER_POLICY: &str = "orders must be paid before shipping";

pub fn place_order() {
    let _ = (Order, Database);
}
