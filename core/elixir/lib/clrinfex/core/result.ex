defmodule Clrinfex.Core.Result do
  @moduledoc "Idiomatic tagged tuples for composition; failures short-circuit without exceptions."
  @type t(value) :: {:ok, value} | {:error, Clrinfex.Core.Error.t()}

  def ok(value), do: {:ok, value}

  def error(envelope) do
    case Clrinfex.Core.Error.validate(envelope) do
      {:ok, envelope} -> {:error, envelope}
      {:error, invalid} -> {:error, invalid}
    end
  end

  def map({:ok, value}, fun) when is_function(fun, 1), do: {:ok, fun.(value)}
  def map({:error, _} = error, _fun), do: error
  def bind({:ok, value}, fun) when is_function(fun, 1), do: fun.(value)
  def bind({:error, _} = error, _fun), do: error
end
