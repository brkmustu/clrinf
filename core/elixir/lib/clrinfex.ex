defmodule Clrinfex do
  @moduledoc """
  clrinfex: High-performance BEAM/Elixir Real-Time Event & Media Streaming Backbone for the clrinf ecosystem.
  """

  @doc "Broadcasts a domain CloudEvent to all live stream listeners."
  def broadcast(event), do: Clrinfex.NatsBridge.broadcast_event(event)

  @doc "Tracks a viewer/client in an active audio/video or event channel."
  def track_viewer(stream_id, user_id, meta \\ %{}), do: Clrinfex.Presence.track(stream_id, user_id, meta)
end
