defmodule Mix.Tasks.Clrinfex.Lint do
  @moduledoc """
  Runs clrinf architectural AST linter across Elixir codebase.

  Usage:
      mix clrinfex.lint [path] [--format human|json]

  Options:
      --format, -f    Output format: 'human' (default) or 'json'
  """
  use Mix.Task
  alias Clrinfex.Linter.ArchLinter

  @shortdoc "Lints Elixir code for clrinf architectural rules (ARCH_EX_*)"

  @impl Mix.Task
  def run(args) do
    {opts, rest, _} =
      OptionParser.parse(args,
        switches: [format: :string],
        aliases: [f: :format]
      )

    format = Keyword.get(opts, :format, "human")
    path = List.first(rest) || "lib"

    violations = ArchLinter.lint_path(path)
    success = Enum.empty?(violations)

    case format do
      "json" ->
        result = %{
          success: success,
          violations: violations
        }

        IO.puts(Jason.encode!(result))

      _ ->
        if success do
          IO.puts("✔ [clrinfex] No architectural violations found in #{path}.")
        else
          Enum.each(violations, fn v ->
            IO.puts("[#{v.code}] #{v.file}:#{v.line}:#{v.col} #{v.message}")
          end)

          IO.puts(
            "\n✖ [clrinfex] Found #{length(violations)} architectural violation(s) in #{path}."
          )
        end
    end

    unless success do
      exit({:shutdown, 1})
    end
  end
end
