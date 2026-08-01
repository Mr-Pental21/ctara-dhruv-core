defmodule CtaraDhruv.Jyotish do
  @moduledoc """
  Jyotish chart computations.

  `full_kundali/2` uses the request's `:sankranti_config` when provided and the
  wrapper's resolved ayanamsha defaults otherwise. `full_kundali/3` is a
  convenience arity for explicitly supplying the chart ayanamsha config from
  Elixir.
  """

  alias CtaraDhruv.Native

  def graha_longitudes(engine, request),
    do:
      Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :graha_longitudes))

  def moving_osculating_apogees(engine, request),
    do:
      Native.call_engine(
        &Native.jyotish_run/2,
        engine,
        Map.put(request, :op, :moving_osculating_apogees)
      )

  def graha_positions(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :graha_positions))

  @doc """
  Fixed-cadence sampling of `graha_positions/2` over `[from_utc, to_utc]`.

  The request takes `:from_utc`, `:to_utc`, and `:step_minutes` instead of
  `:utc` (endpoints inclusive when on the grid, at most 10,000 points) and
  returns `%{"points" => [%{"utc" => ..., "jd_utc" => ..., "positions" => ...}]}`
  where each `positions` value has the same shape as the single-epoch op.
  """
  def graha_positions_series(engine, request),
    do:
      Native.call_engine(
        &Native.jyotish_run/2,
        engine,
        Map.put(request, :op, :graha_positions_series)
      )

  def special_lagnas(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :special_lagnas))

  def arudha(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :arudha))

  def upagrahas(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :upagrahas))

  def bindus(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :bindus))

  def ashtakavarga(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :ashtakavarga))

  def drishti(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :drishti))

  def charakaraka(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :charakaraka))

  def shadbala(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :shadbala))

  def bhavabala(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :bhavabala))

  def vimsopaka(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :vimsopaka))

  def balas(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :balas))

  def avastha(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :avastha))

  def full_kundali(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :full_kundali))

  def full_kundali(engine, request, sankranti_config),
    do: full_kundali(engine, put_sankranti_config(request, sankranti_config))

  def amsha(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :amsha))

  @doc """
  Fixed-cadence sampling of slim varga charts over `[from_utc, to_utc]`.

  The request takes `:from_utc`, `:to_utc`, `:step_minutes`, `:location`, and
  `:amsha_requests` (`[%{code: ..., variation: ...}]`), plus an optional
  `:include_grahas` boolean (default `false`) that adds the nine graha varga
  entries per chart. At most 100,000 cells (points x unique requests). Returns
  `%{"points" => [%{"utc" => ..., "jd_utc" => ..., "charts" => [%{"amsha" =>
  ..., "sanskrit_name" => ..., "variation_code" => ..., "lagna" => ...,
  "grahas" => [...] | nil}]}]}` with charts in request order and entries in
  the single-epoch `amsha/2` entry shape. `"sanskrit_name"` is the library's
  display name for the amsha (`"Navamsha"`) alongside the code-derived
  `"amsha"` key (`"d9"`).
  """
  def amsha_series(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :amsha_series))

  @doc """
  Exact varga-lagna rashi segments over `[from_utc, to_utc]`.

  The request takes `:from_utc`, `:to_utc`, `:location`, and
  `:amsha_requests`, plus an optional `:max_segments` cap (default `0` selects
  the 50,000 ceiling). Returns `%{"entries" => [%{"amsha" => ...,
  "variation_code" => ..., "segments" => [%{"rashi" => ..., "rashi_index" =>
  ..., "start" => utc, "end" => utc}]}], "truncated" => bool,
  "next_from_utc" => utc | nil}` with one entry per unique request and exact
  transition boundaries. On truncation resume from `next_from_utc`.
  """
  def amsha_lagna_events(engine, request),
    do:
      Native.call_engine(
        &Native.jyotish_run/2,
        engine,
        Map.put(request, :op, :amsha_lagna_events)
      )

  defp put_sankranti_config(request, sankranti_config),
    do: Map.put(request, :sankranti_config, sankranti_config)
end
