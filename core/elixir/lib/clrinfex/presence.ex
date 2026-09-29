defmodule Clrinfex.Presence do
  @moduledoc """
  Real-time distributed presence & live streaming participant tracker.
  """
  use GenServer
  require Logger

  @table :clrinf_presence_store

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc "Track a user/client joining a stream channel"
  def track(stream_id, user_id, meta \\ %{}) do
    GenServer.call(__MODULE__, {:track, stream_id, user_id, meta})
  end

  @doc "Untrack a user/client leaving a stream channel"
  def untrack(stream_id, user_id) do
    GenServer.call(__MODULE__, {:untrack, stream_id, user_id})
  end

  @doc "List active participants and viewer count for a stream channel"
  def list_participants(stream_id) do
    case :ets.lookup(@table, stream_id) do
      [{^stream_id, participants}] -> participants
      [] -> []
    end
  end

  @doc "Get count of active participants in a stream channel"
  def count(stream_id) do
    length(list_participants(stream_id))
  end

  @doc "List all active stream channels"
  def list_active_streams do
    :ets.tab2list(@table)
    |> Enum.map(fn {stream_id, participants} ->
      %{stream_id: stream_id, viewers_count: length(participants)}
    end)
  end

  @impl true
  def init(_opts) do
    :ets.new(@table, [:named_table, :set, :public, read_concurrency: true])
    Logger.info("[Clrinfex.Presence] In-memory presence store initialized")
    {:ok, %{}}
  end

  @impl true
  def handle_call({:track, stream_id, user_id, meta}, _from, state) do
    current = list_participants(stream_id)
    entry = %{user_id: user_id, joined_at: DateTime.utc_now() |> DateTime.to_iso8601(), meta: meta}
    updated = [entry | Enum.reject(current, &(&1.user_id == user_id))]
    :ets.insert(@table, {stream_id, updated})
    {:reply, {:ok, length(updated)}, state}
  end

  @impl true
  def handle_call({:untrack, stream_id, user_id}, _from, state) do
    current = list_participants(stream_id)
    updated = Enum.reject(current, &(&1.user_id == user_id))
    if updated == [] do
      :ets.delete(@table, stream_id)
    else
      :ets.insert(@table, {stream_id, updated})
    end
    {:reply, :ok, state}
  end
end
