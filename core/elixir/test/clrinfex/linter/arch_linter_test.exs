defmodule Clrinfex.Linter.ArchLinterTest do
  use ExUnit.Case, async: true
  alias Clrinfex.Linter.ArchLinter

  describe "ARCH_EX_001: Explicit OTP Callback Contracts" do
    test "passes when all GenServer callbacks have @impl true" do
      code = """
      defmodule MyApp.ValidWorker do
        use GenServer

        @impl true
        def init(state), do: {:ok, state}

        @impl true
        def handle_call(:ping, _from, state), do: {:reply, :pong, state}
      end
      """

      violations = ArchLinter.lint_string(code)
      assert violations == []
    end

    test "fails when GenServer callback lacks @impl true" do
      code = """
      defmodule MyApp.InvalidWorker do
        use GenServer

        def init(state), do: {:ok, state}

        def handle_call(:ping, _from, state), do: {:reply, :pong, state}
      end
      """

      violations = ArchLinter.lint_string(code)
      codes = Enum.map(violations, & &1.code)
      assert "ARCH_EX_001" in codes
      assert length(violations) == 2
    end
  end

  describe "ARCH_EX_002: Pure Domain Result Monad" do
    test "passes when domain functions return tagged result tuples" do
      code = """
      defmodule MyApp.Order.Handler do
        def execute(params) do
          {:ok, params}
        end
      end
      """

      violations = ArchLinter.lint_string(code)
      assert violations == []
    end

    test "fails when pure domain handler calls raise or throw" do
      code = """
      defmodule MyApp.Order.Handler do
        def execute(params) do
          if params == nil do
            raise "Invalid params"
          else
            throw(:bad_state)
          end
        end
      end
      """

      violations = ArchLinter.lint_string(code)
      codes = Enum.map(violations, & &1.code)
      assert "ARCH_EX_002" in codes
      assert length(violations) == 2
    end
  end

  describe "ARCH_EX_003: Supervised Concurrency" do
    test "fails when naked spawn or Task.start is used" do
      code = """
      defmodule MyApp.Order.Handler do
        def run_async(data) do
          spawn(fn -> data end)
          Task.start(fn -> data end)
        end
      end
      """

      violations = ArchLinter.lint_string(code)
      codes = Enum.map(violations, & &1.code)
      assert "ARCH_EX_003" in codes
      assert length(violations) == 2
    end
  end

  describe "ARCH_EX_004: Pure Domain Side-Effect Isolation" do
    test "fails when IO or sleep is invoked in domain module" do
      code = """
      defmodule MyApp.Rules.DiscountRule do
        def check(subject) do
          IO.puts("checking...")
          Process.sleep(100)
          :ok
        end
      end
      """

      violations = ArchLinter.lint_string(code)
      codes = Enum.map(violations, & &1.code)
      assert "ARCH_EX_004" in codes
      assert length(violations) == 2
    end
  end

  describe "Syntax error handling" do
    test "reports SYNTAX_ERROR on malformed Elixir code" do
      code = """
      defmodule Broken do
        def unclosed_def(
      """

      violations = ArchLinter.lint_string(code)
      assert length(violations) == 1
      assert hd(violations).code == "SYNTAX_ERROR"
    end
  end
end
