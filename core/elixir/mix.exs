defmodule Clrinfex.MixProject do
  use Mix.Project

  def project do
    [
      app: :clrinfex,
      version: "0.1.0",
      elixirc_paths: elixirc_paths(Mix.env()),
      test_paths: ["test", "examples/ecommerce_monolith/test"],
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  defp elixirc_paths(_), do: ["lib", "examples/ecommerce_monolith/lib"]

  def application do
    [
      extra_applications: [:logger, :crypto],
      mod: {Clrinfex.Application, []}
    ]
  end

  defp deps do
    [
      {:plug_cowboy, "~> 2.7"},
      {:jason, "~> 1.4"},
      {:gnat, "~> 1.6"},
      {:elixir_uuid, "~> 1.2"}
    ]
  end
end
