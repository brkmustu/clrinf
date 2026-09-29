defmodule Mix.Tasks.Clrinfex.LintTest do
  use ExUnit.Case, async: true
  import ExUnit.CaptureIO

  test "runs mix clrinfex.lint on valid file and outputs json" do
    output =
      capture_io(fn ->
        Mix.Tasks.Clrinfex.Lint.run(["lib/clrinfex/linter/arch_linter.ex", "--format", "json"])
      end)

    assert output =~ ~s("success":true)
    assert output =~ ~s("violations":[])
  end

  test "runs mix clrinfex.lint with human format" do
    output =
      capture_io(fn ->
        Mix.Tasks.Clrinfex.Lint.run(["lib/clrinfex/linter/arch_linter.ex", "--format", "human"])
      end)

    assert output =~ "No architectural violations found"
  end
end
