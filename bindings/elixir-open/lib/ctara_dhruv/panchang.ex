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
end
