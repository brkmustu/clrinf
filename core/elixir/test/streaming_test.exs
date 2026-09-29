defmodule Clrinfex.StreamingTest do
  use ExUnit.Case, async: false
  import Plug.Test
  import Plug.Conn
  alias Clrinfex.Core.{CloudEvent, Context, Error}

  defp event do
    {:ok, context} = Context.new("test-tenant", "test-correlation", "test-request")
    {:ok, event} = CloudEvent.new(context, "clrinf/test", "com.clrinf.test.Changed.v1", %{})
    event
  end

  setup do
    start_supervised!({Clrinfex.NatsBridge, nats_url: nil})
    start_supervised!(Clrinfex.Presence)
    :ok
  end

  test "health reports the event bus mode" do
    conn = Clrinfex.Router.call(conn(:get, "/health"), Clrinfex.Router.init([]))
    assert conn.status == 200
    assert Jason.decode!(conn.resp_body)["nats_mode"] == "standalone"
  end

  test "broadcast endpoint rejects invalid event subjects" do
    :ok = Clrinfex.NatsBridge.subscribe(self())

    for type <- ["bad.>", "com.clrinf.test.Changed.v1\n"] do
      conn =
        conn(:post, "/api/broadcast", Jason.encode!(Map.put(event(), "type", type)))
        |> put_req_header("content-type", "application/json")
        |> Clrinfex.Router.call(Clrinfex.Router.init([]))

      assert conn.status == 400
      assert {:ok, _} = Error.validate(Jason.decode!(conn.resp_body))
      refute_receive {:stream_event, _}
    end
  end

  test "oversized HTTP requests return a standard 413 without local delivery" do
    :ok = Clrinfex.NatsBridge.subscribe(self())
    oversized = Map.put(event(), "data", %{"padding" => String.duplicate("x", 1_048_576)})

    response =
      conn(:post, "/api/broadcast", Jason.encode!(oversized))
      |> put_req_header("content-type", "application/json")
      |> Clrinfex.Router.call(Clrinfex.Router.init([]))

    assert response.status == 413
    assert {:ok, error} = Error.validate(Jason.decode!(response.resp_body))
    assert error["error_code"] == "PAYLOAD_TOO_LARGE"
    assert error["retryable"] == false
    refute_receive {:stream_event, _}
  end

  test "broadcast endpoint preserves canonical envelope and flags" do
    :ok = Clrinfex.NatsBridge.subscribe(self())
    event = Map.merge(event(), %{"is_error" => false, "is_compensation" => true})

    response =
      conn(:post, "/api/broadcast", Jason.encode!(event))
      |> put_req_header("content-type", "application/json")
      |> Clrinfex.Router.call(Clrinfex.Router.init([]))

    assert response.status == 202
    assert Jason.decode!(response.resp_body)["event"] == event
    assert_receive {:stream_event, ^event}
  end

  test "HTTP example and in-process invocation share the same core operation" do
    response =
      conn(:post, "/api/example/documents/approve", Jason.encode!(%{"document_id" => "doc-1"}))
      |> put_req_header("content-type", "application/json")
      |> put_req_header("x-tenant-id", "tenant")
      |> put_req_header("x-correlation-id", "workflow")
      |> put_req_header("x-causation-id", "request")
      |> Clrinfex.Router.call(Clrinfex.Router.init([]))

    assert response.status == 200
    assert {:ok, event} = CloudEvent.validate(Jason.decode!(response.resp_body))
    assert event["tenantid"] == "tenant"
    assert event["correlationid"] == "workflow"
    assert event["causationid"] == "request"
    assert event["data"] == %{"document_id" => "doc-1", "status" => "approved"}
  end

  test "missing metadata, malformed JSON, legacy external messages and not found use error envelopes" do
    requests = [
      conn(:post, "/api/example/documents/approve", "{}"),
      conn(:post, "/api/broadcast", "{bad"),
      conn(:post, "/api/broadcast", ~s({"type":"presence_update"})),
      conn(:get, "/does-not-exist")
    ]

    for request <- requests do
      response =
        request
        |> put_req_header("content-type", "application/json")
        |> Clrinfex.Router.call(Clrinfex.Router.init([]))

      assert response.status in [400, 404]
      assert {:ok, error} = Error.validate(Jason.decode!(response.resp_body))
      assert error["retryable"] == false
    end
  end

  test "configured NATS outage returns 503 for health and broadcast" do
    stop_supervised!(Clrinfex.NatsBridge)
    {:ok, listener} = :gen_tcp.listen(0, [:binary, active: false])
    {:ok, {_address, port}} = :inet.sockname(listener)
    :ok = :gen_tcp.close(listener)
    start_supervised!({Clrinfex.NatsBridge, nats_url: "nats://127.0.0.1:#{port}"})

    health = Clrinfex.Router.call(conn(:get, "/health"), Clrinfex.Router.init([]))
    assert health.status == 503
    assert Jason.decode!(health.resp_body)["nats_mode"] == "disconnected"

    broadcast =
      conn(:post, "/api/broadcast", Jason.encode!(event()))
      |> put_req_header("content-type", "application/json")
      |> Clrinfex.Router.call(Clrinfex.Router.init([]))

    assert broadcast.status == 503
    assert {:ok, error} = Error.validate(Jason.decode!(broadcast.resp_body))
    assert error["error_code"] == "BUS_UNAVAILABLE"
    assert error["retryable"] == true
    assert error["tenant_id"] == "test-tenant"
  end

  test "targeted signals retain target_user_id and generic signals still work" do
    :ok = Clrinfex.NatsBridge.subscribe(self())
    state = %{stream_id: "room", user_id: "sender"}

    for target <- [nil, "recipient"] do
      payload = %{"action" => "signal", "signal" => %{"type" => "offer"}}
      payload = if target, do: Map.put(payload, "target_user_id", target), else: payload

      assert {:ok, ^state} =
               Clrinfex.StreamingSocket.websocket_handle({:text, Jason.encode!(payload)}, state)

      assert_receive {:stream_event, event}
      assert event["from_user_id"] == "sender"
      assert event["stream_id"] == "room"
      assert event["target_user_id"] == target
    end
  end
end
