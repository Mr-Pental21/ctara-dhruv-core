defmodule CtaraDhruv.Dasha do
  @moduledoc false

  alias CtaraDhruv.Native

  def hierarchy(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :hierarchy))

  def snapshot(engine, request),
    do: Native.call_engine(&Native.dasha_run/2, engine, Map.put(request, :op, :snapshot))
end
