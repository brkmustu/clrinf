defmodule MinimalService.GreetingTest do
  use ExUnit.Case, async: true

  alias MinimalService.{
    ApplicationError,
    Dispatcher,
    Greet,
    Greeting,
    GreetingModule,
    RequestContext
  }

  test "dispatches and propagates context" do
    context = %RequestContext{correlation_id: "test-request"}

    assert {:ok, %Greeting{message: "Hello, Ada!", correlation_id: "test-request"}} =
             Dispatcher.dispatch(GreetingModule, %Greet{name: " Ada "}, context)
  end

  test "rejects blank names" do
    context = %RequestContext{correlation_id: "test-request"}

    for name <- ["", " \t\n"] do
      assert {:error, %ApplicationError{code: "validation.name_required"}} =
               Dispatcher.dispatch(GreetingModule, %Greet{name: name}, context)
    end
  end
end
