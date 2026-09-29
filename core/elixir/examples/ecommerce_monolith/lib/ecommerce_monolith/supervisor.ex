defmodule EcommerceMonolith.Supervisor do
  @moduledoc """
  Supervision Tree for the Ecommerce Monolith OTP workers.
  """
  use Supervisor

  def start_link(opts \\ []) do
    Supervisor.start_link(__MODULE__, opts, name: opts[:name] || __MODULE__)
  end

  @impl true
  def init(_opts) do
    children = [
      {EcommerceMonolith.Catalog.Worker, [name: EcommerceMonolith.Catalog.Worker]},
      {EcommerceMonolith.Inventory.Worker, [name: EcommerceMonolith.Inventory.Worker]},
      {EcommerceMonolith.Orders.Worker, [name: EcommerceMonolith.Orders.Worker]}
    ]

    Supervisor.init(children, strategy: :one_for_one)
  end
end
