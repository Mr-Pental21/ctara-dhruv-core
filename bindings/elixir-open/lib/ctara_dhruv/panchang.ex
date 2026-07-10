defmodule CtaraDhruv.Panchang do
  @moduledoc false

  alias CtaraDhruv.Native

  def tithi(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :tithi))

  def karana(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :karana))

  def yoga(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :yoga))

  def nakshatra(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :nakshatra))

  def vaar(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :vaar))

  def hora(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :hora))

  def ghatika(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :ghatika))

  def masa(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :masa))

  def ayana(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :ayana))

  def varsha(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :varsha))

  def daily(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :daily))

  @doc """
  Panchang boundary events over `[from_utc, to_utc]`.

  The request takes `:from_utc` and `:to_utc`, an optional `:include_mask`
  restricted to location-independent elements (`tithi`, `karana`, `yoga`,
  `nakshatra`, `masa`, `ayana`, `varsha`; defaults to all of them), and an
  optional `:max_events` cap (default `0` selects the 50,000 ceiling).
  Returns per-kind lists in the same shape as the per-moment ops plus
  `"truncated"` and `"next_from_utc"`; consecutive segments of one kind chain
  exactly (`end == next start`). On truncation resume from `next_from_utc`
  and deduplicate on `{kind, start}`.
  """
  def events(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :events))

  def elongation_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :elongation_at))

  def sidereal_sum_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :sidereal_sum_at))

  def vedic_day_sunrises(engine, request),
    do:
      Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :vedic_day_sunrises))

  def body_ecliptic_lon_lat(engine, request),
    do:
      Native.call_engine(
        &Native.panchang_run/2,
        engine,
        Map.put(request, :op, :body_ecliptic_lon_lat)
      )

  def tithi_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :tithi_at))

  def karana_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :karana_at))

  def yoga_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :yoga_at))

  def nakshatra_at(engine, request),
    do: Native.call_engine(&Native.panchang_run/2, engine, Map.put(request, :op, :nakshatra_at))

  def vaar_from_sunrises(engine, request),
    do:
      Native.call_engine(
        &Native.panchang_run/2,
        engine,
        Map.put(request, :op, :vaar_from_sunrises)
      )

  def hora_from_sunrises(engine, request),
    do:
      Native.call_engine(
        &Native.panchang_run/2,
        engine,
        Map.put(request, :op, :hora_from_sunrises)
      )

  def ghatika_from_sunrises(engine, request),
    do:
      Native.call_engine(
        &Native.panchang_run/2,
        engine,
        Map.put(request, :op, :ghatika_from_sunrises)
      )

  def ghatika_from_elapsed(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :ghatika_from_elapsed))

  def ghatikas_since_sunrise(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :ghatikas_since_sunrise))
end
