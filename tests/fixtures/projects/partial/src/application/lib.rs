//! Aplicação com uma dependência conhecida e outra não mapeada.

use domain::Model;
use external_sdk::Client;

/// Caso de uso que precisa de decisão sobre `external_sdk`.
pub struct UseCase {
    model: Model,
    client: Client,
}
