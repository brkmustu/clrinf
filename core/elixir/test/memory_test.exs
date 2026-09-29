defmodule Clrinfex.MemoryTest do
  use ExUnit.Case, async: true
  alias Clrinfex.Adapters.Memory
  alias Clrinfex.Core.{CloudEvent, Context}

  setup do
    store = start_supervised!(Memory)
    {:ok, context} = Context.new("tenant-a", "workflow", "request")
    {:ok, event} = CloudEvent.new(context, "clrinf/test", "com.clrinf.test.Changed.v1", %{})
    %{store: store, context: context, event: event}
  end

  test "outbox is ordered, tenant scoped and rejects conflicting event IDs", %{
    store: store,
    event: event
  } do
    other_tenant = Map.put(event, "tenantid", "tenant-b")
    next = Map.put(event, "id", UUID.uuid4())
    assert {:ok, ^event} = Memory.append(store, event)
    assert {:ok, ^event} = Memory.append(store, event)
    assert {:ok, ^other_tenant} = Memory.append(store, other_tenant)
    assert {:ok, ^next} = Memory.append(store, next)

    assert {:error, %{"error_code" => "OUTBOX_CONFLICT"}} =
             Memory.append(store, Map.put(event, "data", %{"different" => true}))

    assert {:ok, [^event]} = Memory.pending(store, "tenant-a", 1)
    assert {:ok, [^event, ^next]} = Memory.pending(store, "tenant-a")
    assert {:ok, [^other_tenant]} = Memory.pending(store, "tenant-b")
    assert {:error, _} = Memory.acknowledge(store, "tenant-c", event["id"])
    assert {:ok, ^event} = Memory.acknowledge(store, "tenant-a", event["id"])
    assert {:ok, [^next]} = Memory.pending(store, "tenant-a")
    assert {:ok, [^other_tenant]} = Memory.pending(store, "tenant-b")
    assert {:error, _} = Memory.pending(store, "tenant-a", 0)
    assert {:error, _} = Memory.append(store, %{})
  end

  test "concurrent claims have one owner and completed values replay", %{
    store: store,
    context: context
  } do
    results =
      1..20
      |> Task.async_stream(fn _ -> Memory.claim(store, context, "consumer", "request-1") end)
      |> Enum.map(fn {:ok, result} -> result end)

    assert [{:ok, {:claimed, token}}] = Enum.filter(results, &match?({:ok, {:claimed, _}}, &1))
    assert Enum.count(results, &(&1 == {:ok, :in_progress})) == 19
    assert {:error, _} = Memory.complete(store, context, "consumer", "request-1", "wrong", %{})
    assert {:error, _} = Memory.release(store, context, "consumer", "request-1", "wrong")

    assert {:ok, result} =
             Memory.complete(store, context, "consumer", "request-1", token, %{"done" => true})

    assert {:ok, {:completed, ^result}} = Memory.claim(store, context, "consumer", "request-1")
    assert {:error, _} = Memory.release(store, context, "consumer", "request-1", token)

    assert {:ok, {:claimed, _}} =
             Memory.claim(
               store,
               Map.put(context, "tenant_id", "tenant-b"),
               "consumer",
               "request-1"
             )

    assert {:ok, {:claimed, _}} = Memory.claim(store, context, "other-consumer", "request-1")
  end

  test "release fences stale workers and restarts lose data", %{
    store: store,
    context: context,
    event: event
  } do
    {:ok, {:claimed, old_token}} = Memory.claim(store, context, "consumer", "key")
    assert {:ok, :released} = Memory.release(store, context, "consumer", "key", old_token)
    {:ok, {:claimed, new_token}} = Memory.claim(store, context, "consumer", "key")
    refute old_token == new_token
    assert {:error, _} = Memory.complete(store, context, "consumer", "key", old_token, %{})
    assert {:error, _} = Memory.complete(store, context, "consumer", "key", new_token, self())
    {:ok, _} = Memory.append(store, event)
    stop_supervised!(Memory)
    fresh = start_supervised!(Memory)
    assert {:ok, []} = Memory.pending(fresh, "tenant-a")
    assert {:ok, {:claimed, _}} = Memory.claim(fresh, context, "consumer", "key")
  end
end
