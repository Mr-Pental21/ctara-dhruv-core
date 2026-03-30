defmodule CtaraDhruv.Dasha do
  @moduledoc false

  alias CtaraDhruv.Native

  def hierarchy(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :hierarchy))

  def snapshot(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :snapshot))

  def level0(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :level0))

  def level0_entity(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :level0_entity))

  def children(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :children))

  def child_period(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :child_period))

  def complete_level(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :complete_level))
end
