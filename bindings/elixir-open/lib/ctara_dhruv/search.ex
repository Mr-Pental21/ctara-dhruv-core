defmodule CtaraDhruv.Search do
  @moduledoc false

  alias CtaraDhruv.Native

  def conjunction(engine, request),
    do: Native.call_engine(&Native.search_run/2, engine, Map.put(request, :op, :conjunction))

  def grahan(engine, request),
    do: Native.call_engine(&Native.search_run/2, engine, Map.put(request, :op, :grahan))

  def lunar_phase(engine, request),
    do: Native.call_engine(&Native.search_run/2, engine, Map.put(request, :op, :lunar_phase))

  def sankranti(engine, request),
    do: Native.call_engine(&Native.search_run/2, engine, Map.put(request, :op, :sankranti))

  def motion(engine, request),
    do: Native.call_engine(&Native.search_run/2, engine, Map.put(request, :op, :motion))
end
