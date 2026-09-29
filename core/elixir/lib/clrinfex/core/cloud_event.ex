defmodule Clrinfex.Core.CloudEvent do
  @moduledoc """
  CLRINF's JSON CloudEvents 1.0 profile, independent of a broker or application domain.
  Optional flags and extension fields survive validation and JSON round trips.
  """
  alias Clrinfex.Core.{Context, Validation}

  @type t :: %{required(String.t()) => term()}
  @required ~w(id source type time tenantid correlationid causationid)

  def new(context, source, type, data, options \\ []) do
    with {:ok, context} <- Context.validate(context) do
      %{
        "specversion" => "1.0",
        "id" => Keyword.get_lazy(options, :id, &UUID.uuid4/0),
        "source" => source,
        "type" => type,
        "time" =>
          Keyword.get_lazy(options, :time, fn -> DateTime.to_iso8601(DateTime.utc_now()) end),
        "datacontenttype" => "application/json",
        "tenantid" => context["tenant_id"],
        "correlationid" => context["correlation_id"],
        "causationid" => context["causation_id"],
        "data" => data
      }
      |> put_flags(options)
      |> validate()
    end
  end

  def validate(value) when is_map(value) and not is_struct(value) do
    fields = Validation.required_text(value, @required)

    checks = [
      {"specversion", value["specversion"] == "1.0"},
      {"datacontenttype", value["datacontenttype"] == "application/json"},
      {"time", timestamp?(value["time"])},
      {"data", Validation.object?(value["data"])},
      {"is_error", not Map.has_key?(value, "is_error") or is_boolean(value["is_error"])},
      {"is_compensation",
       not Map.has_key?(value, "is_compensation") or is_boolean(value["is_compensation"])},
      {"json", Validation.json?(value)}
    ]

    invalid = for {field, false} <- checks, do: field
    Validation.result(value, Enum.uniq(fields ++ invalid), "INVALID_EVENT")
  end

  def validate(value), do: {:error, Validation.invalid("INVALID_EVENT", ["event"], value)}

  def context(event) do
    with {:ok, event} <- validate(event),
         do: Context.new(event["tenantid"], event["correlationid"], event["causationid"])
  end

  def encode(event) do
    with {:ok, event} <- validate(event), do: Jason.encode(event)
  end

  def decode(json) when is_binary(json) do
    case Jason.decode(json) do
      {:ok, event} -> validate(event)
      {:error, _} -> {:error, Validation.invalid("INVALID_EVENT", ["json"], %{})}
    end
  end

  def decode(value), do: {:error, Validation.invalid("INVALID_EVENT", ["json"], value)}

  defp timestamp?(value) when is_binary(value) do
    case DateTime.from_iso8601(value) do
      {:ok, _, _} -> true
      _ -> false
    end
  end

  defp timestamp?(_), do: false

  defp put_flags(event, options) do
    Enum.reduce([:is_error, :is_compensation], event, fn flag, acc ->
      if Keyword.has_key?(options, flag),
        do: Map.put(acc, Atom.to_string(flag), Keyword.fetch!(options, flag)),
        else: acc
    end)
  end
end
