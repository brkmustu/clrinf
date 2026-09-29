ExUnit.start(exclude: if(System.get_env("NATS_TEST_URL"), do: [], else: [:nats]))

for app <- [:gnat, :plug_cowboy, :elixir_uuid] do
  {:ok, _apps} = Application.ensure_all_started(app)
end
