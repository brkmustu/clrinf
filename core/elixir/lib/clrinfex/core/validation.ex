defmodule Clrinfex.Core.Validation do
  @moduledoc false

  def text?(value), do: is_binary(value) and String.valid?(value) and String.trim(value) != ""
  def object?(value), do: is_map(value) and not is_struct(value)

  def json?(value) when is_map(value) and not is_struct(value),
    do:
      Enum.all?(value, fn {key, item} -> is_binary(key) and String.valid?(key) and json?(item) end)

  def json?(value) when is_list(value), do: Enum.all?(value, &json?/1)
  def json?(value) when is_binary(value), do: String.valid?(value)
  def json?(value), do: is_nil(value) or is_boolean(value) or is_number(value)

  def required_text(value, fields) do
    Enum.filter(fields, fn field -> not text?(Map.get(value, field)) end)
  end

  def result(value, [], _code), do: {:ok, value}
  def result(value, fields, code), do: {:error, invalid(code, fields, value)}

  def invalid(code, fields, value) do
    value = if object?(value), do: value, else: %{}

    %{
      "error_code" => code,
      "message" => "Invalid contract fields: #{Enum.join(fields, ", ")}",
      "correlation_id" => identifier(value, "correlation_id", "correlationid"),
      "tenant_id" => identifier(value, "tenant_id", "tenantid"),
      "retryable" => false,
      "details" => %{"fields" => fields}
    }
  end

  defp identifier(value, key, extension) do
    id = Map.get(value, key, Map.get(value, extension))
    if text?(id), do: id, else: "unknown"
  end
end
