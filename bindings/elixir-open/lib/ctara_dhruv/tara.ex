defmodule CtaraDhruv.Tara do
  @moduledoc false

  alias CtaraDhruv.Native

  def compute(engine, request),
    do: Native.call_engine(&Native.tara_run/2, engine, Map.put(request, :op, :compute))

  def catalog_info(engine),
    do: Native.call_engine(&Native.tara_run/2, engine, %{op: :catalog_info})

  def propagate_position(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :tara_propagate_position))

  def apply_aberration(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :tara_apply_aberration))

  def apply_light_deflection(request),
    do:
      Native.call_util(&Native.util_run/1, Map.put(request, :op, :tara_apply_light_deflection))

  def galactic_anticenter_icrs(),
    do: Native.call_util(&Native.util_run/1, %{op: :tara_galactic_anticenter_icrs})
end
