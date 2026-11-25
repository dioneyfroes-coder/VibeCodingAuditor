//! Casos de uso permitidos a depender apenas do domínio.

use domain::Entity;

/// Serviço de aplicação.
pub struct Service {
    entity: Entity,
}
