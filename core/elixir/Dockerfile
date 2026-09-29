FROM elixir:1.17-alpine

WORKDIR /app
RUN apk add --no-cache git build-base

ENV MIX_ENV=prod
ENV PORT=4000

COPY mix.exs mix.lock ./
RUN mix local.hex --force && mix local.rebar --force
RUN mix deps.get --only prod

COPY config config
COPY lib lib
RUN mix compile

EXPOSE 4000
CMD ["mix", "run", "--no-halt"]
