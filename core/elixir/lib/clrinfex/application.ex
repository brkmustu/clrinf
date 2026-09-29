defmodule Clrinfex.Application do
  use Application
  require Logger

  @impl true
  def start(_type, _args) do
    children =
      if Application.get_env(:clrinfex, :start_streaming, true),
        do: streaming_children(),
        else: []

    opts = [strategy: :one_for_one, name: Clrinfex.Supervisor]
    Supervisor.start_link(children, opts)
  end

  defp streaming_children do
    port = Application.get_env(:clrinfex, :port, 4000)

    Logger.info("[clrinfex] Streaming & Presence Service starting on port #{port}")

    [
      Clrinfex.Presence,
      Clrinfex.NatsBridge,
      {Plug.Cowboy,
       scheme: :http,
       plug: Clrinfex.Router,
       options: [
         port: port,
         dispatch: dispatch_config()
       ]}
    ]
  end

  defp dispatch_config do
    [
      {:_,
       [
         {"/stream/ws", Clrinfex.StreamingSocket, []},
         {:_, Plug.Cowboy.Handler, {Clrinfex.Router, []}}
       ]}
    ]
  end
end
