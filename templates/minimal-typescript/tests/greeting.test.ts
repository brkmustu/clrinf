import { dispatch } from "../src/core.js";
import { GreetingModule } from "../src/greeting.js";

const module = new GreetingModule();
const context = { correlationId: "test-request" };
const result = dispatch(module, { name: " Ada " }, context);
if (
  !result.ok ||
  result.value.message !== "Hello, Ada!" ||
  result.value.correlationId !== "test-request"
) {
  throw new Error("Dispatch or context propagation failed.");
}

for (const name of ["", " \t\n"]) {
  const invalid = dispatch(module, { name }, context);
  if (invalid.ok || invalid.error.code !== "validation.name_required") {
    throw new Error("Blank name was not rejected.");
  }
}

console.log("All self-tests passed.");
