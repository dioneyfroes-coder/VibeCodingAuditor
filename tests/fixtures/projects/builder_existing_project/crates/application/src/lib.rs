use domain::Entity;
use serde::Serialize;

pub fn describe() -> String {
    format!("{:?}", Entity)
}
