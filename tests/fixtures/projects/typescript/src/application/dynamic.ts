export async function load() {
  const module = await import("../domain/entity");
  return module;
}
