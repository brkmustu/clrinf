defmodule EcommerceMonolith.Catalog do
  @moduledoc """
  Catalog domain entity, rules and functional handlers.
  """

  defmodule Product do
    @enforce_keys [:id, :tenant_id, :sku, :name, :price, :status]
    defstruct [:id, :tenant_id, :sku, :name, :price, :status]
  end

  # ─── Pure Functional Rules ─────────────────────────────────────────────────

  @doc "Validates that product price is greater than zero."
  def validate_price(price) when is_number(price) do
    if price > 0 do
      :ok
    else
      {:error, %{code: "INVALID_PRODUCT_PRICE", message: "Product price must be positive. Received: #{price}"}}
    end
  end
  def validate_price(_), do: {:error, %{code: "INVALID_PRODUCT_PRICE", message: "Price must be a number"}}

  @doc "Validates that SKU does not already exist in the given state for the tenant."
  def validate_sku_unique(sku, tenant_id, state) when is_binary(sku) and is_binary(tenant_id) and is_map(state) do
    exists? =
      Enum.any?(state, fn {_k, %Product{} = p} ->
        p.tenant_id == tenant_id and String.downcase(p.sku) == String.downcase(sku)
      end)

    if exists? do
      {:error, %{code: "DUPLICATE_SKU", message: "Product with SKU '#{sku}' already exists in tenant '#{tenant_id}'."}}
    else
      :ok
    end
  end

  # ─── Pure Functional Pipeline ──────────────────────────────────────────────

  def build_product(params, state) do
    with :ok <- validate_price(params.price),
         :ok <- validate_sku_unique(params.sku, params.tenant_id, state) do
      id = "prod_" <> (:crypto.strong_rand_bytes(4) |> Base.encode16(case: :lower))
      product = %Product{
        id: id,
        tenant_id: params.tenant_id,
        sku: params.sku,
        name: params.name,
        price: params.price,
        status: "Active"
      }
      {:ok, product}
    end
  end

  # ─── OTP GenServer Worker ──────────────────────────────────────────────────

  defmodule Worker do
    use GenServer
    alias EcommerceMonolith.Catalog

    def start_link(opts \\ []) do
      GenServer.start_link(__MODULE__, opts, name: opts[:name] || __MODULE__)
    end

    def create_product(server \\ __MODULE__, params) do
      GenServer.call(server, {:create_product, params})
    end

    def get_product(server \\ __MODULE__, tenant_id, product_id) do
      GenServer.call(server, {:get_product, tenant_id, product_id})
    end

    @impl true
    def init(_opts) do
      {:ok, %{}}
    end

    @impl true
    def handle_call({:create_product, params}, _from, state) do
      case Catalog.build_product(params, state) do
        {:ok, product} ->
          new_state = Map.put(state, {product.tenant_id, product.id}, product)
          {:reply, {:ok, product}, new_state}

        {:error, reason} ->
          {:reply, {:error, reason}, state}
      end
    end

    @impl true
    def handle_call({:get_product, tenant_id, product_id}, _from, state) do
      product = Map.get(state, {tenant_id, product_id})
      {:reply, {:ok, product}, state}
    end
  end
end
