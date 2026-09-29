# Run: CLRINF_MODE=core mix run examples/monolith.exs
alias Clrinfex.Core.{CloudEvent, Context, Result}
alias Clrinfex.Examples.Documents

{:ok, context} = Context.new("example-tenant", "example-workflow", "example-request")

# Modules compose directly; no HTTP server, broker or distributed transaction.
{:ok, event} =
  context
  |> Documents.approve(%{"document_id" => "document-1"})
  |> Result.bind(&CloudEvent.validate/1)

IO.puts(Jason.encode!(event, pretty: true))
