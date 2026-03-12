defmodule CtaraDhruv.Time do
  @moduledoc false

  alias CtaraDhruv.Native

  def utc_to_jd_tdb(engine, request),
    do: Native.call_engine(&Native.time_run/2, engine, Map.put(request, :op, :utc_to_jd_tdb))

  def jd_tdb_to_utc(engine, request),
    do: Native.call_engine(&Native.time_run/2, engine, Map.put(request, :op, :jd_tdb_to_utc))

  def nutation(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :nutation))
end
