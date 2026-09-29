defmodule Clrinfex.NatsBridgeTest do
  use ExUnit.Case, async: false
  alias Clrinfex.NatsBridge

  defp start_bridge(url) do
    start_supervised!({NatsBridge, nats_url: url})
  end

  defp event do
    {:ok, context} = Clrinfex.Core.Context.new("test-tenant", "test-correlation", "test-cause")

    {:ok, event} =
      Clrinfex.Core.CloudEvent.new(context, "clrinf/test", "com.clrinf.test.Changed.v1", %{})

    event
  end

  defp await_mode(expected, attempts \\ 100)
  defp await_mode(expected, 0), do: assert(NatsBridge.mode() == expected)

  defp await_mode(expected, attempts) do
    if NatsBridge.mode() != expected do
      Process.sleep(20)
      await_mode(expected, attempts - 1)
    end
  end

  defp event_with_bytes(bytes) do
    message = Map.put(event(), "data", %{"padding" => ""})
    padding = String.duplicate("x", bytes - byte_size(Jason.encode!(message)))
    message = put_in(message, ["data", "padding"], padding)
    assert byte_size(Jason.encode!(message)) == bytes
    message
  end

  defp await_unsubscribed(bridge, attempts \\ 100)
  defp await_unsubscribed(bridge, 0), do: assert(:sys.get_state(bridge).subscribers == %{})

  defp await_unsubscribed(bridge, attempts) do
    if :sys.get_state(bridge).subscribers != %{} do
      Process.sleep(10)
      await_unsubscribed(bridge, attempts - 1)
    end
  end

  test "standalone broadcasts once and releases subscription monitors" do
    bridge = start_bridge(nil)
    assert NatsBridge.mode() == :standalone
    assert :ok = NatsBridge.subscribe(self())
    assert :ok = NatsBridge.subscribe(self())
    assert {:monitors, [{:process, pid}]} = Process.info(bridge, :monitors)
    assert pid == self()

    message = event()
    assert :ok = NatsBridge.broadcast_event(message)
    assert_receive {:stream_event, ^message}
    refute_receive {:stream_event, _}

    assert :ok = NatsBridge.unsubscribe(self())
    assert {:monitors, []} = Process.info(bridge, :monitors)
    assert :ok = NatsBridge.broadcast_event(message)
    refute_receive {:stream_event, _}
  end

  test "dead subscribers are removed" do
    bridge = start_bridge(nil)
    subscriber = spawn(fn -> Process.sleep(:infinity) end)
    assert :ok = NatsBridge.subscribe(subscriber)
    ref = Process.monitor(subscriber)
    Process.exit(subscriber, :kill)
    assert_receive {:DOWN, ^ref, :process, ^subscriber, :killed}
    await_unsubscribed(bridge)
  end

  test "standalone domain events enforce the exact encoded byte boundary" do
    start_bridge(nil)
    :ok = NatsBridge.subscribe(self())
    limit = NatsBridge.max_event_bytes()
    oversized = event_with_bytes(limit + 1)
    assert {:error, :event_too_large} = NatsBridge.broadcast_event(oversized)
    refute_receive {:stream_event, _}
    boundary = event_with_bytes(limit)
    assert :ok = NatsBridge.broadcast_event(boundary)
    assert_receive {:stream_event, ^boundary}
  end

  test "invalid subjects and payloads are rejected rather than silently dropped" do
    start_bridge(nil)

    for message <- [
          %{},
          %{"type" => nil},
          %{"type" => ""},
          %{"type" => "a.>"},
          %{"type" => "a b"},
          %{"type" => "a..b"},
          %{"type" => "ok", "data" => self()}
        ] do
      assert {:error, :invalid_event} = NatsBridge.broadcast_event(message)
    end
  end

  test "canonical validation applies even to legacy local message type names" do
    start_bridge(nil)
    :ok = NatsBridge.subscribe(self())
    invalid = event() |> Map.put("type", "signal") |> Map.delete("tenantid")
    assert {:error, :invalid_event} = NatsBridge.broadcast_event(invalid)
    refute_receive {:stream_event, _}
  end

  test "presence and signaling remain local during a configured outage" do
    start_bridge("nats://127.0.0.1:1")
    :ok = NatsBridge.subscribe(self())
    message = %{"type" => "presence_update", "stream_id" => "room", "viewers_count" => 1}
    assert :ok = NatsBridge.broadcast_local(message)
    assert_receive {:stream_event, ^message}
    assert :ok = NatsBridge.broadcast_event(message)
    assert_receive {:stream_event, ^message}
    assert {:error, :invalid_event} = NatsBridge.broadcast_local(event())

    assert {:error, :invalid_event} =
             NatsBridge.broadcast_local(%{"type" => "signal", "data" => self()})
  end

  test "invalid or unsupported URLs fail startup" do
    for url <- [
          "bad-url",
          "https://localhost:4222",
          "nats://localhost:0",
          "nats://localhost:4222/ignored",
          42
        ] do
      assert {:error, {:invalid_nats_url, _child}} =
               start_supervised({NatsBridge, nats_url: url})
    end
  end

  test "unavailable configured NATS is disconnected, not standalone or healthy delivery" do
    {:ok, listener} = :gen_tcp.listen(0, [:binary, active: false])
    {:ok, {_address, port}} = :inet.sockname(listener)
    :ok = :gen_tcp.close(listener)
    bridge = start_bridge("nats://127.0.0.1:#{port}")
    assert NatsBridge.mode() == :disconnected
    assert Process.alive?(bridge)
    assert :ok = NatsBridge.subscribe(self())
    assert {:error, :nats_unavailable} = NatsBridge.broadcast_event(event())
    refute_receive {:stream_event, _}
  end

  @tag :nats
  test "oversized publication is rejected before HPUB or fanout and keeps the connection usable" do
    bridge = start_bridge(System.fetch_env!("NATS_TEST_URL"))
    await_mode(:connected)
    :ok = NatsBridge.subscribe(self())
    %{nats: original, server_max_payload: max_payload, origin: origin} = :sys.get_state(bridge)
    assert Gnat.server_info(original).max_payload == max_payload
    headers = [{"clrinf-bridge-origin", origin}]
    header_bytes = IO.iodata_length(Gnat.Headers.encode(headers)) + 12
    budget = min(NatsBridge.max_event_bytes(), max_payload - header_bytes)
    oversized = event_with_bytes(budget + 1)

    response =
      Plug.Test.conn(:post, "/api/broadcast", Jason.encode!(oversized))
      |> Plug.Conn.put_req_header("content-type", "application/json")
      |> Clrinfex.Router.call(Clrinfex.Router.init([]))

    assert response.status == 413

    assert {:ok, %{"error_code" => "PAYLOAD_TOO_LARGE"}} =
             Clrinfex.Core.Error.validate(Jason.decode!(response.resp_body))

    assert {:error, :event_too_large} = NatsBridge.broadcast_event(oversized)
    refute_receive {:stream_event, _}, 200
    assert Process.alive?(original)
    assert NatsBridge.mode() == :connected
    assert :sys.get_state(bridge).nats == original
    boundary = event_with_bytes(budget)
    assert :ok = NatsBridge.broadcast_event(boundary)
    assert_receive {:stream_event, ^boundary}
    refute_receive {:stream_event, _}, 200
    assert :sys.get_state(bridge).nats == original
    assert Process.alive?(original)
  end

  @tag :nats
  test "real NATS traffic, echo suppression and reconnect preserve local subscriptions" do
    bridge = start_bridge(System.fetch_env!("NATS_TEST_URL"))
    await_mode(:connected)
    assert :ok = NatsBridge.subscribe(self())
    %{nats: original, settings: settings} = :sys.get_state(bridge)
    remote = start_supervised!(%{id: :remote, start: {Gnat, :start_link, [settings]}})
    assert {:ok, _sid} = Gnat.sub(remote, self(), "com.clrinf.test.>")
    marker = event()
    assert :ok = Gnat.pub(remote, marker["type"], Jason.encode!(marker))
    assert_receive {:msg, %{gnat: ^remote}}, 1_000
    assert_receive {:stream_event, ^marker}, 1_000

    local_event = event()
    assert :ok = NatsBridge.broadcast_event(local_event)
    assert_receive {:stream_event, ^local_event}, 1_000
    assert_receive {:msg, %{gnat: ^remote, body: payload}}, 1_000
    assert Jason.decode!(payload) == local_event
    refute_receive {:stream_event, _}, 200

    remote_event = event()
    assert :ok = Gnat.pub(remote, remote_event["type"], Jason.encode!(remote_event))
    assert_receive {:stream_event, ^remote_event}, 1_000
    refute_receive {:stream_event, _}, 100

    assert :ok = Gnat.pub(remote, remote_event["type"], "not-json")
    refute_receive {:stream_event, _}, 100

    assert :ok =
             Gnat.pub(
               remote,
               remote_event["type"],
               Jason.encode!(%{"type" => remote_event["type"]})
             )

    refute_receive {:stream_event, _}, 100
    assert :ok = Gnat.pub(remote, "com.clrinf.test.Wrong.v1", Jason.encode!(remote_event))
    refute_receive {:stream_event, _}, 100

    local_message = %{"type" => "signal", "signal" => %{"type" => "offer"}}
    assert {:ok, _sid} = Gnat.sub(remote, self(), "signal")
    assert :ok = NatsBridge.broadcast_local(local_message)
    assert_receive {:stream_event, ^local_message}
    refute_receive {:msg, %{topic: "signal"}}, 100

    ref = Process.monitor(original)
    Process.exit(original, :kill)
    assert_receive {:DOWN, ^ref, :process, ^original, :killed}
    # The bridge handles the linked exit before retrying, without losing subscribers.
    Process.sleep(20)
    await_mode(:disconnected)
    await_mode(:connected)
    assert Process.alive?(bridge)
    refute :sys.get_state(bridge).nats == original
    state = :sys.get_state(bridge)
    assert state.server_max_payload == Gnat.server_info(state.nats).max_payload

    after_reconnect = event()
    assert :ok = Gnat.pub(remote, after_reconnect["type"], Jason.encode!(after_reconnect))
    assert_receive {:stream_event, ^after_reconnect}, 1_000
    refute_receive {:stream_event, _}, 100

    connection = :sys.get_state(bridge).nats
    monitor = Process.monitor(connection)
    stop_supervised!(NatsBridge)
    assert_receive {:DOWN, ^monitor, :process, ^connection, _reason}, 1_000
  end
end
