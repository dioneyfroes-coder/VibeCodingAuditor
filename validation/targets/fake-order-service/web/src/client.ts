import { placeOrder } from 'application';

const dynamicTarget = import(moduleName);

export function submit(): void {
  placeOrder();
  void dynamicTarget;
}
