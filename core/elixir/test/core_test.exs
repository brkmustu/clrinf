defmodule Clrinfex.CoreTest do
  use ExUnit.Case, async: true
  alias Clrinfex.Core.{CloudEvent, Context, Error, Result}

  test "context, standard error and event compose with native result tuples" do
    assert {:ok, context} = Context.new("tenant", "correlation", "cause")
    assert {:ok, child} = Context.child(context, "event-1")
    assert child == Map.put(context, "causation_id", "event-1")

    assert {:ok, error} =
             Error.new(context, "NOT_FOUND", "Document not found", false, %{"id" => "doc"})

    assert error["details"] == %{"id" => "doc"}

    assert Result.bind({:error, error}, fn _ -> flunk("failure was not short-circuited") end) ==
             {:error, error}

    assert Result.map({:error, error}, fn _ -> flunk("failure was not short-circuited") end) ==
             {:error, error}

    assert Result.map(Result.ok(1), &(&1 + 1)) == {:ok, 2}
    assert Result.bind(Result.ok(context), &Context.validate/1) == {:ok, context}
    assert Result.error(error) == {:error, error}
  end

  test "event validation rejects malformed fields and non JSON values" do
    {:ok, context} = Context.new("tenant", "correlation", "cause")
    {:ok, event} = CloudEvent.new(context, "clrinf/test", "com.clrinf.test.Changed.v1", %{})

    for {field, value} <- [
          {"id", " "},
          {"source", nil},
          {"time", "not-a-time"},
          {"time", "2026-09-09T03:00:00"},
          {"data", []},
          {"data", %{atom: :value}},
          {"data", %{"pid" => self()}},
          {"specversion", "2.0"},
          {"datacontenttype", "text/plain"},
          {"is_error", nil},
          {"is_compensation", "false"}
        ] do
      assert {:error, error} = CloudEvent.validate(Map.put(event, field, value))
      assert {:ok, _} = Error.validate(error)
      assert error["error_code"] == "INVALID_EVENT"
    end

    for value <- [nil, 1, [], self(), Date.utc_today()] do
      assert {:error, _} = Context.validate(value)
      assert {:error, _} = Error.validate(value)
      assert {:error, _} = CloudEvent.validate(value)
    end

    assert {:error, _} = CloudEvent.decode("{")
    assert {:error, _} = CloudEvent.decode(nil)
    assert {:error, _} = Context.new("", "correlation", "cause")
  end

  test "core example needs neither HTTP nor a broker" do
    {:ok, context} = Context.new("tenant", "correlation", "cause")

    assert {:ok, event} =
             Clrinfex.Examples.Documents.approve(context, %{"document_id" => "doc-1"})

    assert {:ok, ^event} = CloudEvent.validate(event)
    assert {:ok, ^context} = CloudEvent.context(event)

    assert {:error, %{"error_code" => "INVALID_DOCUMENT"}} =
             Clrinfex.Examples.Documents.approve(context, %{})
  end
end
