defmodule CtaraDhruvTest do
  use ExUnit.Case

  alias CtaraDhruv.{Dasha, Engine, Ephemeris, Jyotish, Math, Panchang, Search, Tara, Time, Vedic}

  @repo_root Path.expand("../../..", __DIR__)
  @kernel_dir Path.join(@repo_root, "kernels/data")
  @spk Path.join(@kernel_dir, "de442s.bsp")
  @lsk Path.join(@kernel_dir, "naif0012.tls")
  @eop Path.join(@kernel_dir, "finals2000A.all")
  @tara Path.join(@kernel_dir, "hgca_tara.json")

  defp with_engine do
    if File.exists?(@spk) and File.exists?(@lsk) do
      {:ok, engine} =
        Engine.new(%{
          spk_paths: [@spk],
          lsk_path: @lsk,
          cache_capacity: 64,
          strict_validation: false,
          time_policy: %{mode: :hybrid_delta_t}
        })

      on_exit(fn -> Engine.close(engine) end)
      {:ok, engine}
    else
      :skip
    end
  end

  test "elixir graha name lookup uses canonical Mangal and Buddh" do
    assert {:ok, %{name: "Mangal"}} = Math.graha_name(%{index: 2})
    assert {:ok, %{name: "Buddh"}} = Math.graha_name(%{index: 3})
    assert {:ok, %{name: "Mangal"}} = Math.graha_name(%{graha: :mangal})
    assert {:ok, %{name: "Buddh"}} = Math.graha_name(%{graha: :buddh})

    assert {:ok, %{name: "Mangala"}} = Math.yogini_name(%{index: 0})
  end

  test "engine lifecycle and native families smoke" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        assert {:ok, _} = Ephemeris.cartesian_to_spherical(%{x: 1.0, y: 0.0, z: 0.0})
        assert {:ok, _} = Time.nutation(%{jd_tdb: 2_451_545.0})

        assert {:ok, _} =
                 Ephemeris.query(engine, %{
                   target: 499,
                   observer: 0,
                   frame: 1,
                   epoch_tdb_jd: 2_451_545.0
                 })

        assert {:ok, %{diagnostics: diagnostics}} =
                 Time.utc_to_jd_tdb(engine, %{
                   utc: %{year: 2015, month: 1, day: 1, hour: 12, minute: 0, second: 0.0}
                 })

        assert is_map(diagnostics)

        if File.exists?(@eop) do
          assert {:ok, _} = Engine.load_eop(engine, @eop)
          location = %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0}
          utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}

          assert {:ok, _} =
                   Vedic.ayanamsha(engine, %{
                     jd_tdb: 2_460_311.0,
                     system: :lahiri,
                     use_nutation: false
                   })

          assert {:ok, _} =
                   Vedic.rise_set(engine, %{utc: utc, location: location, event: :sunrise})

          assert {:ok, _} = Panchang.tithi(engine, %{utc: utc})
          assert {:ok, _} = Search.sankranti(engine, %{mode: :next, at_utc: utc})

          assert {:ok, %{events: eclipse}} =
                   Search.grahan(engine, %{
                     mode: :next,
                     kind: :surya,
                     at_utc: %{year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0.0},
                     location: %{
                       latitude_deg: 25.2854,
                       longitude_deg: -104.3,
                       altitude_m: 0.0
                     },
                     config: %{
                       include_path: true,
                       path_step_minutes: 10,
                       boundary_step_deg: 15,
                       include_contact_footprints: true,
                       include_umbra_footprints: true,
                       instantaneous_magnitude_levels: [0.5]
                     }
                   })

          assert eclipse.grahan_type == :total
          assert eclipse.besselian.l1 > 0.0
          assert length(eclipse.path) > 10
          assert length(eclipse.footprints) > 20
          assert eclipse.local.visible == true
          assert eclipse.local.grahan_type == :total
          assert eclipse.centrality == :full

          # Change 6: sampled footprints carry contains_pole; contact and
          # umbral footprints are present for a central event.
          [sampled_footprint | _] = eclipse.footprints
          assert sampled_footprint.contains_pole in [nil, :north, :south]
          contact_kinds = Enum.map(eclipse.contact_footprints, & &1.contact)
          assert contact_kinds == [:c1, :c2, :greatest, :c3, :c4]
          greatest_contact =
            Enum.find(eclipse.contact_footprints, &(&1.contact == :greatest))

          assert length(greatest_contact.boundary) > 10
          assert List.first(greatest_contact.boundary) == List.last(greatest_contact.boundary)
          assert greatest_contact.contains_pole in [nil, :north, :south]
          assert length(eclipse.umbra_footprints) > 10
          [umbra | _] = eclipse.umbra_footprints
          assert umbra.grahan_type == :total
          assert List.first(umbra.boundary) == List.last(umbra.boundary)

          # Instantaneous iso-magnitude rings at the greatest contact.
          assert [magnitude_ring | _] = greatest_contact.magnitude_rings
          assert magnitude_ring.level == 0.5
          assert length(magnitude_ring.boundary) > 10
          assert List.first(magnitude_ring.boundary) == List.last(magnitude_ring.boundary)
          assert magnitude_ring.contains_pole in [nil, :north, :south]
          assert Enum.any?(eclipse.footprints, &(&1.magnitude_rings != []))

          assert {:ok, %{events: field_eclipse, effective_config: effective_config}} =
                   Search.grahan(engine, %{
                     mode: :next,
                     kind: :surya,
                     at_utc: %{year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0.0},
                     config: %{
                       include_local_grid: true,
                       local_grid_step_deg: 20.0,
                       include_isolines: true,
                       duration_isoline_fractions: [0.5, 0.25, 2.0],
                       magnitude_isoline_levels: [0.5]
                     }
                   })

          # Effective config echoes clamped/sanitized values for cache keys.
          assert effective_config.local_grid_step_deg == 10.0
          assert effective_config.duration_isoline_fractions == [0.25, 0.5]
          assert effective_config.magnitude_isoline_levels == [0.5]
          assert effective_config.include_local_grid == true

          assert field_eclipse.centrality == :full
          assert length(field_eclipse.local_grid) > 20
          [grid_sample | _] = field_eclipse.local_grid
          assert is_float(grid_sample.latitude_deg)
          assert is_float(grid_sample.magnitude)
          assert is_float(grid_sample.visible_duration_seconds)
          assert is_map(grid_sample.maximum_utc)
          assert is_map(grid_sample.first_contact_utc)

          assert [visibility_ring | _] = field_eclipse.isolines.visibility_boundary
          assert length(visibility_ring.boundary) > 10
          assert List.first(visibility_ring.boundary) == List.last(visibility_ring.boundary)
          assert visibility_ring.contains_pole in [nil, :north, :south]
          assert [duration_level | _] = field_eclipse.isolines.duration_isolines
          assert duration_level.fraction == 0.25
          assert [magnitude_level | _] = field_eclipse.isolines.magnitude_isolines
          assert magnitude_level.level == 0.5
          assert {:ok, _} = Jyotish.graha_positions(engine, %{utc: utc, location: location})

          assert {:ok, equatorial_positions} =
                   Jyotish.graha_positions(engine, %{
                     utc: utc,
                     location: location,
                     graha_positions_config: %{include_equatorial: true}
                   })

          assert equatorial_positions.earth_orientation_valid == true
          assert is_float(equatorial_positions.gmst_deg)
          assert equatorial_positions.gmst_deg >= 0.0
          assert equatorial_positions.gmst_deg < 360.0
          assert is_float(equatorial_positions.gast_deg)

          for entry <- equatorial_positions.grahas do
            assert entry.equatorial_valid == true
            assert entry.right_ascension_deg >= 0.0
            assert entry.right_ascension_deg < 360.0
            assert entry.declination_deg >= -90.0
            assert entry.declination_deg <= 90.0
          end

          rahu = Enum.find(equatorial_positions.grahas, &(&1.graha == :rahu))
          ketu = Enum.find(equatorial_positions.grahas, &(&1.graha == :ketu))
          assert rahu.ecliptic_latitude_deg == 0.0
          assert ketu.ecliptic_latitude_deg == 0.0
          assert equatorial_positions.lagna.ecliptic_latitude_deg == 0.0

          to_utc = Map.put(utc, :hour, utc.hour + 2)

          assert {:ok, series} =
                   Jyotish.graha_positions_series(engine, %{
                     from_utc: utc,
                     to_utc: to_utc,
                     step_minutes: 60,
                     location: location,
                     graha_positions_config: %{include_equatorial: true}
                   })

          assert length(series.points) == 3
          [first_point | _] = series.points
          assert first_point.positions.gmst_deg == equatorial_positions.gmst_deg
          assert is_float(first_point.jd_utc)

          assert {:error, _} =
                   Jyotish.graha_positions_series(engine, %{
                     from_utc: utc,
                     to_utc: to_utc,
                     location: location
                   })

          assert {:ok, _} = Jyotish.bindus(engine, %{utc: utc, location: location})

          assert {:ok, _} =
                   Dasha.hierarchy(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     max_level: 1
                   })

          assert {:ok, level0} =
                   Dasha.level0(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari
                   })

          assert length(level0) > 0
          first = hd(level0)

          assert {:ok, level0_two_cycles} =
                   Dasha.level0(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     variation: %{cycles: 2}
                   })

          assert length(level0_two_cycles) == 2 * length(level0)

          assert {:ok, level0_span} =
                   Dasha.level0(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     variation: %{min_span_years: 200.0}
                   })

          assert length(level0_span) == 2 * length(level0)

          assert {:error, %CtaraDhruv.Error{kind: :invalid_request}} =
                   Dasha.level0(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     variation: %{cycles: 0}
                   })

          assert {:ok, level0_entity} =
                   Dasha.level0_entity(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     entity: first.entity
                   })

          assert level0_entity.entity.index == first.entity.index

          assert {:ok, children} =
                   Dasha.children(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     parent: first
                   })

          assert length(children) > 0
          first_child = hd(children)

          assert {:ok, child_period} =
                   Dasha.child_period(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     parent: first,
                     child_entity: first_child.entity
                   })

          assert child_period.entity.index == first_child.entity.index

          assert {:ok, complete_level} =
                   Dasha.complete_level(engine, %{
                     birth_utc: utc,
                     location: location,
                     system: :vimshottari,
                     parent_periods: level0,
                     child_level: :antardasha
                   })

          assert length(complete_level) >= length(children)
        end

        if File.exists?(@tara) do
          assert {:ok, _} = Engine.load_tara_catalog(engine, @tara)
          assert {:ok, _} = Tara.catalog_info(engine)
        else
          assert {:ok, _} = Tara.catalog_info(engine)
        end
    end
  end

  test "search ops accept transit bodies, multi-angle sweeps, and sidereal echo" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        january = %{year: 2024, month: 1, day: 1, hour: 0, minute: 0, second: 0.0}
        february = %{january | month: 2}
        end_of_january = %{january | day: 31}

        # Non-Sun ingress: the Moon changes rashi every ~2.5 days.
        assert {:ok, %{events: moon_ingresses}} =
                 Search.sankranti(engine, %{
                   mode: :range,
                   start_utc: january,
                   end_utc: february,
                   body: :moon
                 })

        assert length(moon_ingresses) >= 12

        for event <- moon_ingresses do
          assert event.body == :moon
          assert Map.has_key?(event, :is_retrograde)
          offset = :math.fmod(event.sidereal_longitude_deg, 30.0)
          assert min(offset, 30.0 - offset) < 1.0e-3
        end

        # Backward compat: no :body means the Sun, with the legacy keys intact.
        assert {:ok, %{events: sun_event}} =
                 Search.sankranti(engine, %{mode: :next, at_utc: january})

        assert sun_event.body == :sun
        assert is_float(sun_event.sun_sidereal_longitude_deg)
        assert is_float(sun_event.sun_tropical_longitude_deg)
        assert sun_event.sidereal_longitude_deg == sun_event.sun_sidereal_longitude_deg

        # Conjunction accepts Rahu/Ketu; Sun conjoins Rahu near the April
        # 2024 eclipse season.
        assert {:ok, %{events: node_event}} =
                 Search.conjunction(engine, %{
                   mode: :next,
                   body1: :sun,
                   body2: :rahu,
                   at_utc: %{january | month: 3}
                 })

        refute is_nil(node_event)
        assert node_event.utc.year == 2024
        assert node_event.utc.month in [3, 4]

        # Multi-angle sweep: new moon and full moon in one range request.
        assert {:ok, %{events: phase_events}} =
                 Search.conjunction(engine, %{
                   mode: :range,
                   body1: :sun,
                   body2: :moon,
                   start_utc: january,
                   end_utc: end_of_january,
                   config: %{target_separations_deg: [0.0, 180.0], step_size_days: 0.5}
                 })

        assert length(phase_events) >= 2
        for event <- phase_events, do: assert(event.target_separation_deg in [0.0, 180.0])

        # Opt-in sidereal echo via :sankranti_config.
        assert {:ok, %{events: echo_event}} =
                 Search.conjunction(engine, %{
                   mode: :next,
                   body1: :sun,
                   body2: :moon,
                   at_utc: january,
                   sankranti_config: %{}
                 })

        assert is_float(echo_event.body1_sidereal_longitude_deg)
        assert is_float(echo_event.body2_sidereal_longitude_deg)

        # True-node stationary search works; the mean node is rejected.
        assert {:ok, %{events: stations}} =
                 Search.motion(engine, %{
                   mode: :range,
                   body: :rahu,
                   kind: :stationary,
                   start_utc: january,
                   end_utc: end_of_january
                 })

        assert stations != []

        assert {:error, _} =
                 Search.motion(engine, %{
                   mode: :range,
                   body: :rahu,
                   kind: :stationary,
                   start_utc: january,
                   end_utc: end_of_january,
                   config: %{node_mode: :mean}
                 })
    end
  end

  test "elixir engine constructor accepts omitted shared default fields" do
    if File.exists?(@spk) and File.exists?(@lsk) do
      assert {:ok, engine} =
               Engine.new(%{
                 spk_paths: [@spk],
                 lsk_path: @lsk,
                 time_policy: %{mode: :hybrid_delta_t}
               })

      assert {:ok, %{closed: true}} = Engine.close(engine)
    else
      assert true
    end
  end

  test "elixir config loading supports typed request and defaults mode" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        dir =
          Path.join(System.tmp_dir!(), "dhruv-config-#{System.unique_integer([:positive])}")

        config_path = Path.join(dir, "config.toml")
        File.mkdir_p!(dir)
        File.write!(config_path, "version = 1\n")

        on_exit(fn ->
          File.rm_rf(dir)
        end)

        loaded_recommended =
          Engine.load_config(engine, %{path: config_path, defaults_mode: :recommended})

        assert match?({:ok, %{loaded: true}}, loaded_recommended)

        assert {:ok, %{cleared: true}} = Engine.clear_config(engine)

        loaded_explicit = Engine.load_config(engine, %{path: config_path, defaults_mode: :none})
        assert match?({:ok, %{loaded: true}}, loaded_explicit)
    end
  end

  test "elixir wrapper exposes sidereal bhavas and full_kundali defaults" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        if File.exists?(@eop) do
          assert {:ok, _} = Engine.load_eop(engine, @eop)

          location = %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0}
          utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}
          request = %{utc: utc, location: location}
          sidereal = %{ayanamsha_system: :lahiri, use_nutation: false}

          assert {:ok, %{longitude_deg: tropical_lagna}} = Vedic.lagna(engine, request)
          assert {:ok, %{longitude_deg: sidereal_lagna}} = Vedic.lagna(engine, request, sidereal)
          assert abs(tropical_lagna - sidereal_lagna) > 0.1

          assert {:ok, %{longitude_deg: sidereal_mc}} = Vedic.mc(engine, request, sidereal)

          assert {:ok, bhavas} = Vedic.bhavas(engine, request, sidereal)
          assert length(bhavas.bhavas) == 12
          assert_in_delta bhavas.lagna_deg, sidereal_lagna, 1.0e-6
          assert_in_delta bhavas.mc_deg, sidereal_mc, 1.0e-6

          assert {:ok, chart} = Jyotish.full_kundali(engine, request, sidereal)
          assert is_map(chart.graha_positions)
          assert is_map(chart.graha_positions.lagna)
          assert is_float(chart.graha_positions.lagna.sidereal_longitude)
          assert is_map(chart.bhava_cusps)
          assert_in_delta chart.bhava_cusps.lagna_deg, sidereal_lagna, 1.0e-6
          assert_in_delta chart.bhava_cusps.mc_deg, sidereal_mc, 1.0e-6

          too_many_systems = List.duplicate(:vimshottari, 24)

          assert {:error, %CtaraDhruv.Error{kind: :invalid_request, message: message}} =
                   Jyotish.full_kundali(engine, %{
                     utc: utc,
                     location: location,
                     full_kundali_config: %{
                       include_dasha: true,
                       dasha_config: %{systems: too_many_systems}
                     }
                   })

          assert message =~ "systems may contain at most"
        else
          assert true
        end
    end
  end

  test "elixir jyotish wrappers accept amsha_selection and return resolved amsha union" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        if File.exists?(@eop) do
          assert {:ok, _} = Engine.load_eop(engine, @eop)

          location = %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0}
          utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}
          d2_variation = [%{code: 2, variation: 1}]
          d9_default = [%{code: 9}]

          assert {:ok, shadbala} =
                   Jyotish.shadbala(engine, %{
                     utc: utc,
                     location: location,
                     amsha_selection: d2_variation
                   })

          assert length(shadbala.entries) == 7

          assert {:ok, vimsopaka} =
                   Jyotish.vimsopaka(engine, %{
                     utc: utc,
                     location: location,
                     amsha_selection: d2_variation
                   })

          assert length(vimsopaka.entries) == 9

          assert {:ok, balas} =
                   Jyotish.balas(engine, %{
                     utc: utc,
                     location: location,
                     amsha_selection: d2_variation
                   })

          assert length(balas.shadbala.entries) == 7
          assert length(balas.vimsopaka.entries) == 9

          assert {:ok, avastha} =
                   Jyotish.avastha(engine, %{
                     utc: utc,
                     location: location,
                     amsha_selection: d9_default
                   })

          assert length(avastha.entries) == 9

          assert {:ok, chart} =
                   Jyotish.full_kundali(engine, %{
                     utc: utc,
                     location: location,
                     full_kundali_config: %{
                       include_amshas: true,
                       include_shadbala: true,
                       include_vimsopaka: true,
                       amsha_selection: d2_variation
                     }
                   })

          assert length(chart.amshas.charts) == 16
          assert hd(chart.amshas.charts).amsha == "d2"
          assert hd(chart.amshas.charts).sanskrit_name == "Hora"
          assert hd(chart.amshas.charts).variation == "cancer-leo-only"
          assert Enum.any?(chart.amshas.charts, &(&1.amsha == "d60"))

          assert Enum.any?(
                   chart.amshas.charts,
                   &(&1.amsha == "d60" and &1.sanskrit_name == "Shashtiamsha")
                 )
        else
          assert true
        end
    end
  end

  test "engine replaces and lists spks" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        assert {:ok, initial} = Engine.list_spks(engine)
        assert length(initial.spks) == 1
        assert hd(initial.spks).generation == 0

        assert {:ok, report} = Engine.replace_spks(engine, [@spk, @spk])
        assert report.generation == 1
        assert report.active_count == 2
        assert report.loaded_count == 0
        assert report.reused_count == 2

        assert {:ok, active} = Engine.list_spks(engine)
        assert length(active.spks) == 2
        assert Enum.all?(active.spks, &(&1.generation == report.generation))

        missing = Path.join(@kernel_dir, "missing.bsp")
        assert {:error, _} = Engine.replace_spks(engine, [missing])
        assert {:ok, after_failure} = Engine.list_spks(engine)
        assert hd(after_failure.spks).generation == report.generation
    end
  end

  test "elixir math exposes engine-free batched amsha mapping" do
    assert {:ok, %{entries: [first, second]}} =
             Math.amsha_rashi_infos(%{
               longitudes: [15.0, 100.0],
               amsha_requests: [%{code: 1}, %{code: 9}]
             })

    assert [d1, d9] = first
    assert d1.rashi_index == 0
    assert_in_delta d1.degrees_in_rashi, 15.0, 1.0e-9
    assert_in_delta d1.amsha_longitude, 15.0, 1.0e-9
    assert d9.rashi_index == 4
    assert_in_delta d9.amsha_longitude, 135.0, 1.0e-9

    assert [d1b, d9b] = second
    assert d1b.rashi_index == 3
    assert_in_delta d1b.degrees_in_rashi, 10.0, 1.0e-9
    assert d9b.rashi_index == 6
    assert_in_delta d9b.amsha_longitude, 180.0, 1.0e-9

    assert {:error, %CtaraDhruv.Error{}} =
             Math.amsha_rashi_infos(%{
               longitudes: [15.0],
               amsha_requests: [%{code: 13}]
             })
  end

  test "elixir batch sweep ops: amsha_series, amsha_lagna_events, panchang events" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        if File.exists?(@eop) do
          assert {:ok, _} = Engine.load_eop(engine, @eop)

          location = %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0}
          from_utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}
          to_utc = %{from_utc | hour: 7}

          assert {:ok, series} =
                   Jyotish.amsha_series(engine, %{
                     from_utc: from_utc,
                     to_utc: to_utc,
                     step_minutes: 30,
                     location: location,
                     amsha_requests: [%{code: 1}, %{code: 9}]
                   })

          assert length(series.points) == 3
          [point | _] = series.points
          assert is_float(point.jd_utc)
          assert [d1_chart, d9_chart] = point.charts
          assert d1_chart.amsha == "d1"
          assert d9_chart.amsha == "d9"
          assert d1_chart.variation_code == 0
          assert is_float(d1_chart.lagna.sidereal_longitude)
          assert is_nil(d1_chart.grahas)

          assert {:ok, series_with_grahas} =
                   Jyotish.amsha_series(engine, %{
                     from_utc: from_utc,
                     to_utc: to_utc,
                     step_minutes: 30,
                     location: location,
                     amsha_requests: [%{code: 9}],
                     include_grahas: true
                   })

          [point | _] = series_with_grahas.points
          assert [chart] = point.charts
          assert length(chart.grahas) == 9

          assert {:error, %CtaraDhruv.Error{}} =
                   Jyotish.amsha_series(engine, %{
                     from_utc: from_utc,
                     to_utc: to_utc,
                     step_minutes: 0,
                     location: location,
                     amsha_requests: [%{code: 1}]
                   })

          assert {:ok, events} =
                   Jyotish.amsha_lagna_events(engine, %{
                     from_utc: from_utc,
                     to_utc: %{from_utc | hour: 12},
                     location: location,
                     amsha_requests: [%{code: 1}]
                   })

          assert [entry] = events.entries
          assert entry.amsha == "d1"
          assert length(entry.segments) >= 2
          refute events.truncated
          assert is_nil(events.next_from_utc)

          entry.segments
          |> Enum.chunk_every(2, 1, :discard)
          |> Enum.each(fn [a, b] -> assert Map.fetch!(a, :end) == b.start end)

          assert {:ok, panchang_events} =
                   Panchang.events(engine, %{
                     from_utc: from_utc,
                     to_utc: %{from_utc | day: 18},
                     include_mask: [:tithi, :nakshatra]
                   })

          assert length(panchang_events.tithi) >= 3
          assert length(panchang_events.nakshatra) >= 3
          assert panchang_events.karana == []
          refute panchang_events.truncated

          panchang_events.tithi
          |> Enum.chunk_every(2, 1, :discard)
          |> Enum.each(fn [a, b] -> assert Map.fetch!(a, :end) == b.start end)

          # Location-dependent elements are opt-in and need :location.
          assert {:ok, located_events} =
                   Panchang.events(engine, %{
                     from_utc: from_utc,
                     to_utc: %{from_utc | day: 18},
                     location: location,
                     include_mask: [:vaar, :hora, :ghatika]
                   })

          assert length(located_events.vaar) in 3..5
          assert length(located_events.hora) in 72..120
          assert located_events.ghatika != []
          assert located_events.tithi == []
          refute located_events.truncated

          for kind <- [:vaar, :hora, :ghatika] do
            located_events
            |> Map.fetch!(kind)
            |> Enum.chunk_every(2, 1, :discard)
            |> Enum.each(fn [a, b] -> assert Map.fetch!(a, :end) == b.start end)
          end

          # Hora indices cycle 0..23, including across Vedic-day rolls.
          located_events.hora
          |> Enum.chunk_every(2, 1, :discard)
          |> Enum.each(fn [a, b] -> assert b.hora_index == rem(a.hora_index + 1, 24) end)

          # Selecting a location-dependent element without a location fails.
          assert {:error, %CtaraDhruv.Error{kind: :search_error, message: message}} =
                   Panchang.events(engine, %{
                     from_utc: from_utc,
                     to_utc: %{from_utc | day: 18},
                     include_mask: [:vaar]
                   })

          assert message =~ "location required"
        else
          assert true
        end
    end
  end

  test "elixir panchang daily reuses known calendar elements" do
    case with_engine() do
      :skip ->
        assert true

      {:ok, engine} ->
        if File.exists?(@eop) do
          assert {:ok, _} = Engine.load_eop(engine, @eop)

          utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}
          mask = [:tithi, :masa, :ayana, :varsha]

          assert {:ok, base} = Panchang.daily(engine, %{utc: utc, include_mask: mask})
          assert is_map(base.masa)
          assert is_map(base.ayana)
          assert is_map(base.varsha)

          # A nearby date inside the validity windows echoes the fed-back
          # values verbatim instead of recomputing them.
          nearby = %{utc | day: 16}

          assert {:ok, reused} =
                   Panchang.daily(engine, %{
                     utc: nearby,
                     include_mask: mask,
                     known_masa: base.masa,
                     known_ayana: base.ayana,
                     known_varsha: base.varsha
                   })

          assert reused.masa == base.masa
          assert reused.ayana == base.ayana
          assert reused.varsha == base.varsha

          # The known value is echoed verbatim, not recomputed: a tagged copy
          # (flipped adhika) inside the window comes back with the tag intact.
          tagged_masa = %{base.masa | adhika: not base.masa.adhika}

          assert {:ok, tagged} =
                   Panchang.daily(engine, %{
                     utc: nearby,
                     include_mask: mask,
                     known_masa: tagged_masa
                   })

          assert tagged.masa == tagged_masa

          # Outside the masa window the stale value is ignored and recomputed.
          far = %{utc | month: 7}

          assert {:ok, recomputed} =
                   Panchang.daily(engine, %{
                     utc: far,
                     include_mask: mask,
                     known_masa: base.masa
                   })

          assert recomputed.masa != base.masa

          # Unknown enum names are rejected loudly, not silently dropped.
          assert {:error, %CtaraDhruv.Error{kind: :invalid_request}} =
                   Panchang.daily(engine, %{
                     utc: nearby,
                     include_mask: mask,
                     known_masa: %{base.masa | masa: "not_a_masa"}
                   })
        else
          assert true
        end
    end
  end

  test "elixir math exposes amsha variation catalogs" do
    assert {:ok, d2} = Math.amsha_variations(%{amsha_code: 2})
    assert d2.amsha_code == 2
    assert d2.default_variation_code == 0
    assert Enum.any?(d2.variations, &(&1.name == "cancer-leo-only" and &1.variation_code == 1))
    assert Enum.any?(d2.variations, &(&1.name == "lunar-hora" and &1.variation_code == 2))
    assert Enum.any?(d2.variations, &(&1.name == "kashinath-hora" and &1.variation_code == 3))

    assert {:ok, many} = Math.amsha_variations_many(%{amsha_codes: [2, 9]})
    assert length(many.catalogs) == 2
    assert Enum.at(many.catalogs, 1).amsha_code == 9
    assert Enum.at(Enum.at(many.catalogs, 1).variations, 0).is_default == true
  end
end
