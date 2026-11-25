import { Entity } from "../domain/entity";

export class Client {
  send(entity: Entity) {
    return entity.name;
  }
}
