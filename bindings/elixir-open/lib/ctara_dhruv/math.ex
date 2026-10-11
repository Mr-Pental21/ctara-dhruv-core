defmodule CtaraDhruv.Math do
  @moduledoc false

  alias CtaraDhruv.Native

  def rashi_from_longitude(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :rashi_from_longitude))

  def nakshatra_from_longitude(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :nakshatra_from_longitude)
      )

  def nakshatra28_from_longitude(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :nakshatra28_from_longitude)
      )

  def rashi_from_tropical(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :rashi_from_tropical))

  def nakshatra_from_tropical(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :nakshatra_from_tropical)
      )

  def nakshatra28_from_tropical(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :nakshatra28_from_tropical)
      )

  def graha_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :graha_name))

  def yogini_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :yogini_name))

  def rashi_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :rashi_name))

  def nakshatra_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :nakshatra_name))

  def nakshatra28_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :nakshatra28_name))

  def sphuta_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :sphuta_name))

  def upagraha_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :upagraha_name))

  @doc """
  Engine-free batched varga mapping.

  The request takes `:longitudes` (list of sidereal longitudes in degrees)
  and `:amsha_requests` (`[%{code: ..., variation: ...}]`). Returns
  `%{"entries" => [[...]]}`: one list per longitude with one map per request
  carrying `"amsha_longitude"`, `"rashi"`, `"rashi_index"`,
  `"degrees_in_rashi"`, and `"dms"`.
  """
  def amsha_rashi_infos(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :amsha_rashi_infos))

  def amsha_variations(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :amsha_variations))

  def amsha_variations_many(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :amsha_variations_many))

  def hora_lord(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :hora_lord))

  def masa_lord(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :masa_lord))

  def samvatsara_lord(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :samvatsara_lord))

  def exaltation_degree(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :exaltation_degree))

  def debilitation_degree(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :debilitation_degree))

  def moolatrikone_range(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :moolatrikone_range))

  def combustion_threshold(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :combustion_threshold))

  def combust?(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :is_combust))

  def all_combustion_status(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :all_combustion_status))

  def naisargika_maitri(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :naisargika_maitri))

  def tatkalika_maitri(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :tatkalika_maitri))

  def panchadha_maitri(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :panchadha_maitri))

  def dignity_in_rashi(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :dignity_in_rashi))

  def dignity_in_rashi_with_positions(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :dignity_in_rashi_with_positions)
      )

  def node_dignity_in_rashi(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :node_dignity_in_rashi))

  def natural_benefic_malefic(request),
    do:
      Native.call_util(
        &Native.util_run/1,
        Map.put(request, :op, :natural_benefic_malefic)
      )

  def moon_benefic_nature(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :moon_benefic_nature))

  def graha_gender(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :graha_gender))

  def graha_drishti(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :graha_drishti))

  def graha_drishti_matrix(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :graha_drishti_matrix))

  def sun_based_upagrahas(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :sun_based_upagrahas))

  def time_upagraha_jd(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :time_upagraha_jd))

  def all_sphutas(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :all_sphutas))

  def calculate_ashtakavarga(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :calculate_ashtakavarga))

  def calculate_bav(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :calculate_bav))

  def calculate_all_bav(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :calculate_all_bav))

  def calculate_sav(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :calculate_sav))

  def trikona_sodhana(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :trikona_sodhana))

  def ekadhipatya_sodhana(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :ekadhipatya_sodhana))

  @doc """
  Engine-free tara bala of `:target_nakshatra_index` counted from
  `:reference_nakshatra_index` (both 0-based, reduced modulo 27). Returns
  `%{tara: ..., tara_name: ..., tara_index: 0..8, count: 1..27, cycle: 1..3,
  favorable: bool}`.
  """
  def tara_bala(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :tara_bala))

  @doc "Name of tara `:index` (0=Janma .. 8=Parama Mitra)."
  def tara_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :tara_name))

  @doc "0-based kakshya (eighth of a rashi, 0..7) of `:sidereal_longitude_deg`."
  def kakshya_index(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :kakshya_index))

  @doc """
  Lord of kakshya `:kakshya_index` (0..7) as a BAV contributor index
  (0=Sun .. 6=Saturn, 7=Lagna); the order is Saturn, Jupiter, Mars, Sun,
  Venus, Mercury, Moon, Lagna.
  """
  def kakshya_lord(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :kakshya_lord))

  @doc """
  Whether the kakshya occupied by `:sidereal_longitude_deg` carries a bindu in
  `:bav` (a bhinna ashtakavarga map with `:graha_index`, `:points`, and
  `:contributors`, as returned by `calculate_bav/1`).
  """
  def kakshya_bindu(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :kakshya_bindu))

  @doc """
  Engine-free transit ashtakavarga: scores `:transit_sidereal_lons` (7
  sidereal longitudes, Sun through Saturn) against a natal ashtakavarga given
  either as `:natal` (the `calculate_ashtakavarga/1` result map) or as the
  natal `:graha_rashis` / `:lagna_rashi` inputs. Returns `%{entries: [...]}`
  in the `CtaraDhruv.Jyotish.transit_ashtakavarga/2` entry shape.
  """
  def transit_ashtakavarga(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :transit_ashtakavarga))

  @doc "Name and defining event of samvatsara office `:index` (0=Raja .. 8=Durgesha)."
  def samvatsara_office_name(request),
    do: Native.call_util(&Native.util_run/1, Map.put(request, :op, :samvatsara_office_name))
end
