defmodule CtaraDhruv.Ephemeris do
  @moduledoc false

  alias CtaraDhruv.Native

  def query(engine, request),
    do: Native.call_engine(&Native.ephemeris_run/2, engine, Map.put(request, :op, :query))

  def body_ecliptic_lon_lat(engine, request),
    do:
      Native.call_engine(
        &Native.ephemeris_run/2,
        engine,
        Map.put(request, :op, :body_ecliptic_lon_lat)
      )

  def cartesian_to_spherical(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :cartesian_to_spherical))
end
