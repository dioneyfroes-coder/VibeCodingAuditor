use auditor_core::Contract;
use crate::domain::Service;

pub fn build() -> String {
    let service = Service;
    format!("{service:?}")
}
