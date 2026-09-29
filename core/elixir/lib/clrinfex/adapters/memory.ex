defmodule Clrinfex.Adapters.Memory do
  @moduledoc """
  Serialized, VOLATILE reference for Outbox and Idempotency.

  Each GenServer call is atomic only within this process. There is no disk,
  replication, cross-call transaction, business-state transaction or durability.
  Restart loses pending events, completed results and claims. Claims have no
  timeout; a failed worker must explicitly release its token or remain blocked.
  Completed results remain in memory without eviction until the process stops.
  Use a transactional storage adapter for production delivery guarantees.
  """
  use GenServer
  @behaviour Clrinfex.Core.Outbox
  @behaviour Clrinfex.Core.Idempotency
  alias Clrinfex.Core.{CloudEvent, Context, Error, Validation}

  def start_link(options \\ []), do: GenServer.start_link(__MODULE__, nil, options)

  @impl Clrinfex.Core.Outbox
  def append(server, event) do
    with {:ok, event} <- CloudEvent.validate(event),
         do: GenServer.call(server, {:append, event})
  end

  @impl Clrinfex.Core.Outbox
  def pending(server, tenant, limit \\ 100) do
    if Validation.text?(tenant) and is_integer(limit) and limit > 0 do
      GenServer.call(server, {:pending, tenant, limit})
    else
      {:error, Validation.invalid("INVALID_OUTBOX_QUERY", ["tenant_id", "limit"], %{})}
    end
  end

  @impl Clrinfex.Core.Outbox
  def acknowledge(server, tenant, id) do
    with {:ok, context} <- Context.new(tenant, id, id),
         do: GenServer.call(server, {:acknowledge, context, id})
  end

  @impl Clrinfex.Core.Idempotency
  def claim(server, context, consumer, key) do
    with {:ok, scope} <- scope(context, consumer, key),
         do: GenServer.call(server, {:claim, scope})
  end

  @impl Clrinfex.Core.Idempotency
  def complete(server, context, consumer, key, token, result) do
    with {:ok, scope} <- scope(context, consumer, key) do
      if Validation.json?(result) do
        GenServer.call(server, {:complete, scope, token, result, context})
      else
        {:error, Validation.invalid("INVALID_RESULT", ["result"], context)}
      end
    end
  end

  @impl Clrinfex.Core.Idempotency
  def release(server, context, consumer, key, token) do
    with {:ok, scope} <- scope(context, consumer, key),
         do: GenServer.call(server, {:release, scope, token, context})
  end

  @impl GenServer
  def init(nil), do: {:ok, %{outbox: %{}, inbox: %{}, sequence: 0}}

  @impl GenServer
  def handle_call({:append, event}, _from, state) do
    key = {event["tenantid"], event["id"]}

    case Map.get(state.outbox, key) do
      nil ->
        sequence = state.sequence + 1
        outbox = Map.put(state.outbox, key, {sequence, event})
        {:reply, {:ok, event}, %{state | outbox: outbox, sequence: sequence}}

      {_sequence, ^event} ->
        {:reply, {:ok, event}, state}

      _ ->
        {:ok, context} = CloudEvent.context(event)

        {:reply, failure(context, "OUTBOX_CONFLICT", "Event ID already has a different payload"),
         state}
    end
  end

  def handle_call({:pending, tenant, limit}, _from, state) do
    events =
      state.outbox
      |> Enum.filter(fn {{scope, _id}, _entry} -> scope == tenant end)
      |> Enum.map(fn {_key, entry} -> entry end)
      |> Enum.sort_by(&elem(&1, 0))
      |> Enum.take(limit)
      |> Enum.map(&elem(&1, 1))

    {:reply, {:ok, events}, state}
  end

  def handle_call({:acknowledge, context, id}, _from, state) do
    case Map.pop(state.outbox, {context["tenant_id"], id}) do
      {nil, _outbox} ->
        {:reply, failure(context, "OUTBOX_NOT_FOUND", "Pending event not found"), state}

      {{_sequence, event}, outbox} ->
        {:reply, {:ok, event}, %{state | outbox: outbox}}
    end
  end

  def handle_call({:claim, scope}, _from, state) do
    case Map.get(state.inbox, scope) do
      nil ->
        token = UUID.uuid4()
        inbox = Map.put(state.inbox, scope, {:processing, token})
        {:reply, {:ok, {:claimed, token}}, %{state | inbox: inbox}}

      {:processing, _token} ->
        {:reply, {:ok, :in_progress}, state}

      {:completed, result} ->
        {:reply, {:ok, {:completed, result}}, state}
    end
  end

  def handle_call({:complete, scope, token, result, context}, _from, state) do
    case Map.get(state.inbox, scope) do
      {:processing, ^token} ->
        inbox = Map.put(state.inbox, scope, {:completed, result})
        {:reply, {:ok, result}, %{state | inbox: inbox}}

      _ ->
        {:reply,
         failure(
           context,
           "INVALID_CLAIM",
           "Claim is missing, completed or owned by another worker"
         ), state}
    end
  end

  def handle_call({:release, scope, token, context}, _from, state) do
    case Map.get(state.inbox, scope) do
      {:processing, ^token} ->
        {:reply, {:ok, :released}, %{state | inbox: Map.delete(state.inbox, scope)}}

      _ ->
        {:reply,
         failure(context, "INVALID_CLAIM", "Only the current processing claim can be released"),
         state}
    end
  end

  defp scope(context, consumer, key) do
    with {:ok, context} <- Context.validate(context) do
      if Validation.text?(consumer) and Validation.text?(key) do
        {:ok, {context["tenant_id"], consumer, key}}
      else
        {:error, Validation.invalid("INVALID_IDEMPOTENCY_KEY", ["consumer", "key"], context)}
      end
    end
  end

  defp failure(context, code, message) do
    {:ok, error} = Error.new(context, code, message)
    {:error, error}
  end
end
