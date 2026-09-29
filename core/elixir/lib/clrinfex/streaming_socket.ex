defmodule Clrinfex.StreamingSocket do
  @moduledoc """
  Cowboy / Plug WebSocket handler for real-time CloudEvents, Presence, and WebRTC Audio/Video Stream Signaling.
  """
  @behaviour :cowboy_websocket
  require Logger

  @impl true
  def init(req, _opts) do
    {:cowboy_websocket, req, %{stream_id: nil, user_id: nil}, %{max_frame_size: 1_000_000}}
  end

  @impl true
  def websocket_init(state) do
    Clrinfex.NatsBridge.subscribe(self())
    Logger.info("[StreamingSocket] Client connected (pid=#{inspect(self())})")
    {:ok, state}
  end

  @impl true
  def websocket_handle({type, json_str}, state) when type in [:text, :binary] do
    case Jason.decode(json_str) do
      {:ok, %{"action" => "join", "stream_id" => stream_id, "user_id" => user_id} = payload} ->
        meta = Map.get(payload, "meta", %{})
        {:ok, viewer_count} = Clrinfex.Presence.track(stream_id, user_id, meta)

        Logger.info(
          "[StreamingSocket] JOIN user=#{user_id} stream=#{stream_id} viewers=#{viewer_count}"
        )

        reply = %{
          "type" => "presence_update",
          "stream_id" => stream_id,
          "viewers_count" => viewer_count,
          "participants" => Clrinfex.Presence.list_participants(stream_id)
        }

        Clrinfex.NatsBridge.broadcast_local(reply)
        {:ok, %{state | stream_id: stream_id, user_id: user_id}}

      {:ok, %{"action" => "signal", "target_user_id" => target, "signal" => signal}} ->
        signal_type = signal["type"] || "unknown"

        Logger.info(
          "[StreamingSocket] SIGNAL from=#{state.user_id} -> #{target} type=#{signal_type}"
        )

        reply = %{
          "type" => "signal",
          "from_user_id" => state.user_id,
          "target_user_id" => target,
          "stream_id" => state.stream_id,
          "signal" => signal
        }

        Clrinfex.NatsBridge.broadcast_local(reply)
        {:ok, state}

      {:ok, %{"action" => "signal", "signal" => signal}} ->
        signal_type = signal["type"] || "unknown"

        Logger.info(
          "[StreamingSocket] SIGNAL from=#{state.user_id} type=#{signal_type} size=#{byte_size(json_str)}"
        )

        reply = %{
          "type" => "signal",
          "from_user_id" => state.user_id,
          "stream_id" => state.stream_id,
          "signal" => signal
        }

        Clrinfex.NatsBridge.broadcast_local(reply)
        {:ok, state}

      {:ok, %{"action" => "ping"}} ->
        {:reply,
         {:text,
          Jason.encode!(%{
            "type" => "pong",
            "time" => DateTime.utc_now() |> DateTime.to_iso8601()
          })}, state}

      {:ok, other} ->
        Logger.warning("[StreamingSocket] UNMATCHED action=#{inspect(Map.get(other, "action"))}")
        {:ok, state}

      {:error, reason} ->
        Logger.error("[StreamingSocket] JSON DECODE ERROR: #{inspect(reason)}")
        {:ok, state}
    end
  end

  @impl true
  def websocket_handle(_frame, state) do
    {:ok, state}
  end

  @impl true
  def websocket_info({:stream_event, event}, state) do
    {:reply, {:text, Jason.encode!(event)}, state}
  end

  @impl true
  def websocket_info(_info, state) do
    {:ok, state}
  end

  @impl true
  def terminate(_reason, _req, state) do
    if state.stream_id && state.user_id do
      Logger.info("[StreamingSocket] LEAVE user=#{state.user_id} stream=#{state.stream_id}")
      Clrinfex.Presence.untrack(state.stream_id, state.user_id)

      reply = %{
        "type" => "presence_update",
        "stream_id" => state.stream_id,
        "viewers_count" => Clrinfex.Presence.count(state.stream_id),
        "participants" => Clrinfex.Presence.list_participants(state.stream_id)
      }

      Clrinfex.NatsBridge.broadcast_local(reply)
    end

    Clrinfex.NatsBridge.unsubscribe(self())
    :ok
  end
end
