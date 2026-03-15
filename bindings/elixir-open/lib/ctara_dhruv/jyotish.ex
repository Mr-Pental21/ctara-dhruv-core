defmodule CtaraDhruv.Jyotish do
  @moduledoc false

  alias CtaraDhruv.Native

  def graha_longitudes(engine, request),
    do:
      Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :graha_longitudes))

  def graha_positions(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :graha_positions))

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

  def amsha(engine, request),
    do: Native.call_engine(&Native.jyotish_run/2, engine, Map.put(request, :op, :amsha))
end
