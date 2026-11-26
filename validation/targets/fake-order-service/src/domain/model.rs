use application::CreateOrder;

pub struct Order;

pub const ORDER_POLICY: &str = "orders must be paid before shipping";

pub fn describe(_: CreateOrder) -> Order {
    Order
}
