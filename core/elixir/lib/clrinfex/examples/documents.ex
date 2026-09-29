defmodule Clrinfex.Examples.Documents do
  @moduledoc "Pure example application module, called unchanged by HTTP or in-process code."
  alias Clrinfex.Core.{CloudEvent, Context, Error, Validation}

  def approve(context, command) do
    with {:ok, context} <- Context.validate(context) do
      if Validation.object?(command) and Validation.text?(command["document_id"]) do
        CloudEvent.new(
          context,
          "clrinf/document-example",
          "com.clrinf.documents.DocumentApproved.v1",
          %{
            "document_id" => command["document_id"],
            "status" => "approved"
          }
        )
      else
        {:ok, error} = Error.new(context, "INVALID_DOCUMENT", "document_id is required")
        {:error, error}
      end
    end
  end
end
