defmodule Clrinfex.Adapters.JSONBody do
  @moduledoc false
  @behaviour Plug
  alias Clrinfex.Adapters.HTTP
  alias Clrinfex.Core.Validation

  @impl true
  def init(_options) do
    Plug.Parsers.init(
      parsers: [:json],
      pass: [],
      json_decoder: Jason,
      length: Clrinfex.NatsBridge.max_event_bytes(),
      read_length: Clrinfex.NatsBridge.max_event_bytes(),
      body_reader: {__MODULE__, :read_body, []}
    )
  end

  def read_body(conn, options) do
    case Plug.Conn.read_body(conn, options) do
      {:ok, body, conn} ->
        if byte_size(body) <= Clrinfex.NatsBridge.max_event_bytes(),
          do: {:ok, body, conn},
          else: {:more, body, conn}

      other ->
        other
    end
  end

  @impl true
  def call(conn, options) do
    Plug.Parsers.call(conn, options)
  rescue
    error in [
      Plug.Parsers.ParseError,
      Plug.Parsers.RequestTooLargeError,
      Plug.Parsers.UnsupportedMediaTypeError
    ] ->
      context =
        case HTTP.context(conn) do
          {:ok, context} -> context
          {:error, context_error} -> context_error
        end

      code = if error.plug_status == 413, do: "PAYLOAD_TOO_LARGE", else: "INVALID_REQUEST"

      conn
      |> HTTP.json(error.plug_status, Validation.invalid(code, ["body"], context))
      |> Plug.Conn.halt()
  end
end
