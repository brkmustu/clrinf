import { dispatch } from "./core.js";
import { GreetingModule } from "./greeting.js";

const result = dispatch(
  new GreetingModule(),
  { name: "World" },
  { correlationId: "demo-request" },
);

if (!result.ok) {
  throw new Error(`${result.error.code}: ${result.error.message}`);
}
console.log(`${result.value.message} [${result.value.correlationId}]`);
