defmodule CtaraDhruv.Tara do
  @moduledoc false

  alias CtaraDhruv.Native

  def compute(engine, request),
    do: Native.call_engine(&Native.tara_run/2, engine, Map.put(request, :op, :compute))

  def catalog_info(engine),
    do: Native.call_engine(&Native.tara_run/2, engine, %{op: :catalog_info})
end
