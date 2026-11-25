use application::Service;
use domain::Entity;

pub struct Client {
    entity: Entity,
    service: Service,
}
