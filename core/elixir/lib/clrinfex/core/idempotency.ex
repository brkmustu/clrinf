defmodule Clrinfex.Core.Idempotency do
  @moduledoc """
  Tenant/consumer/key-scoped processing claims. A claim token fences completion
  and release, and completed results can be replayed without executing again.

  This is not an exactly-once engine. Durable implementations must transact
  claims/results with business writes and outbox, and define crash recovery,
  retention and lease policy. The memory reference has no automatic lease expiry.
  """
  alias Clrinfex.Core.{Context, Result}

  @callback claim(GenServer.server(), Context.t(), String.t(), String.t()) ::
              Result.t({:claimed, String.t()} | :in_progress | {:completed, term()})
  @callback complete(GenServer.server(), Context.t(), String.t(), String.t(), String.t(), term()) ::
              Result.t(term())
  @callback release(GenServer.server(), Context.t(), String.t(), String.t(), String.t()) ::
              Result.t(:released)
end
