defmodule EcommerceMonolith.Inventory do
  @moduledoc """
  Inventory domain entity, business rules and OTP Worker.
  """

  defmodule StockItem do
    @enforce_keys [:product_id, :tenant_id, :available_quantity, :reserved_quantity]
    defstruct [:product_id, :tenant_id, :available_quantity, :reserved_quantity]
  end

  # ─── Pure Functional Rules ─────────────────────────────────────────────────

  @doc "Validates that requested quantity is greater than zero."
  def validate_quantity(qty) when is_integer(qty) do
    if qty > 0 do
      :ok
    else
      {:error, %{code: "INVALID_QUANTITY", message: "Requested quantity must be positive. Received: #{qty}"}}
    end
  end
  def validate_quantity(_), do: {:error, %{code: "INVALID_QUANTITY", message: "Quantity must be an integer"}}

  @doc "Validates that available stock is sufficient for requested reservation."
  def validate_availability(%StockItem{available_quantity: avail}, qty) do
    if avail >= qty do
      :ok
    else
      {:error, %{code: "STOCK_INSUFFICIENT", message: "Insufficient stock. Available: #{avail}, Requested: #{qty}"}}
    end
  end
  def validate_availability(nil, _qty) do
    {:error, %{code: "STOCK_INSUFFICIENT", message: "Stock item not found"}}
  end

  # ─── OTP GenServer Worker ──────────────────────────────────────────────────

  defmodule Worker do
    use GenServer
    alias EcommerceMonolith.Inventory
    alias EcommerceMonolith.Inventory.StockItem

    def start_link(opts \\ []) do
      GenServer.start_link(__MODULE__, opts, name: opts[:name] || __MODULE__)
    end

    def set_stock(server \\ __MODULE__, tenant_id, product_id, initial_qty) do
      GenServer.call(server, {:set_stock, tenant_id, product_id, initial_qty})
    end

    def reserve_stock(server \\ __MODULE__, tenant_id, product_id, qty) do
      GenServer.call(server, {:reserve_stock, tenant_id, product_id, qty})
    end

    def release_stock(server \\ __MODULE__, tenant_id, product_id, qty) do
      GenServer.call(server, {:release_stock, tenant_id, product_id, qty})
    end

    def commit_stock(server \\ __MODULE__, tenant_id, product_id, qty) do
      GenServer.call(server, {:commit_stock, tenant_id, product_id, qty})
    end

    def get_stock(server \\ __MODULE__, tenant_id, product_id) do
      GenServer.call(server, {:get_stock, tenant_id, product_id})
    end

    @impl true
    def init(_opts) do
      {:ok, %{}}
    end

    @impl true
    def handle_call({:set_stock, tenant_id, product_id, qty}, _from, state) do
      item = %StockItem{
        product_id: product_id,
        tenant_id: tenant_id,
        available_quantity: qty,
        reserved_quantity: 0
      }
      new_state = Map.put(state, {tenant_id, product_id}, item)
      {:reply, {:ok, item}, new_state}
    end

    @impl true
    def handle_call({:reserve_stock, tenant_id, product_id, qty}, _from, state) do
      item = Map.get(state, {tenant_id, product_id})

      with :ok <- Inventory.validate_quantity(qty),
           :ok <- Inventory.validate_availability(item, qty) do
        updated = %StockItem{
          item |
          available_quantity: item.available_quantity - qty,
          reserved_quantity: item.reserved_quantity + qty
        }
        new_state = Map.put(state, {tenant_id, product_id}, updated)
        {:reply, {:ok, updated}, new_state}
      else
        {:error, reason} ->
          {:reply, {:error, reason}, state}
      end
    end

    @impl true
    def handle_call({:release_stock, tenant_id, product_id, qty}, _from, state) do
      case Map.get(state, {tenant_id, product_id}) do
        %StockItem{} = item ->
          release_qty = min(item.reserved_quantity, qty)
          updated = %StockItem{
            item |
            available_quantity: item.available_quantity + release_qty,
            reserved_quantity: item.reserved_quantity - release_qty
          }
          new_state = Map.put(state, {tenant_id, product_id}, updated)
          {:reply, {:ok, updated}, new_state}

        nil ->
          {:reply, {:error, %{code: "STOCK_NOT_FOUND", message: "Stock item not found"}}, state}
      end
    end

    @impl true
    def handle_call({:commit_stock, tenant_id, product_id, qty}, _from, state) do
      case Map.get(state, {tenant_id, product_id}) do
        %StockItem{} = item ->
          commit_qty = min(item.reserved_quantity, qty)
          updated = %StockItem{
            item |
            reserved_quantity: item.reserved_quantity - commit_qty
          }
          new_state = Map.put(state, {tenant_id, product_id}, updated)
          {:reply, {:ok, updated}, new_state}

        nil ->
          {:reply, {:error, %{code: "STOCK_NOT_FOUND", message: "Stock item not found"}}, state}
      end
    end

    @impl true
    def handle_call({:get_stock, tenant_id, product_id}, _from, state) do
      {:reply, {:ok, Map.get(state, {tenant_id, product_id})}, state}
    end
  end
end
