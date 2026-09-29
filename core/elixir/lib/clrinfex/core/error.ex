defmodule Clrinfex.Core.Error do
  @moduledoc "Portable failure envelope; errors are values, not transport exceptions."
  alias Clrinfex.Core.{Context, Validation}

  @type t :: %{required(String.t()) => term()}

  def new(context, code, message, retryable \\ false, details \\ nil) do
    with {:ok, context} <- Context.validate(context) do
      envelope = %{
        "error_code" => code,
        "message" => message,
        "correlation_id" => context["correlation_id"],
        "tenant_id" => context["tenant_id"],
        "retryable" => retryable
      }

      envelope = if is_nil(details), do: envelope, else: Map.put(envelope, "details", details)
      validate(envelope)
    end
  end

  def validate(value) when is_map(value) and not is_struct(value) do
    fields =
      Validation.required_text(value, ~w(error_code message correlation_id tenant_id))

    fields = if is_boolean(value["retryable"]), do: fields, else: ["retryable" | fields]
    fields = if Validation.json?(value), do: fields, else: ["json" | fields]
    Validation.result(value, fields, "INVALID_ERROR")
  end

  def validate(value), do: {:error, Validation.invalid("INVALID_ERROR", ["error"], value)}
end
