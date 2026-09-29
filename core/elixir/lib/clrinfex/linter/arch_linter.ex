defmodule Clrinfex.Linter.ArchLinter do
  @moduledoc """
  Architectural AST linter for Elixir adhering to clrinf constitution and BEAM/OTP principles.

  Rules:
  - ARCH_EX_001: Explicit OTP Callback Contracts (@impl true on GenServer callbacks)
  - ARCH_EX_002: Pure Domain Result Monad (no raise/throw in pure domain/handlers; use {:ok, _} | {:error, _})
  - ARCH_EX_003: Supervised Concurrency (no naked spawn or Task.start without supervisor)
  - ARCH_EX_004: Pure Domain Side-Effect Isolation (no IO.puts, Process.sleep, System.cmd in pure domain)
  """

  @genserver_callbacks ~w(init handle_call handle_cast handle_info terminate code_change format_status)a

  defmodule Violation do
    @derive Jason.Encoder
    defstruct [:code, :file, :line, :col, :message]
  end

  @doc """
  Lints a path (file or directory) and returns a list of violations.
  """
  def lint_path(path) do
    path = Path.expand(path)

    files =
      cond do
        File.regular?(path) ->
          if String.ends_with?(path, [".ex", ".exs"]), do: [path], else: []

        File.dir?(path) ->
          find_source_files(path)

        true ->
          []
      end

    violations =
      Enum.flat_map(files, fn file ->
        case File.read(file) do
          {:ok, content} -> lint_string(content, file)
          {:error, _reason} -> []
        end
      end)

    violations
  end

  @doc """
  Lints an Elixir source string.
  """
  def lint_string(source, file \\ "nofile") do
    case Code.string_to_quoted(source, file: file, columns: true) do
      {:ok, ast} ->
        check_ast(ast, file)

      {:error, {meta, message, _token}} ->
        line = Keyword.get(meta, :line, 1)
        col = Keyword.get(meta, :column, 1)

        [
          %Violation{
            code: "SYNTAX_ERROR",
            file: file,
            line: line,
            col: col,
            message: "Syntax error: #{message}"
          }
        ]
    end
  end

  defp find_source_files(dir) do
    dir
    |> File.ls!()
    |> Enum.reject(&(&1 in ~w(_build deps .git node_modules)))
    |> Enum.flat_map(fn entry ->
      sub = Path.join(dir, entry)

      cond do
        File.dir?(sub) -> find_source_files(sub)
        String.ends_with?(entry, [".ex", ".exs"]) -> [sub]
        true -> []
      end
    end)
  end

  defp check_ast(ast, file) do
    # Walk modules
    modules = extract_modules(ast)

    Enum.flat_map(modules, fn {mod_name, body, _meta} ->
      is_domain_mod = domain_module?(mod_name, file)
      is_genserver = genserver_module?(body)

      v1 = if is_genserver, do: check_arch_ex_001(body, file), else: []
      v2 = if is_domain_mod, do: check_arch_ex_002(body, file), else: []
      v3 = check_arch_ex_003(body, file, is_domain_mod or is_genserver)
      v4 = if is_domain_mod, do: check_arch_ex_004(body, file), else: []

      v1 ++ v2 ++ v3 ++ v4
    end)
  end

  defp extract_modules(ast) do
    {_, modules} =
      Macro.prewalk(ast, [], fn
        {:defmodule, meta, [{:__aliases__, _, parts}, [do: body]]}, acc ->
          mod_name = Enum.join(parts, ".")
          {{:defmodule, meta, []}, [{mod_name, body, meta} | acc]}

        other, acc ->
          {other, acc}
      end)

    Enum.reverse(modules)
  end

  defp domain_module?(mod_name, file) do
    String.contains?(mod_name, [".Handler", ".Rules.", ".Domain.", ".Entity.", ".Model."]) or
      String.contains?(file, ["/rules/", "/domain/", "/handlers/"])
  end

  defp genserver_module?(body) do
    found =
      Macro.prewalk(body, false, fn
        {:use, _, [{:__aliases__, _, [:GenServer]} | _]}, _acc -> {nil, true}
        {:@, _, [{:behaviour, _, [{:__aliases__, _, [:GenServer]}]}]}, _acc -> {nil, true}
        other, acc -> {other, acc}
      end)

    elem(found, 1)
  end

  # ARCH_EX_001: GenServer callbacks must have @impl true or @impl GenServer
  defp check_arch_ex_001(body, file) do
    statements = extract_statements(body)
    check_callbacks_impl(statements, {false, nil}, file, [])
  end

  defp extract_statements({:__block__, _, stmts}), do: stmts
  defp extract_statements(stmt) when is_tuple(stmt), do: [stmt]
  defp extract_statements(_), do: []

  defp check_callbacks_impl([], _state, _file, acc), do: Enum.reverse(acc)

  defp check_callbacks_impl([stmt | rest], {has_impl, active_callback}, file, acc) do
    case stmt do
      {:@, _, [{:impl, _, _}]} ->
        check_callbacks_impl(rest, {true, nil}, file, acc)

      {:def, meta, [head | _]} ->
        {name, args} =
          case head do
            {:when, _, [{n, _, a} | _]} when is_atom(n) -> {n, a}
            {n, _, a} when is_atom(n) -> {n, a}
            _ -> {nil, nil}
          end

        if name != nil do
          arity = if is_list(args), do: length(args), else: 0

          cond do
            name in @genserver_callbacks and has_impl ->
              # First clause of a callback decorated with @impl
              check_callbacks_impl(rest, {false, name}, file, acc)

            name in @genserver_callbacks and active_callback == name ->
              # Subsequent clause of the same callback
              check_callbacks_impl(rest, {false, name}, file, acc)

            name in @genserver_callbacks ->
              # Callback missing @impl annotation
              line = Keyword.get(meta, :line, 1)
              col = Keyword.get(meta, :column, 1)

              v = %Violation{
                code: "ARCH_EX_001",
                file: file,
                line: line,
                col: col,
                message: "GenServer callback #{name}/#{arity} must specify @impl true"
              }

              check_callbacks_impl(rest, {false, nil}, file, [v | acc])

            true ->
              # Non-callback function, reset state
              check_callbacks_impl(rest, {false, nil}, file, acc)
          end
        else
          check_callbacks_impl(rest, {false, nil}, file, acc)
        end

      _other ->
        # Preserve annotation state through comments or intermediate expressions
        check_callbacks_impl(rest, {has_impl, active_callback}, file, acc)
    end
  end

  # ARCH_EX_002: Pure domain functions must not raise/throw; return tagged result tuples
  defp check_arch_ex_002(body, file) do
    {_, violations} =
      Macro.prewalk(body, [], fn
        {:raise, meta, _} = node, acc ->
          line = Keyword.get(meta, :line, 1)
          col = Keyword.get(meta, :column, 1)

          v = %Violation{
            code: "ARCH_EX_002",
            file: file,
            line: line,
            col: col,
            message:
              "Pure domain functions must return {:ok, _} or {:error, _} result tuples; 'raise' is forbidden"
          }

          {node, [v | acc]}

        {:throw, meta, _} = node, acc ->
          line = Keyword.get(meta, :line, 1)
          col = Keyword.get(meta, :column, 1)

          v = %Violation{
            code: "ARCH_EX_002",
            file: file,
            line: line,
            col: col,
            message:
              "Pure domain functions must return {:ok, _} or {:error, _} result tuples; 'throw' is forbidden"
          }

          {node, [v | acc]}

        other, acc ->
          {other, acc}
      end)

    Enum.reverse(violations)
  end

  # ARCH_EX_003: Supervised Concurrency - no raw spawn or un-supervised Task.start in domain/handlers/workers
  defp check_arch_ex_003(body, file, should_check) do
    if not should_check do
      []
    else
      {_, violations} =
        Macro.prewalk(body, [], fn
          {:spawn, meta, _} = node, acc ->
            {node, [naked_spawn_violation("spawn", meta, file) | acc]}

          {:spawn_link, meta, _} = node, acc ->
            {node, [naked_spawn_violation("spawn_link", meta, file) | acc]}

          {:spawn_monitor, meta, _} = node, acc ->
            {node, [naked_spawn_violation("spawn_monitor", meta, file) | acc]}

          {{:., meta, [{:__aliases__, _, [:Task]}, :start]}, _, _} = node, acc ->
            {node, [naked_spawn_violation("Task.start", meta, file) | acc]}

          {{:., meta, [{:__aliases__, _, [:Task]}, :start_link]}, _, _} = node, acc ->
            {node, [naked_spawn_violation("Task.start_link", meta, file) | acc]}

          other, acc ->
            {other, acc}
        end)

      Enum.reverse(violations)
    end
  end

  defp naked_spawn_violation(call_name, meta, file) do
    line = Keyword.get(meta, :line, 1)
    col = Keyword.get(meta, :column, 1)

    %Violation{
      code: "ARCH_EX_003",
      file: file,
      line: line,
      col: col,
      message:
        "Naked process spawn '#{call_name}' is forbidden; concurrency must be managed via a Supervisor or Task.Supervisor"
    }
  end

  # ARCH_EX_004: Side-effect isolation - no IO.puts, Process.sleep, :timer.sleep, System.cmd in pure domain
  defp check_arch_ex_004(body, file) do
    {_, violations} =
      Macro.prewalk(body, [], fn
        {{:., meta, [{:__aliases__, _, [:IO]}, func]}, _, _} = node, acc
        when func in [:puts, :write, :inspect] ->
          {node, [side_effect_violation("IO.#{func}", meta, file) | acc]}

        {{:., meta, [{:__aliases__, _, [:Process]}, :sleep]}, _, _} = node, acc ->
          {node, [side_effect_violation("Process.sleep", meta, file) | acc]}

        {{:., meta, [:timer, :sleep]}, _, _} = node, acc ->
          {node, [side_effect_violation(":timer.sleep", meta, file) | acc]}

        {{:., meta, [{:__aliases__, _, [:System]}, :cmd]}, _, _} = node, acc ->
          {node, [side_effect_violation("System.cmd", meta, file) | acc]}

        other, acc ->
          {other, acc}
      end)

    Enum.reverse(violations)
  end

  defp side_effect_violation(call_name, meta, file) do
    line = Keyword.get(meta, :line, 1)
    col = Keyword.get(meta, :column, 1)

    %Violation{
      code: "ARCH_EX_004",
      file: file,
      line: line,
      col: col,
      message:
        "Side-effect call '#{call_name}' is forbidden in pure domain modules; isolate side-effects to adapters or GenServers"
    }
  end
end
