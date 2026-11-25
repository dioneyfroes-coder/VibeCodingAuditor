import { Entity } from "../domain/entity";
import { Client } from "../infrastructure/client";

export function run(name: string) {
  return new Client().send(new Entity(name));
}
