defmodule Clrinfex.Core.Outbox do
  @moduledoc """
  Port for pending canonical events. Keys are scoped by tenant and event ID.

  A durable implementation must enlist append with application state and inbox
  changes in the SAME storage transaction. This port alone cannot guarantee
  atomic business writes, broker acknowledgements, or exactly-once delivery.
  A dispatcher acknowledges only after the chosen transport confirms acceptance;
  retries can duplicate delivery, so consumers need idempotency.
  """
  alias Clrinfex.Core.{CloudEvent, Result}

  @callback append(GenServer.server(), CloudEvent.t()) :: Result.t(CloudEvent.t())
  @callback pending(GenServer.server(), String.t(), pos_integer()) :: Result.t([CloudEvent.t()])
  @callback acknowledge(GenServer.server(), String.t(), String.t()) :: Result.t(CloudEvent.t())
end
