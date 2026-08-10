defmodule CtaraDhruv.Native do
  @moduledoc false
  # Result terms arrive from the NIF already in their final shape: atom map
  # keys and atomized enum values (see ENUM_VALUE_KEYS in
  # native/dhruv_elixir_nif/src/lib.rs). The historical deep postprocess walk
  # now happens during term encoding on the DirtyCpu scheduler instead of in
  # the calling process, so `handle/1` is a passthrough.

  use Rustler,
    otp_app: :ctara_dhruv,
    crate: "dhruv_elixir_nif",
    path: "native/dhruv_elixir_nif",
    mode: if(Mix.env() == :prod, do: :release, else: :debug)

  alias CtaraDhruv.Engine
  alias CtaraDhruv.Error

  def engine_new(_config), do: :erlang.nif_error(:nif_not_loaded)
  def engine_close(_resource), do: :erlang.nif_error(:nif_not_loaded)
  def engine_load_config(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def engine_clear_config(_resource), do: :erlang.nif_error(:nif_not_loaded)
  def engine_replace_spks(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def engine_list_spks(_resource), do: :erlang.nif_error(:nif_not_loaded)
  def engine_load_eop(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def engine_clear_eop(_resource), do: :erlang.nif_error(:nif_not_loaded)
  def engine_load_tara_catalog(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def engine_reset_tara_catalog(_resource), do: :erlang.nif_error(:nif_not_loaded)
  def ephemeris_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def time_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def util_run(_request), do: :erlang.nif_error(:nif_not_loaded)
  def vedic_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def panchang_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def search_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def jyotish_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def dasha_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)
  def tara_run(_resource, _request), do: :erlang.nif_error(:nif_not_loaded)

  def create_engine(config), do: handle(engine_new(normalize_term(config)))

  def call_engine_noarg(fun, %Engine{} = engine), do: handle(fun.(engine.resource))

  def call_engine(fun, %Engine{} = engine, request),
    do: handle(fun.(engine.resource, normalize_term(request)))

  def call_util(fun, request), do: handle(fun.(normalize_term(request)))

  defp handle({:ok, result}), do: {:ok, result}
  defp handle({:error, %{} = error}), do: {:error, Error.from_term(error)}

  defp normalize_term(term) when is_atom(term) and term not in [true, false, nil],
    do: Atom.to_string(term)

  defp normalize_term(%_{} = struct), do: struct |> Map.from_struct() |> normalize_term()

  defp normalize_term(%{} = map),
    do: Map.new(map, fn {k, v} -> {normalize_key(k), normalize_term(v)} end)

  defp normalize_term(list) when is_list(list), do: Enum.map(list, &normalize_term/1)
  defp normalize_term(term), do: term

  defp normalize_key(key) when is_atom(key), do: Atom.to_string(key)
  defp normalize_key(key), do: key
end
