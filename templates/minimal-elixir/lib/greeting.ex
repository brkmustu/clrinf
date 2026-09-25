defmodule MinimalService.Greet do
  @enforce_keys [:name]
  defstruct [:name]
end

defmodule MinimalService.Greeting do
  @enforce_keys [:message, :correlation_id]
  defstruct [:message, :correlation_id]
end

defmodule MinimalService.GreetingModule do
  @behaviour MinimalService.ApplicationModule

  alias MinimalService.{ApplicationError, Greet, Greeting, RequestContext}

  @impl true
  def handle(%Greet{name: name}, %RequestContext{} = context) when is_binary(name) do
    case String.trim(name) do
      "" ->
        {:error,
         %ApplicationError{
           code: "validation.name_required",
           message: "Name must not be blank."
         }}

      trimmed ->
        {:ok,
         %Greeting{
           message: "Hello, #{trimmed}!",
           correlation_id: context.correlation_id
         }}
    end
  end
end
