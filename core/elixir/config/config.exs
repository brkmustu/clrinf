import Config

nats_url =
  case System.get_env("NATS_URL", "nats://localhost:51700") do
    "" -> nil
    url -> url
  end

start_streaming =
  case System.get_env("CLRINF_MODE", "streaming") do
    "streaming" -> true
    "core" -> false
    mode -> raise "Unsupported CLRINF_MODE: #{inspect(mode)}"
  end

# clrinf port scheme: 51704 is the default for the Elixir streaming service.
config :clrinfex,
  port: String.to_integer(System.get_env("PORT") || "51704"),
  nats_url: nats_url,
  nats_subject: System.get_env("CLRINF_NATS_SUBJECT", "com.clrinf.>"),
  start_streaming: start_streaming

config :logger, level: :info

if File.exists?("config/#{Mix.env()}.exs"),
  do: import_config("#{Mix.env()}.exs")
