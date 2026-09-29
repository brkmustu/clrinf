defmodule EcommerceMonolith.Orders do
  @moduledoc """
  Orders domain entity, business rules and OTP Worker.
  """

  defmodule OrderItem do
    @enforce_keys [:product_id, :quantity, :unit_price]
    defstruct [:product_id, :quantity, :unit_price]
  end

  defmodule Order do
    @enforce_keys [:id, :tenant_id, :customer_id, :items, :total_amount, :status]
    defstruct [:id, :tenant_id, :customer_id, :items, :total_amount, :status]
  end

  @min_order_amount 25.0

  # ─── Pure Functional Rules ─────────────────────────────────────────────────

  @doc "Validates that order contains at least one item."
  def validate_items_not_empty(items) when is_list(items) do
    if length(items) > 0 do
      :ok
    else
      {:error, %{code: "EMPTY_ORDER_ITEMS", message: "Order must contain at least one item."}}
    end
  end
  def validate_items_not_empty(_), do: {:error, %{code: "EMPTY_ORDER_ITEMS", message: "Items must be a list."}}

  @doc "Validates that total order amount is above minimum threshold ($25.00)."
  def validate_minimum_amount(total) when is_number(total) do
    if total >= @min_order_amount do
      :ok
    else
      {:error, %{code: "MINIMUM_ORDER_AMOUNT_NOT_MET", message: "Order total $#{total} is below minimum required threshold $25.00."}}
    end
  end

  @doc "Validates that order is cancellable (cannot cancel completed order)."
  def validate_cancellable(%Order{status: "Completed"}) do
    {:error, %{code: "INVALID_ORDER_STATE", message: "Cannot cancel completed order."}}
  end
  def validate_cancellable(%Order{}), do: :ok
  def validate_cancellable(nil), do: {:error, %{code: "ORDER_NOT_FOUND", message: "Order not found."}}

  # ─── Pure Functional Pipeline ──────────────────────────────────────────────

  def build_order(params) do
    items = params[:items] || []
    total = Enum.reduce(items, 0.0, fn item, acc -> acc + item.quantity * item.unit_price end)

    with :ok <- validate_items_not_empty(items),
         :ok <- validate_minimum_amount(total) do
      id = "ord_" <> (:crypto.strong_rand_bytes(4) |> Base.encode16(case: :lower))
      order = %Order{
        id: id,
        tenant_id: params[:tenant_id],
        customer_id: params[:customer_id],
        items: items,
        total_amount: total,
        status: "Pending"
      }
      {:ok, order}
    end
  end

  # ─── OTP GenServer Worker ──────────────────────────────────────────────────

  defmodule Worker do
    use GenServer
    alias EcommerceMonolith.Orders
    alias EcommerceMonolith.Orders.Order

    def start_link(opts \\ []) do
      GenServer.start_link(__MODULE__, opts, name: opts[:name] || __MODULE__)
    end

    def create_order(server \\ __MODULE__, params) do
      GenServer.call(server, {:create_order, params})
    end

    def cancel_order(server \\ __MODULE__, tenant_id, order_id) do
      GenServer.call(server, {:cancel_order, tenant_id, order_id})
    end

    def get_order(server \\ __MODULE__, tenant_id, order_id) do
      GenServer.call(server, {:get_order, tenant_id, order_id})
    end

    @impl true
    def init(_opts) do
      {:ok, %{}}
    end

    @impl true
    def handle_call({:create_order, params}, _from, state) do
      case Orders.build_order(params) do
        {:ok, order} ->
          new_state = Map.put(state, {order.tenant_id, order.id}, order)
          {:reply, {:ok, order}, new_state}

        {:error, reason} ->
          {:reply, {:error, reason}, state}
      end
    end

    @impl true
    def handle_call({:cancel_order, tenant_id, order_id}, _from, state) do
      case Map.get(state, {tenant_id, order_id}) do
        %Order{} = order ->
          case Orders.validate_cancellable(order) do
            :ok ->
              updated = %Order{order | status: "Cancelled"}
              new_state = Map.put(state, {tenant_id, order_id}, updated)
              {:reply, {:ok, updated}, new_state}

            {:error, reason} ->
              {:reply, {:error, reason}, state}
          end

        nil ->
          {:reply, {:error, %{code: "ORDER_NOT_FOUND", message: "Order not found."}}, state}
      end
    end

    @impl true
    def handle_call({:get_order, tenant_id, order_id}, _from, state) do
      {:reply, {:ok, Map.get(state, {tenant_id, order_id})}, state}
    end
  end
end
