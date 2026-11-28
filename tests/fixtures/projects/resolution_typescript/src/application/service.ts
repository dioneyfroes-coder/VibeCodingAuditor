import { Entity } from "@domain/entity";
import { duploAplicacao } from "@local/duplo";
import { ausente } from "@local/ausente";
import { readFileSync } from "node:fs";
import React from "react";
import desconhecido from "desconhecido/nested";
import { Api } from "@demo/api";
export { Entity as Dominio } from "domain";

export function servico(): string {
  return `${Entity.name} ${duploAplicacao} ${ausente} ${readFileSync} ${React} ${desconhecido} ${Api}`;
}
