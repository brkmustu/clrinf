defmodule MinimalService.Demo do
  alias MinimalService.{Dispatcher, Greet, GreetingModule, RequestContext}

  def run do
    context = %RequestContext{correlation_id: "demo-request"}

    case Dispatcher.dispatch(GreetingModule, %Greet{name: "World"}, context) do
      {:ok, greeting} ->
        IO.puts("#{greeting.message} [#{greeting.correlation_id}]")

      {:error, error} ->
        raise "#{error.code}: #{error.message}"
    end
  end
end
