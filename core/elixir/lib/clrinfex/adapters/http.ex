defmodule Clrinfex.Adapters.HTTP do
  @moduledoc """
  Small Plug adapter for core metadata and JSON results.

  Headers are not authentication. A production adapter must authenticate and
  authorize the tenant before invoking application code.
  """
  import Plug.Conn
  alias Clrinfex.Core.{Context, Error}

  def context(conn) do
    Context.new(
      header(conn, "x-tenant-id"),
      header(conn, "x-correlation-id"),
      header(conn, "x-causation-id")
    )
  end

  def respond(conn, result, status \\ 200)
  def respond(conn, {:ok, value}, status), do: json(conn, status, value)
  def respond(conn, {:error, error}, _status), do: json(conn, 400, error)

  def error(conn, context, code, message, status, retryable \\ false) do
    {:ok, envelope} = Error.new(context, code, message, retryable)
    json(conn, status, envelope)
  end

  def json(conn, status, value) do
    conn
    |> put_resp_content_type("application/json")
    |> send_resp(status, Jason.encode!(value))
  end

  defp header(conn, name) do
    case get_req_header(conn, name) do
      [value] -> value
      _ -> nil
    end
  end
end
