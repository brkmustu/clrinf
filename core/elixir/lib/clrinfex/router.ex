defmodule Clrinfex.Router do
  use Plug.Router
  alias Clrinfex.Adapters.HTTP
  alias Clrinfex.Core.{CloudEvent, Context}

  plug(:match)
  plug(Clrinfex.Adapters.JSONBody)
  plug(:dispatch)

  get "/health" do
    mode = Clrinfex.NatsBridge.mode()
    ready = mode != :disconnected

    send_resp(
      conn,
      if(ready, do: 200, else: 503),
      Jason.encode!(%{
        status: if(ready, do: "healthy", else: "degraded"),
        service: "clrinfex-streaming",
        version: "0.1.0",
        nats_mode: mode,
        active_streams: Clrinfex.Presence.list_active_streams()
      })
    )
  end

  get "/metrics" do
    streams = Clrinfex.Presence.list_active_streams()
    total_viewers = Enum.reduce(streams, 0, fn s, acc -> acc + s.viewers_count end)

    metrics_text = """
    # HELP clrinf_streaming_active_channels Number of active live channels
    # TYPE clrinf_streaming_active_channels gauge
    clrinf_streaming_active_channels #{length(streams)}

    # HELP clrinf_streaming_total_viewers Total active stream viewers
    # TYPE clrinf_streaming_total_viewers gauge
    clrinf_streaming_total_viewers #{total_viewers}
    """

    conn
    |> put_resp_content_type("text/plain")
    |> send_resp(200, metrics_text)
  end

  get "/api/streams" do
    streams = Clrinfex.Presence.list_active_streams()
    send_resp(conn, 200, Jason.encode!(streams))
  end

  post "/api/broadcast" do
    case CloudEvent.validate(conn.body_params) do
      {:ok, event} ->
        {:ok, context} = CloudEvent.context(event)

        case Clrinfex.NatsBridge.broadcast_event(event) do
          :ok ->
            HTTP.json(conn, 202, %{status: "broadcasted", event: event})

          {:error, :invalid_event} ->
            HTTP.error(conn, context, "INVALID_EVENT", "Invalid event subject", 400)

          {:error, :event_too_large} ->
            HTTP.error(
              conn,
              context,
              "PAYLOAD_TOO_LARGE",
              "Event exceeds transport size limit",
              413
            )

          {:error, _reason} ->
            HTTP.error(conn, context, "BUS_UNAVAILABLE", "Event bus unavailable", 503, true)
        end

      {:error, error} ->
        HTTP.json(conn, 400, error)
    end
  end

  post "/api/example/documents/approve" do
    result =
      with {:ok, context} <- HTTP.context(conn),
           do: Clrinfex.Examples.Documents.approve(context, conn.body_params)

    HTTP.respond(conn, result)
  end

  get "/stream/sse" do
    conn =
      conn
      |> put_resp_header("content-type", "text/event-stream")
      |> put_resp_header("cache-control", "no-cache")
      |> put_resp_header("connection", "keep-alive")
      |> put_resp_header("access-control-allow-origin", "*")
      |> send_chunked(200)

    Clrinfex.NatsBridge.subscribe(self())

    sse_loop(conn)
  end

  match _ do
    context =
      case HTTP.context(conn) do
        {:ok, context} ->
          context

        {:error, error} ->
          {:ok, context} = Context.new(error["tenant_id"], error["correlation_id"], "unknown")
          context
      end

    HTTP.error(conn, context, "NOT_FOUND", "Not Found", 404)
  end

  defp sse_loop(conn) do
    receive do
      {:stream_event, event} ->
        case chunk(conn, "data: #{Jason.encode!(event)}\n\n") do
          {:ok, conn} ->
            sse_loop(conn)

          {:error, _reason} ->
            Clrinfex.NatsBridge.unsubscribe(self())
            conn
        end
    after
      25_000 ->
        # SSE Keep-alive heartbeat
        case chunk(conn, ": ping\n\n") do
          {:ok, conn} ->
            sse_loop(conn)

          {:error, _reason} ->
            Clrinfex.NatsBridge.unsubscribe(self())
            conn
        end
    end
  end
end
