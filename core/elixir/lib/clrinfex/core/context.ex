defmodule Clrinfex.Core.Context do
  @moduledoc """
  Explicit request metadata shared across in-process and transport boundaries.

  Values use string keys to match the polyglot wire contract. Validation does
  not authenticate tenants: adapters must supply metadata from trusted identity.
  """
  alias Clrinfex.Core.Validation

  @type t :: %{
          required(String.t()) => String.t()
        }

  def new(tenant_id, correlation_id, causation_id) do
    validate(%{
      "tenant_id" => tenant_id,
      "correlation_id" => correlation_id,
      "causation_id" => causation_id
    })
  end

  def validate(value) when is_map(value) and not is_struct(value) do
    fields = Validation.required_text(value, ~w(tenant_id correlation_id causation_id))
    fields = if Validation.json?(value), do: fields, else: ["json" | fields]
    Validation.result(value, fields, "INVALID_CONTEXT")
  end

  def validate(value),
    do: {:error, Validation.invalid("INVALID_CONTEXT", ["context"], value)}

  @doc "Preserves tenant/correlation while setting the next operation's cause."
  def child(context, cause) do
    with {:ok, context} <- validate(context),
         do: context |> Map.put("causation_id", cause) |> validate()
  end
end
