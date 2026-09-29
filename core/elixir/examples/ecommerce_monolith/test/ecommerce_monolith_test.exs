defmodule EcommerceMonolithTest do
  use ExUnit.Case, async: false

  alias EcommerceMonolith.Catalog
  alias EcommerceMonolith.Inventory
  alias EcommerceMonolith.Orders
  alias EcommerceMonolith.Orders.OrderItem

  setup do
    start_supervised!({Catalog.Worker, [name: :test_catalog_worker]})
    start_supervised!({Inventory.Worker, [name: :test_inventory_worker]})
    start_supervised!({Orders.Worker, [name: :test_orders_worker]})
    :ok
  end

  test "creates product successfully with positive price" do
    params = %{
      tenant_id: "tenant_acme",
      sku: "SKU-KEYBOARD",
      name: "Mechanical Keyboard",
      price: 120.0
    }

    assert {:ok, product} = Catalog.Worker.create_product(:test_catalog_worker, params)
    assert product.sku == "SKU-KEYBOARD"
    assert product.price == 120.0
    assert product.status == "Active"
  end

  test "fails to create product when price is zero or negative" do
    params = %{
      tenant_id: "tenant_acme",
      sku: "SKU-FREE",
      name: "Zero Price Item",
      price: 0.0
    }

    assert {:error, error} = Catalog.Worker.create_product(:test_catalog_worker, params)
    assert error.code == "INVALID_PRODUCT_PRICE"
  end

  test "fails to create product when SKU is duplicate in same tenant" do
    params1 = %{
      tenant_id: "tenant_acme",
      sku: "SKU-MOUSE",
      name: "Gaming Mouse",
      price: 60.0
    }
    assert {:ok, _} = Catalog.Worker.create_product(:test_catalog_worker, params1)

    params2 = %{
      tenant_id: "tenant_acme",
      sku: "SKU-MOUSE",
      name: "Office Mouse",
      price: 40.0
    }
    assert {:error, error} = Catalog.Worker.create_product(:test_catalog_worker, params2)
    assert error.code == "DUPLICATE_SKU"
  end

  test "reserves stock successfully when available" do
    assert {:ok, _} = Inventory.Worker.set_stock(:test_inventory_worker, "tenant_acme", "prod_1", 10)

    assert {:ok, updated} = Inventory.Worker.reserve_stock(:test_inventory_worker, "tenant_acme", "prod_1", 3)
    assert updated.available_quantity == 7
    assert updated.reserved_quantity == 3
  end

  test "fails to reserve stock when requested quantity exceeds available stock" do
    assert {:ok, _} = Inventory.Worker.set_stock(:test_inventory_worker, "tenant_acme", "prod_2", 2)

    assert {:error, error} = Inventory.Worker.reserve_stock(:test_inventory_worker, "tenant_acme", "prod_2", 5)
    assert error.code == "STOCK_INSUFFICIENT"
  end

  test "creates order successfully with valid items and total" do
    params = %{
      tenant_id: "tenant_acme",
      customer_id: "cust_1",
      items: [%OrderItem{product_id: "prod_1", quantity: 1, unit_price: 120.0}]
    }

    assert {:ok, order} = Orders.Worker.create_order(:test_orders_worker, params)
    assert order.status == "Pending"
    assert order.total_amount == 120.0
  end

  test "fails to create order when total amount is below minimum threshold" do
    params = %{
      tenant_id: "tenant_acme",
      customer_id: "cust_1",
      items: [%OrderItem{product_id: "prod_sticker", quantity: 1, unit_price: 5.0}]
    }

    assert {:error, error} = Orders.Worker.create_order(:test_orders_worker, params)
    assert error.code == "MINIMUM_ORDER_AMOUNT_NOT_MET"
  end

  test "fails to create order when items list is empty" do
    params = %{
      tenant_id: "tenant_acme",
      customer_id: "cust_1",
      items: []
    }

    assert {:error, error} = Orders.Worker.create_order(:test_orders_worker, params)
    assert error.code == "EMPTY_ORDER_ITEMS"
  end

  test "executes order cancellation and stock release compensating lifecycle" do
    # 1. Set stock
    assert {:ok, _} = Inventory.Worker.set_stock(:test_inventory_worker, "tenant_acme", "prod_headset", 10)

    # 2. Reserve 2 items
    assert {:ok, res} = Inventory.Worker.reserve_stock(:test_inventory_worker, "tenant_acme", "prod_headset", 2)
    assert res.available_quantity == 8
    assert res.reserved_quantity == 2

    # 3. Create order
    order_params = %{
      tenant_id: "tenant_acme",
      customer_id: "cust_1",
      items: [%OrderItem{product_id: "prod_headset", quantity: 2, unit_price: 50.0}]
    }
    assert {:ok, order} = Orders.Worker.create_order(:test_orders_worker, order_params)
    assert order.status == "Pending"

    # 4. Cancel order
    assert {:ok, cancelled} = Orders.Worker.cancel_order(:test_orders_worker, "tenant_acme", order.id)
    assert cancelled.status == "Cancelled"

    # 5. Compensating action: Release reserved stock
    assert {:ok, released} = Inventory.Worker.release_stock(:test_inventory_worker, "tenant_acme", "prod_headset", 2)
    assert released.available_quantity == 10
    assert released.reserved_quantity == 0
  end

  test "verifies tenant isolation in stock queries" do
    assert {:ok, _} = Inventory.Worker.set_stock(:test_inventory_worker, "tenant_alpha", "prod_shared", 50)

    # Tenant beta queries prod_shared -> must be nil
    assert {:ok, nil} = Inventory.Worker.get_stock(:test_inventory_worker, "tenant_beta", "prod_shared")
  end
end
