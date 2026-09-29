import Config

# Test ortamında streaming ve NatsBridge başlatılmaz.
# Her test kendi supervised sürecini start_supervised!/1 ile açar;
# uygulama süpervizörüyle çakışmaz.
config :clrinfex,
  start_streaming: false,
  port: 51704

config :logger, level: :warning
