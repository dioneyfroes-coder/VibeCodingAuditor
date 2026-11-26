use application::place_order;
use infrastructure::Database;

pub fn handle() {
    place_order();
    let _ = Database;
}
