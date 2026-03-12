defmodule CtaraDhruv.Vedic do
  @moduledoc false

  alias CtaraDhruv.Native

  def ayanamsha(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :ayanamsha))

  def lunar_node(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :lunar_node))

  def rise_set(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :rise_set))

  def all_events(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :all_events))

  def lagna(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :lagna))

  def mc(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :mc))

  def ramc(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :ramc))

  def bhavas(engine, request),
    do: Native.call_engine(&Native.vedic_run/2, engine, Map.put(request, :op, :bhavas))
end
