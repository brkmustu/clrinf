defmodule MinimalService.RequestContext do
  @enforce_keys [:correlation_id]
  defstruct [:correlation_id]

  @type t :: %__MODULE__{correlation_id: String.t()}
end

defmodule MinimalService.ApplicationError do
  @enforce_keys [:code, :message]
  defstruct [:code, :message]

  @type t :: %__MODULE__{code: String.t(), message: String.t()}
end

defmodule MinimalService.ApplicationModule do
  alias MinimalService.{ApplicationError, RequestContext}

  @type result(response) :: {:ok, response} | {:error, ApplicationError.t()}
  @callback handle(term(), RequestContext.t()) :: result(term())
end

defmodule MinimalService.Dispatcher do
  alias MinimalService.{ApplicationModule, RequestContext}

  @spec dispatch(module(), term(), RequestContext.t()) :: ApplicationModule.result(term())
  def dispatch(module, request, %RequestContext{} = context) do
    module.handle(request, context)
  end
end
