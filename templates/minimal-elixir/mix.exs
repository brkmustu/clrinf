defmodule MinimalService.MixProject do
  use Mix.Project

  def project do
    [
      app: :minimal_service,
      version: "0.1.0",
      elixir: ">= 1.14.0",
      deps: []
    ]
  end

  def application, do: []
end
