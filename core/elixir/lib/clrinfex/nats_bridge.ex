defmodule Clrinfex.NatsBridge do
  @moduledoc """
  Live NATS Core fan-out for local SSE and WebSocket subscribers.

  With no NATS URL, events remain process-local (`:standalone`). When NATS is
  configured, connection failures report `:disconnected` and are retried;
  publishing fails explicitly until the connection is restored. This is not a
  durable JetStream consumer: missed messages are not replayed.
  """
  use GenServer
  require Logger
  alias Clrinfex.Core.{CloudEvent, Validation}

  @subject "com.clrinf.>"
  @origin_header "clrinf-bridge-origin"
  @retry_interval 1_000
  @max_event_bytes 256 * 1024

  @doc "Maximum encoded domain event size, in bytes, independent of broker limits."
  def max_event_bytes, do: @max_event_bytes

  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Publishes a canonical domain event. Legacy presence/signal maps remain local
  for compatibility; new callers should use `broadcast_local/1` explicitly.
  """
  def broadcast_event(%{"type" => type} = event)
      when type in ["presence_update", "signal"] and not is_map_key(event, "specversion"),
      do: broadcast_local(event)

  def broadcast_event(event), do: GenServer.call(__MODULE__, {:broadcast, event})

  @doc "Local-only presence/signaling, never sent to or accepted from NATS."
  def broadcast_local(message), do: GenServer.call(__MODULE__, {:local, message})
  def subscribe(pid), do: GenServer.call(__MODULE__, {:subscribe, pid})
  def unsubscribe(pid), do: GenServer.call(__MODULE__, {:unsubscribe, pid})

  @doc "Returns `:connected`, `:disconnected` or `:standalone`."
  def mode, do: GenServer.call(__MODULE__, :mode)

  @impl true
  def init(opts) do
    Process.flag(:trap_exit, true)
    url = Keyword.get(opts, :nats_url, Application.get_env(:clrinfex, :nats_url))

    with {:ok, settings} <- connection_settings(url) do
      state = %{
        subscribers: %{},
        nats: nil,
        server_max_payload: nil,
        settings: settings,
        subject:
          Keyword.get(opts, :subject, Application.get_env(:clrinfex, :nats_subject, @subject)),
        origin: UUID.uuid4(),
        retry: nil
      }

      if settings, do: {:ok, state, {:continue, :connect}}, else: {:ok, state}
    else
      {:error, reason} -> {:stop, reason}
    end
  end

  @impl true
  def handle_continue(:connect, state), do: connect(state)

  @impl true
  def handle_call({:subscribe, pid}, _from, state) do
    subscribers = Map.put_new_lazy(state.subscribers, pid, fn -> Process.monitor(pid) end)
    {:reply, :ok, %{state | subscribers: subscribers}}
  end

  def handle_call({:unsubscribe, pid}, _from, state) do
    {ref, subscribers} = Map.pop(state.subscribers, pid)
    if ref, do: Process.demonitor(ref, [:flush])
    {:reply, :ok, %{state | subscribers: subscribers}}
  end

  def handle_call(:mode, _from, state) do
    mode =
      cond do
        state.nats -> :connected
        state.settings -> :disconnected
        true -> :standalone
      end

    {:reply, mode, state}
  end

  def handle_call({:broadcast, event}, _from, state) do
    with {:ok, type, payload} <- encode_event(event),
         :ok <- publish(state, type, payload) do
      fan_out(state.subscribers, event)
      {:reply, :ok, state}
    else
      {:error, reason} = error ->
        Logger.warning("[Clrinfex.NatsBridge] Broadcast rejected: #{inspect(reason)}")
        {:reply, error, state}
    end
  end

  def handle_call({:local, %{"type" => type} = message}, _from, state)
      when type in ["presence_update", "signal"] do
    if Validation.json?(message) do
      fan_out(state.subscribers, message)
      {:reply, :ok, state}
    else
      Logger.warning("[Clrinfex.NatsBridge] Invalid local message")
      {:reply, {:error, :invalid_event}, state}
    end
  end

  def handle_call({:local, _message}, _from, state) do
    Logger.warning("[Clrinfex.NatsBridge] Unsupported local message")
    {:reply, {:error, :invalid_event}, state}
  end

  @impl true
  def handle_info(:connect, state), do: connect(%{state | retry: nil})

  def handle_info({:msg, %{gnat: conn, body: body} = message}, %{nats: conn} = state) do
    # Locally published messages have already been delivered to local subscribers.
    unless {@origin_header, state.origin} in Map.get(message, :headers, []) do
      with :ok <- check_size(byte_size(body), @max_event_bytes),
           {:ok, event} <- CloudEvent.decode(body),
           {:ok, type, _payload} <- encode_event(event),
           true <- type == Map.get(message, :topic) do
        fan_out(state.subscribers, event)
      else
        _ -> Logger.warning("[Clrinfex.NatsBridge] Rejected invalid NATS event")
      end
    end

    {:noreply, state}
  end

  def handle_info({:EXIT, conn, _reason}, %{nats: conn} = state) do
    Logger.warning("[Clrinfex.NatsBridge] NATS disconnected; retrying")
    {:noreply, schedule_retry(%{state | nats: nil, server_max_payload: nil})}
  end

  def handle_info({:DOWN, ref, :process, pid, _reason}, state) do
    subscribers =
      if Map.get(state.subscribers, pid) == ref,
        do: Map.delete(state.subscribers, pid),
        else: state.subscribers

    {:noreply, %{state | subscribers: subscribers}}
  end

  # A failed start or a closed connection can leave a late exit/message queued.
  def handle_info({:EXIT, _pid, _reason}, state), do: {:noreply, state}
  def handle_info({:msg, _message}, state), do: {:noreply, state}

  @impl true
  def terminate(_reason, state) do
    if state.retry, do: Process.cancel_timer(state.retry)
    if state.nats, do: Process.exit(state.nats, :shutdown)
  end

  defp connect(state) do
    case Gnat.start_link(state.settings) do
      {:ok, conn} ->
        result =
          nats_call(fn ->
            with %{max_payload: max_payload} when is_integer(max_payload) and max_payload > 0 <-
                   Gnat.server_info(conn),
                 {:ok, _sid} <- Gnat.sub(conn, self(), state.subject) do
              {:ok, max_payload}
            else
              {:error, _} = error -> error
              _ -> {:error, :invalid_server_info}
            end
          end)

        case result do
          {:ok, max_payload} ->
            Logger.info("[Clrinfex.NatsBridge] Connected; subscribed to #{state.subject}")
            {:noreply, %{state | nats: conn, server_max_payload: max_payload}}

          {:error, _reason} ->
            Process.exit(conn, :shutdown)
            connection_failed(state)
        end

      {:error, _reason} ->
        connection_failed(state)
    end
  end

  defp connection_failed(state) do
    Logger.warning("[Clrinfex.NatsBridge] NATS connection failed; retrying")
    {:noreply, schedule_retry(state)}
  end

  defp schedule_retry(%{retry: nil} = state) do
    %{state | retry: Process.send_after(self(), :connect, @retry_interval)}
  end

  defp schedule_retry(state), do: state

  defp fan_out(subscribers, event) do
    for {pid, _ref} <- subscribers, do: send(pid, {:stream_event, event})
  end

  defp connection_settings(nil), do: {:ok, nil}

  defp connection_settings(url) when is_binary(url) do
    case URI.new(url) do
      {:ok,
       %URI{
         scheme: "nats",
         host: host,
         port: port,
         userinfo: userinfo,
         path: path,
         query: nil,
         fragment: nil
       }}
      when is_binary(host) and host != "" and (is_nil(port) or port in 1..65535) and
             path in [nil, "", "/"] ->
        settings = %{host: host, port: port || 4222, connection_timeout: 1_000}

        credentials =
          case userinfo do
            nil ->
              %{}

            info ->
              case String.split(info, ":", parts: 2) do
                [user, password] ->
                  %{username: URI.decode(user), password: URI.decode(password)}

                [token] ->
                  %{token: URI.decode(token)}
              end
          end

        {:ok, Map.merge(settings, credentials)}

      _ ->
        {:error, :invalid_nats_url}
    end
  end

  defp connection_settings(_url), do: {:error, :invalid_nats_url}

  defp encode_event(%{"type" => type} = event) when is_binary(type) do
    with true <- Regex.match?(~r/\A[^\s.*>]+(?:\.[^\s.*>]+)*\z/u, type),
         {:ok, payload} <- CloudEvent.encode(event),
         :ok <- check_size(byte_size(payload), @max_event_bytes) do
      {:ok, type, payload}
    else
      {:error, :event_too_large} = error -> error
      _ -> {:error, :invalid_event}
    end
  end

  defp encode_event(_event), do: {:error, :invalid_event}

  defp publish(%{settings: nil}, _type, _payload), do: :ok
  defp publish(%{nats: nil}, _type, _payload), do: {:error, :nats_unavailable}

  defp publish(state, type, payload) do
    headers = [{@origin_header, state.origin}]
    # HPUB includes "NATS/1.0\r\n" and the blank line before the JSON body.
    wire_size = byte_size(payload) + IO.iodata_length(Gnat.Headers.encode(headers)) + 12

    with :ok <- check_size(wire_size, state.server_max_payload) do
      nats_call(fn -> Gnat.pub(state.nats, type, payload, headers: headers) end)
    end
  end

  defp check_size(size, limit) when size <= limit, do: :ok
  defp check_size(_size, _limit), do: {:error, :event_too_large}

  # A connection can exit between the mode check and a synchronous Gnat call.
  defp nats_call(fun) do
    fun.()
  catch
    :exit, {_reason, {GenServer, :call, _args}} -> {:error, :nats_unavailable}
  end
end
