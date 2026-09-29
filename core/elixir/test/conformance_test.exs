defmodule Clrinfex.ConformanceTest do
  use ExUnit.Case, async: true
  alias Clrinfex.Core.{CloudEvent, Context, Error}

  defp fixture(name) do
    directory =
      System.get_env("CLRINF_CONFORMANCE_DIR") ||
        Path.expand("../../tests/conformance/fixtures", __DIR__)

    directory |> Path.join(name <> ".json") |> File.read!() |> Jason.decode!()
  end

  test "shared polyglot contracts validate and round trip without losing flags" do
    valid = fixture("valid")

    for {key, module} <- [{"context", Context}, {"error", Error}, {"event", CloudEvent}] do
      value = Map.fetch!(valid, key)
      assert {:ok, ^value} = module.validate(value)
      assert value == value |> Jason.encode!() |> Jason.decode!()
    end

    assert {:ok, json} = CloudEvent.encode(valid["event"])
    assert {:ok, event} = CloudEvent.decode(json)
    assert event == valid["event"]
  end

  test "shared invalid cases fail with standard error envelopes" do
    invalid = fixture("invalid")

    for {key, module} <- [{"context", Context}, {"error", Error}, {"event", CloudEvent}],
        %{"name" => name, "value" => value} <- Map.fetch!(invalid, key) do
      assert {:error, error} = module.validate(value), "Accepted invalid #{key}: #{name}"
      assert {:ok, ^error} = Error.validate(error)
    end
  end
end
