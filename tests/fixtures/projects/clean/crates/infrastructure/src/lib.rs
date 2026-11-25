//! Adaptadores que podem depender de domínio e aplicação.

use application::Service;
use domain::Entity;

/// Cliente que monta um serviço a partir de uma entidade.
pub struct Client {
    service: Service,
    entity: Entity,
}
