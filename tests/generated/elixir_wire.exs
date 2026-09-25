ExUnit.start()

defmodule GeneratedWireTest do
  use ExUnit.Case
  alias ClrInf.Contracts.Common.RecordV1

  test "wire keys survive map conversion without dynamic atoms" do
    payload = %{
      "displayName" => "sample",
      "enabled" => true,
      "nested" => [[1, 2]],
      "payload" => %{"value" => "nested"},
      "type" => "first",
      "records" => [%{"any" => 7}]
    }
    assert {:ok, record} = RecordV1.from_map(payload)
    assert RecordV1.to_map(record) == payload
    envelope = RecordV1.to_cloudevent(record,
      tenant_id: "tenant", correlation_id: "workflow", causation_id: "parent", id: "message")
    assert envelope["data"] == payload
    assert envelope["tenantid"] == "tenant"
    assert envelope["type"] == "test.Record.v1"
  end

  test "missing required keys and non-maps fail explicitly" do
    assert {:error, _} = RecordV1.from_map(%{})
    assert {:error, _} = RecordV1.from_map([])
  end
end
