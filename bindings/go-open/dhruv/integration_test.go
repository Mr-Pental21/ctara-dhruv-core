package dhruv

import (
	"math"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func repoRootFromTest(t *testing.T) string {
	t.Helper()
	_, file, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatalf("runtime.Caller failed")
	}
	return filepath.Clean(filepath.Join(filepath.Dir(file), "../../.."))
}

func kernelPaths(t *testing.T) (spk, lsk, eop string, ok bool) {
	t.Helper()
	root := repoRootFromTest(t)
	spk = filepath.Join(root, "kernels", "data", "de442s.bsp")
	lsk = filepath.Join(root, "kernels", "data", "naif0012.tls")
	eop = filepath.Join(root, "kernels", "data", "finals2000A.all")
	if !fileExists(spk) || !fileExists(lsk) || !fileExists(eop) {
		return "", "", "", false
	}
	return spk, lsk, eop, true
}

func fileExists(p string) bool {
	_, err := os.Stat(p)
	return err == nil
}

func TestABIVersion(t *testing.T) {
	if APIVersion() != ExpectedAPIVersion {
		t.Fatalf("ABI mismatch: got=%d want=%d", APIVersion(), ExpectedAPIVersion)
	}
}

func TestCalculateBhavaBalaNodeAspectFlag(t *testing.T) {
	var inputs BhavaBalaInputs
	inputs.AscendantSiderealLon = 0
	inputs.MeridianSiderealLon = 90
	inputs.BirthPeriod = 0
	inputs.AspectVirupas[4][0] = 40 // Guru full positive.
	inputs.AspectVirupas[7][0] = 20 // Rahu quarter-negative only when included.

	withoutNodes, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala without nodes: %v", err)
	}
	inputs.IncludeNodeAspects = true
	withNodes, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala with nodes: %v", err)
	}

	if math.Abs(withoutNodes.Entries[0].Drishti-40.0) > 1e-9 {
		t.Fatalf("unexpected no-node drishti: %v", withoutNodes.Entries[0].Drishti)
	}
	if math.Abs(withNodes.Entries[0].Drishti-35.0) > 1e-9 {
		t.Fatalf("unexpected with-node drishti: %v", withNodes.Entries[0].Drishti)
	}
}

func TestCalculateBhavaBalaDynamicChandraBuddhRules(t *testing.T) {
	var inputs BhavaBalaInputs
	inputs.ChandraBeneficRule = ChandraBeneficRuleBrightness72
	inputs.AspectVirupas[4][0] = 40 // Guru full positive.
	inputs.AspectVirupas[3][0] = 30 // Buddh full signed by dynamic nature.
	inputs.AspectVirupas[1][0] = 20 // Chandra quarter signed by selected rule.
	inputs.GrahaSiderealLons[1] = 10
	inputs.GrahaSiderealLons[2] = 60
	inputs.GrahaSiderealLons[3] = 60

	malefic, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala malefic dynamic rules: %v", err)
	}
	if math.Abs(malefic.Entries[0].Drishti-5.0) > 1e-9 {
		t.Fatalf("unexpected malefic drishti: %v", malefic.Entries[0].Drishti)
	}

	inputs.ChandraBeneficRule = ChandraBeneficRuleWaxing180
	inputs.GrahaSiderealLons[1] = 180
	inputs.GrahaSiderealLons[3] = 90
	benefic, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala benefic dynamic rules: %v", err)
	}
	if math.Abs(benefic.Entries[0].Drishti-75.0) > 1e-9 {
		t.Fatalf("unexpected benefic drishti: %v", benefic.Entries[0].Drishti)
	}
}

func TestCalculateBhavaBalaSpecialRulesFlag(t *testing.T) {
	var inputs BhavaBalaInputs
	inputs.CuspSiderealLons[0] = 65
	inputs.AscendantSiderealLon = 15
	inputs.MeridianSiderealLon = 105
	inputs.BirthPeriod = 0
	inputs.GrahaBhavaNumbers[4] = 1
	inputs.HouseLordStrengths[0] = 300

	withoutSpecial, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala without special rules: %v", err)
	}
	inputs.IncludeSpecialRules = true
	withSpecial, err := CalculateBhavaBala(inputs)
	if err != nil {
		t.Fatalf("CalculateBhavaBala with special rules: %v", err)
	}

	if math.Abs(withSpecial.Entries[0].OccupationBonus-60.0) > 1e-9 {
		t.Fatalf("unexpected occupation bonus: %v", withSpecial.Entries[0].OccupationBonus)
	}
	if math.Abs(withSpecial.Entries[0].RisingBonus-15.0) > 1e-9 {
		t.Fatalf("unexpected rising bonus: %v", withSpecial.Entries[0].RisingBonus)
	}
	if math.Abs((withSpecial.Entries[0].TotalVirupas-withoutSpecial.Entries[0].TotalVirupas)-75.0) > 1e-9 {
		t.Fatalf("unexpected special total delta: with=%v without=%v", withSpecial.Entries[0].TotalVirupas, withoutSpecial.Entries[0].TotalVirupas)
	}
}

func TestKshetraSphutaMatchesAllSphutas(t *testing.T) {
	inputs := SphutalInputs{
		Sun: 10, Moon: 20, Mars: 30, Jupiter: 40, Venus: 50,
		Rahu: 60, Lagna: 70, EighthLord: 80, Gulika: 90,
	}
	all, err := AllSphutas(inputs)
	if err != nil {
		t.Fatalf("AllSphutas: %v", err)
	}
	scalar := KshetraSphuta(inputs.Moon, inputs.Mars, inputs.Jupiter, inputs.Venus, inputs.Lagna)

	// ALL_SPHUTAS order in dhruv_vedic_base: KshetraSphuta is index 8.
	kshetraIdx := 8
	if math.Abs(scalar-all.Longitudes[kshetraIdx]) > 1e-9 {
		t.Fatalf("kshetra mismatch: scalar=%v all[%d]=%v", scalar, kshetraIdx, all.Longitudes[kshetraIdx])
	}
}

func TestHelperAndTaraPrimitives(t *testing.T) {
	if HoraLord(0, 0) != 0 {
		t.Fatalf("HoraLord(sunday,0) = %d, want 0", HoraLord(0, 0))
	}

	has, exalt, err := ExaltationDegree(0)
	if err != nil {
		t.Fatalf("ExaltationDegree: %v", err)
	}
	if !has || math.Abs(exalt-10.0) > 1e-9 {
		t.Fatalf("unexpected exaltation degree: has=%v value=%v", has, exalt)
	}

	relationship, err := NaisargikaMaitri(0, 1)
	if err != nil {
		t.Fatalf("NaisargikaMaitri: %v", err)
	}
	if relationship != NaisargikaFriend {
		t.Fatalf("unexpected naisargika relationship: got=%d", relationship)
	}

	position, err := TaraPropagatePosition(10.0, 20.0, 10.0, 0.0, 0.0, 0.0, 0.0)
	if err != nil {
		t.Fatalf("TaraPropagatePosition: %v", err)
	}
	if math.Abs(position.RADeg-10.0) > 1e-9 || math.Abs(position.DecDeg-20.0) > 1e-9 {
		t.Fatalf("unexpected propagated position: %+v", position)
	}

	dir, err := TaraGalacticAnticenterICRS()
	if err != nil {
		t.Fatalf("TaraGalacticAnticenterICRS: %v", err)
	}
	norm := math.Sqrt(dir[0]*dir[0] + dir[1]*dir[1] + dir[2]*dir[2])
	if math.Abs(norm-1.0) > 1e-9 {
		t.Fatalf("anticenter vector not normalized: %v", dir)
	}
}

func TestLoadConfigSupportsDiscoveryOptions(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "config.toml")
	if err := os.WriteFile(configPath, []byte("version = 1\n"), 0o644); err != nil {
		t.Fatalf("WriteFile: %v", err)
	}
	t.Setenv("DHRUV_CONFIG_FILE", configPath)
	cfg, err := LoadConfig(ConfigLoadOptionsDefault())
	if err != nil {
		t.Fatalf("LoadConfig: %v", err)
	}
	defer cfg.Close()
	if err := ClearActiveConfig(); err != nil {
		t.Fatalf("ClearActiveConfig: %v", err)
	}
}

func TestEngineQueryAndTimeRoundTrip(t *testing.T) {
	spk, lskPath, _, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}

	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	defer eng.Close()

	q := QueryRequest{
		Target:     301,
		Observer:   399,
		Frame:      1,
		TimeKind:   QueryTimeJDTDB,
		EpochTdbJD: 2451545.0,
		OutputMode: QueryOutputCartesian,
	}
	result, err := eng.Query(q)
	if err != nil {
		t.Fatalf("Query: %v", err)
	}
	if result.State == nil {
		t.Fatalf("expected cartesian state in query result")
	}
	sv := result.State
	if math.IsNaN(sv.PositionKm[0]) || math.IsInf(sv.PositionKm[0], 0) {
		t.Fatalf("invalid state vector output: %+v", sv)
	}

	lsk, err := LoadLSK(lskPath)
	if err != nil {
		t.Fatalf("LoadLSK: %v", err)
	}
	defer lsk.Close()

	utc := UtcTime{Year: 2025, Month: 1, Day: 1, Hour: 0, Minute: 0, Second: 0}
	resultTime, err := UTCToTdbJD(lsk, nil, UtcToTdbRequest{
		UTC: utc,
		Policy: TimePolicy{
			Mode: TimePolicyHybridDeltaT,
			Options: TimeConversionOptions{
				WarnOnFallback:         true,
				DeltaTModel:            DeltaTModelSmh2016WithPre720Quadratic,
				FreezeFutureDut1:       true,
				PreRangeDut1:           0.0,
				FutureDeltaTTransition: FutureDeltaTTransitionLegacyTtUtcBlend,
				FutureTransitionYears:  100.0,
				SmhFutureFamily:        SmhFutureFamilyAddendum2020Piecewise,
			},
		},
	})
	if err != nil {
		t.Fatalf("UTCToTdbJD: %v", err)
	}
	back, err := JdTdbToUTC(lsk, resultTime.JdTdb)
	if err != nil {
		t.Fatalf("JdTdbToUTC: %v", err)
	}
	if back.Year != utc.Year || back.Month != utc.Month || back.Day != utc.Day {
		t.Fatalf("roundtrip mismatch: got=%+v want=%+v", back, utc)
	}
}

func TestEngineReplaceAndListSPKs(t *testing.T) {
	spk, lskPath, _, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}

	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	defer eng.Close()

	initial, err := eng.ListSPKs()
	if err != nil {
		t.Fatalf("ListSPKs initial: %v", err)
	}
	if len(initial) != 1 || initial[0].Generation != 0 {
		t.Fatalf("unexpected initial SPK list: %+v", initial)
	}

	report, err := eng.ReplaceSPKs([]string{spk, spk})
	if err != nil {
		t.Fatalf("ReplaceSPKs: %v", err)
	}
	if report.Generation != 1 || report.ActiveCount != 2 || report.LoadedCount != 0 || report.ReusedCount != 2 {
		t.Fatalf("unexpected replace report: %+v", report)
	}

	list, err := eng.ListSPKs()
	if err != nil {
		t.Fatalf("ListSPKs after replace: %v", err)
	}
	if len(list) != 2 || list[0].Generation != report.Generation || list[1].Generation != report.Generation {
		t.Fatalf("unexpected replacement SPK list: %+v", list)
	}

	if _, err := eng.ReplaceSPKs([]string{filepath.Join(filepath.Dir(spk), "missing.bsp")}); err == nil {
		t.Fatalf("expected missing SPK replacement to fail")
	}
	afterFailure, err := eng.ListSPKs()
	if err != nil {
		t.Fatalf("ListSPKs after failure: %v", err)
	}
	if len(afterFailure) != 2 || afterFailure[0].Generation != report.Generation {
		t.Fatalf("failed replacement changed active SPKs: %+v", afterFailure)
	}
}

func TestSearchAndPanchangSmoke(t *testing.T) {
	spk, lskPath, eopPath, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}

	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	defer eng.Close()

	eop, err := LoadEOP(eopPath)
	if err != nil {
		t.Fatalf("LoadEOP: %v", err)
	}
	defer eop.Close()

	req := LunarPhaseSearchRequest{
		PhaseKind: 1,
		QueryMode: 2,
		StartUTC:  UtcTime{Year: 2000, Month: 1, Day: 1, Hour: 12, Minute: 0, Second: 0},
		EndUTC:    UtcTime{Year: 2000, Month: 12, Day: 31, Hour: 12, Minute: 0, Second: 0},
	}
	_, found, events, err := eng.LunarPhaseSearch(req, 1)
	if err != nil {
		t.Fatalf("LunarPhaseSearch: %v", err)
	}
	if found {
		t.Fatalf("range LunarPhaseSearch should not report single-event found=true")
	}
	if len(events) < 12 {
		t.Fatalf("expected auto-expanded lunar phase range results, got %d", len(events))
	}

	utc := UtcTime{Year: 2025, Month: 1, Day: 15, Hour: 12, Minute: 0, Second: 0}
	if _, err := eng.TithiForDate(utc); err != nil {
		t.Fatalf("TithiForDate: %v", err)
	}

	loc := GeoLocation{LatitudeDeg: 12.9716, LongitudeDeg: 77.5946, AltitudeM: 920}
	if _, err := eng.VaarForDate(eop, utc, loc, RiseSetConfigDefault()); err != nil {
		t.Fatalf("VaarForDate: %v", err)
	}

	lsk, err := LoadLSK(lskPath)
	if err != nil {
		t.Fatalf("LoadLSK: %v", err)
	}
	defer lsk.Close()

	panchangReq := PanchangComputeRequest{
		TimeKind:        QueryTimeUTC,
		UTC:             utc,
		IncludeMask:     PanchangIncludeLocationIndependent,
		SankrantiConfig: SankrantiConfigDefault(),
	}
	panchang, err := eng.PanchangComputeEx(eop, lsk, panchangReq)
	if err != nil {
		t.Fatalf("PanchangComputeEx without location: %v", err)
	}
	if !panchang.TithiValid || !panchang.MasaValid {
		t.Fatalf("expected location-independent elements without location")
	}
	if panchang.VaarValid || panchang.HoraValid || panchang.GhatikaValid {
		t.Fatalf("location-dependent elements must stay invalid without location")
	}

	panchangReq.IncludeMask = PanchangIncludeLocationDependent
	if _, err := eng.PanchangComputeEx(eop, lsk, panchangReq); err == nil {
		t.Fatalf("expected error for location-dependent mask without location")
	}
	panchangReq.HasLocation = true
	panchangReq.Location = loc
	panchangReq.RiseSetConfig = RiseSetConfigDefault()
	panchang, err = eng.PanchangComputeEx(eop, lsk, panchangReq)
	if err != nil {
		t.Fatalf("PanchangComputeEx with location: %v", err)
	}
	if !panchang.VaarValid || !panchang.HoraValid || !panchang.GhatikaValid {
		t.Fatalf("expected location-dependent elements with location")
	}

	// Known calendar values are reused verbatim inside their validity window
	// and silently recomputed when stale.
	calReq := PanchangComputeRequest{
		TimeKind:        QueryTimeUTC,
		UTC:             UtcTime{Year: 2024, Month: 1, Day: 15, Hour: 12},
		IncludeMask:     PanchangIncludeAllCalendar,
		SankrantiConfig: SankrantiConfigDefault(),
	}
	first, err := eng.PanchangComputeEx(eop, lsk, calReq)
	if err != nil {
		t.Fatalf("PanchangComputeEx calendar: %v", err)
	}
	if !first.MasaValid || !first.AyanaValid || !first.VarshaValid {
		t.Fatalf("expected calendar elements in calendar-mask result")
	}
	calReq.UTC = UtcTime{Year: 2024, Month: 1, Day: 20, Hour: 12}
	calReq.KnownMasa = &first.Masa
	calReq.KnownAyana = &first.Ayana
	calReq.KnownVarsha = &first.Varsha
	reused, err := eng.PanchangComputeEx(eop, lsk, calReq)
	if err != nil {
		t.Fatalf("PanchangComputeEx with known calendar values: %v", err)
	}
	if reused.Masa != first.Masa || reused.Ayana != first.Ayana || reused.Varsha != first.Varsha {
		t.Fatalf("known calendar values must be reused verbatim inside their windows")
	}
	calReq.UTC = UtcTime{Year: 2024, Month: 5, Day: 20, Hour: 12}
	recomputed, err := eng.PanchangComputeEx(eop, lsk, calReq)
	if err != nil {
		t.Fatalf("PanchangComputeEx with stale known calendar values: %v", err)
	}
	if !recomputed.MasaValid {
		t.Fatalf("expected recomputed masa for stale known value")
	}
	if recomputed.Masa == first.Masa {
		t.Fatalf("stale known masa must be recomputed, not echoed")
	}

	bhava := BhavaConfigDefault()
	if !bhava.UseRashiBhavaForBalaAvastha {
		t.Fatalf("expected rashi-bhava bala/avastha default on")
	}
	if bhava.IncludeNodeAspectsForDrikBala {
		t.Fatalf("expected Rahu/Ketu Drik Bala aspects default off")
	}
	if !bhava.DivideGuruBuddhDrishtiBy4ForDrikBala {
		t.Fatalf("expected Guru/Buddh Drik Bala divisor default on")
	}
	if bhava.ChandraBeneficRule != ChandraBeneficRuleBrightness72 {
		t.Fatalf("expected Chandra benefic rule default brightness-72")
	}
	if !bhava.IncludeRashiBhavaResults {
		t.Fatalf("expected rashi-bhava result default on")
	}
	riseset := RiseSetConfigDefault()
	if _, err := eng.ShadbalaForDate(
		eop,
		utc,
		loc,
		bhava,
		riseset,
		0,
		true,
		AmshaSelectionConfig{},
	); err != nil {
		t.Fatalf("ShadbalaForDate: %v", err)
	}
	if _, err := eng.CharakarakaForDate(eop, utc, 0, true, CharakarakaSchemeMixedParashara); err != nil {
		t.Fatalf("CharakarakaForDate: %v", err)
	}

	if _, err := eng.FullKundaliForDateSummary(eop, utc, loc, bhava, riseset, 0, true); err != nil {
		t.Fatalf("FullKundaliForDateSummary: %v", err)
	}

	cfg := FullKundaliConfigDefault()
	cfg.PanchangIncludeMask = PanchangIncludeAll
	cfg.IncludeDasha = true
	cfg.DashaConfig.Count = 2
	cfg.DashaConfig.Systems[0] = 0
	cfg.DashaConfig.Systems[1] = 1
	cfg.DashaConfig.MaxLevels[0] = 0
	cfg.DashaConfig.MaxLevels[1] = 1
	kundali, err := eng.FullKundaliForDate(eop, utc, loc, bhava, riseset, 0, true, cfg)
	if err != nil {
		t.Fatalf("FullKundaliForDate: %v", err)
	}
	if kundali.Sphutas == nil || len(kundali.Sphutas.Longitudes) != SphutaCount {
		t.Fatalf("expected root sphutas in full kundali")
	}
	if kundali.Panchang == nil {
		t.Fatalf("expected panchang section in full kundali")
	}
	if !kundali.Panchang.TithiValid || !kundali.Panchang.VaarValid || !kundali.Panchang.MasaValid {
		t.Fatalf("expected all panchang elements valid in full kundali: %+v", kundali.Panchang)
	}
	if len(kundali.Dasha) != 2 {
		t.Fatalf("expected 2 dasha hierarchies, got %d", len(kundali.Dasha))
	}
	if len(kundali.Dasha[0].Levels) != 1 || len(kundali.Dasha[1].Levels) != 2 {
		t.Fatalf("unexpected per-system dasha depths: %d %d", len(kundali.Dasha[0].Levels), len(kundali.Dasha[1].Levels))
	}

	amshaScope := AmshaChartScope{
		IncludeBhavaCusps:    true,
		IncludeArudhaPadas:   true,
		IncludeUpagrahas:     true,
		IncludeSphutas:       true,
		IncludeSpecialLagnas: true,
		IncludeOuterPlanets:  true,
	}
	amshaChart, err := eng.AmshaChartForDate(eop, utc, loc, bhava, riseset, 0, true, 9, 0, amshaScope)
	if err != nil {
		t.Fatalf("AmshaChartForDate: %v", err)
	}
	if len(amshaChart.BhavaCusps) != 12 || len(amshaChart.ArudhaPadas) != 12 {
		t.Fatalf("expected bhava/arudha amsha sections, got %d/%d", len(amshaChart.BhavaCusps), len(amshaChart.ArudhaPadas))
	}
	if len(amshaChart.Upagrahas) != 11 || len(amshaChart.Sphutas) != SphutaCount || len(amshaChart.SpecialLagnas) != 8 {
		t.Fatalf(
			"expected upagraha/sphuta/special-lagna amsha sections, got %d/%d/%d",
			len(amshaChart.Upagrahas), len(amshaChart.Sphutas), len(amshaChart.SpecialLagnas),
		)
	}
	if !amshaChart.OuterPlanetsValid || len(amshaChart.OuterPlanets) != 3 {
		t.Fatalf("expected 3 outer planets in amsha chart")
	}

	amshaCfg := FullKundaliConfigDefault()
	amshaCfg.IncludeBhavaCusps = true
	amshaCfg.IncludeBindus = true
	amshaCfg.IncludeUpagrahas = true
	amshaCfg.IncludeSphutas = true
	amshaCfg.IncludeSpecialLagnas = true
	amshaCfg.IncludeAmshas = true
	amshaCfg.AmshaScope = amshaScope
	amshaCfg.AmshaSelection.Count = 1
	amshaCfg.AmshaSelection.Codes[0] = 9
	amshaCfg.AmshaSelection.Variations[0] = 0
	kundaliWithAmshas, err := eng.FullKundaliForDate(eop, utc, loc, bhava, riseset, 0, true, amshaCfg)
	if err != nil {
		t.Fatalf("FullKundaliForDate (amshas): %v", err)
	}
	if len(kundaliWithAmshas.Amshas) != 1 {
		t.Fatalf("expected 1 amsha chart, got %d", len(kundaliWithAmshas.Amshas))
	}
	if len(kundaliWithAmshas.Amshas[0].Sphutas) != SphutaCount {
		t.Fatalf("expected scoped amsha sphutas in full kundali, got %d", len(kundaliWithAmshas.Amshas[0].Sphutas))
	}

	gpCfg := GrahaPositionsConfig{
		IncludeLagna:        true,
		IncludeOuterPlanets: true,
		IncludeEquatorial:   true,
	}
	gp, err := eng.GrahaPositionsForDate(eop, utc, loc, bhava, 0, true, gpCfg)
	if err != nil {
		t.Fatalf("GrahaPositionsForDate (equatorial): %v", err)
	}
	for i, entry := range gp.Grahas {
		if !entry.EquatorialValid {
			t.Fatalf("graha %d: expected EquatorialValid", i)
		}
		if entry.RightAscensionDeg < 0 || entry.RightAscensionDeg >= 360 {
			t.Fatalf("graha %d: RA out of [0,360): %v", i, entry.RightAscensionDeg)
		}
		if math.Abs(entry.DeclinationDeg) > 90 {
			t.Fatalf("graha %d: declination out of [-90,90]: %v", i, entry.DeclinationDeg)
		}
	}
	// Rahu (7) and Ketu (8) are ecliptic points; latitude must be exactly 0.
	if gp.Grahas[7].EclipticLatitudeDeg != 0 || gp.Grahas[8].EclipticLatitudeDeg != 0 {
		t.Fatalf(
			"expected zero node ecliptic latitude, got %v/%v",
			gp.Grahas[7].EclipticLatitudeDeg, gp.Grahas[8].EclipticLatitudeDeg,
		)
	}
	if !gp.EarthOrientationValid {
		t.Fatalf("expected EarthOrientationValid with IncludeEquatorial")
	}
	if gp.GmstDeg < 0 || gp.GmstDeg >= 360 {
		t.Fatalf("GMST out of [0,360): %v", gp.GmstDeg)
	}

	// Series: 2h at 60-minute cadence yields 3 points; the first must match
	// the single-epoch call exactly.
	toUTC := utc
	toUTC.Hour += 2
	series, err := eng.GrahaPositionsSeriesForDate(eop, utc, toUTC, 60, loc, bhava, 0, true, gpCfg)
	if err != nil {
		t.Fatalf("GrahaPositionsSeriesForDate: %v", err)
	}
	if len(series) != 3 {
		t.Fatalf("expected 3 series points, got %d", len(series))
	}
	for i := range gp.Grahas {
		if series[0].Positions.Grahas[i].SiderealLongitude != gp.Grahas[i].SiderealLongitude {
			t.Fatalf("series point 0 graha %d longitude mismatch", i)
		}
	}
	if _, err := eng.GrahaPositionsSeriesForDate(eop, utc, toUTC, 0, loc, bhava, 0, true, gpCfg); err == nil {
		t.Fatalf("expected error for stepMinutes == 0")
	}
}

// utcJD converts a UtcTime to an approximate JD (UTC) for ordering and
// closeness assertions in tests.
func utcJD(u UtcTime) float64 {
	y, m := int(u.Year), int(u.Month)
	a := (14 - m) / 12
	yy := y + 4800 - a
	mm := m + 12*a - 3
	jdn := int(u.Day) + (153*mm+2)/5 + 365*yy + yy/4 - yy/100 + yy/400 - 32045
	return float64(jdn) - 0.5 + (float64(u.Hour)+(float64(u.Minute)+u.Second/60.0)/60.0)/24.0
}

func angularSepDeg(a, b float64) float64 {
	d := math.Mod(math.Abs(a-b), 360)
	if d > 180 {
		d = 360 - d
	}
	return d
}

func newRangeOpsFixtures(t *testing.T) (*Engine, *EOP) {
	t.Helper()
	spk, lskPath, eopPath, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}
	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	t.Cleanup(func() { eng.Close() })
	eop, err := LoadEOP(eopPath)
	if err != nil {
		t.Fatalf("LoadEOP: %v", err)
	}
	t.Cleanup(func() { eop.Close() })
	return eng, eop
}

func TestAmshaSeriesMatchesSingleEpochChart(t *testing.T) {
	eng, eop := newRangeOpsFixtures(t)

	from := UtcTime{Year: 1990, Month: 1, Day: 15, Hour: 6, Minute: 30, Second: 0}
	to := from
	to.Hour += 2
	loc := GeoLocation{LatitudeDeg: 28.6139, LongitudeDeg: 77.2090, AltitudeM: 0}
	sank := SankrantiConfigDefault()

	// D9, D1, duplicate D9: charts come back in request order, duplicates
	// repeated.
	requests := []AmshaRequest{{AmshaCode: 9}, {AmshaCode: 1}, {AmshaCode: 9}}
	points, err := eng.AmshaSeries(eop, from, to, 60, loc, sank, requests, true)
	if err != nil {
		t.Fatalf("AmshaSeries: %v", err)
	}
	if len(points) != 3 {
		t.Fatalf("expected 3 series points over 2h at 60min step, got %d", len(points))
	}
	if math.Abs(points[0].JdUtc-utcJD(from)) > 1e-5 {
		t.Fatalf("first point JD %v does not match from %v", points[0].JdUtc, utcJD(from))
	}
	for i, pt := range points {
		if len(pt.Charts) != 3 {
			t.Fatalf("point %d: expected 3 charts, got %d", i, len(pt.Charts))
		}
		if pt.Charts[0].AmshaCode != 9 || pt.Charts[1].AmshaCode != 1 || pt.Charts[2].AmshaCode != 9 {
			t.Fatalf("point %d: charts not in request order: %d/%d/%d",
				i, pt.Charts[0].AmshaCode, pt.Charts[1].AmshaCode, pt.Charts[2].AmshaCode)
		}
		if pt.Charts[0].Lagna.SiderealLongitude != pt.Charts[2].Lagna.SiderealLongitude {
			t.Fatalf("point %d: duplicate D9 charts differ", i)
		}
		if !pt.Charts[0].GrahasValid {
			t.Fatalf("point %d: expected GrahasValid with includeGrahas", i)
		}
	}

	// The first-epoch D9 chart must match the single-epoch op.
	single, err := eng.AmshaChartForDate(
		eop, from, loc, BhavaConfigDefault(), RiseSetConfigDefault(),
		uint32(sank.AyanamshaSystem), sank.UseNutation, 9, 0, AmshaChartScope{},
	)
	if err != nil {
		t.Fatalf("AmshaChartForDate: %v", err)
	}
	chart0 := points[0].Charts[0]
	if sep := angularSepDeg(chart0.Lagna.SiderealLongitude, single.Lagna.SiderealLongitude); sep > 1e-6 {
		t.Fatalf("series lagna %v vs single-epoch lagna %v (sep %v)",
			chart0.Lagna.SiderealLongitude, single.Lagna.SiderealLongitude, sep)
	}
	for g := 0; g < GrahaCount; g++ {
		if sep := angularSepDeg(chart0.Grahas[g].SiderealLongitude, single.Grahas[g].SiderealLongitude); sep > 1e-6 {
			t.Fatalf("graha %d: series %v vs single %v (sep %v)",
				g, chart0.Grahas[g].SiderealLongitude, single.Grahas[g].SiderealLongitude, sep)
		}
	}

	// Invalid requests are rejected.
	if _, err := eng.AmshaSeries(eop, from, to, 0, loc, sank, requests, true); err == nil {
		t.Fatalf("expected error for stepMinutes == 0")
	}
	if _, err := eng.AmshaSeries(eop, from, to, 60, loc, sank, nil, true); err == nil {
		t.Fatalf("expected error for empty request list")
	}
	if _, err := eng.AmshaSeries(eop, to, from, 60, loc, sank, requests, true); err == nil {
		t.Fatalf("expected error for reversed range")
	}
}

func TestPanchangEventsTithiChainingAndResume(t *testing.T) {
	eng, eop := newRangeOpsFixtures(t)

	from := UtcTime{Year: 2024, Month: 1, Day: 1}
	to := UtcTime{Year: 2024, Month: 2, Day: 5}
	sank := SankrantiConfigDefault()

	full, err := eng.PanchangEvents(eop, from, to, PanchangIncludeTithi, nil, nil, sank, 0)
	if err != nil {
		t.Fatalf("PanchangEvents: %v", err)
	}
	// ~35 days of tithis (mean tithi length is a bit under one day).
	if len(full.Tithis) < 34 || len(full.Tithis) > 40 {
		t.Fatalf("unexpected tithi count over 35 days: %d", len(full.Tithis))
	}
	for i, ti := range full.Tithis {
		if ti.TithiIndex < 0 || ti.TithiIndex >= 30 {
			t.Fatalf("tithi %d: index out of range: %d", i, ti.TithiIndex)
		}
		if i > 0 {
			prev := full.Tithis[i-1]
			if prev.End != ti.Start {
				t.Fatalf("tithi %d does not chain: prev.End=%+v start=%+v", i, prev.End, ti.Start)
			}
			if ti.TithiIndex != (prev.TithiIndex+1)%30 {
				t.Fatalf("tithi %d index not consecutive: prev=%d cur=%d", i, prev.TithiIndex, ti.TithiIndex)
			}
		}
	}
	if utcJD(full.Tithis[0].Start) > utcJD(from)+1e-5 {
		t.Fatalf("first tithi must start at or before from: %+v", full.Tithis[0].Start)
	}
	if utcJD(full.Tithis[len(full.Tithis)-1].End) < utcJD(to)-1e-5 {
		t.Fatalf("last tithi must end at or after to: %+v", full.Tithis[len(full.Tithis)-1].End)
	}
	if len(full.Karanas) != 0 || len(full.Masas) != 0 || len(full.Varshas) != 0 {
		t.Fatalf("unselected kinds must be empty")
	}
	if full.Truncated || full.NextFromUTC != nil {
		t.Fatalf("full sweep must not be truncated: truncated=%v next=%v", full.Truncated, full.NextFromUTC)
	}

	// A tiny event budget truncates and yields a resume point; resuming from
	// NextFromUTC with dedup on Start reproduces the full sweep.
	small, err := eng.PanchangEvents(eop, from, to, PanchangIncludeTithi, nil, nil, sank, 5)
	if err != nil {
		t.Fatalf("PanchangEvents truncated: %v", err)
	}
	if len(small.Tithis) != 5 {
		t.Fatalf("expected 5 tithis under maxEvents=5, got %d", len(small.Tithis))
	}
	if !small.Truncated || small.NextFromUTC == nil {
		t.Fatalf("expected truncation metadata: truncated=%v next=%v", small.Truncated, small.NextFromUTC)
	}
	if utcJD(*small.NextFromUTC) <= utcJD(from) {
		t.Fatalf("resume point must be after from: %+v", *small.NextFromUTC)
	}
	resumed, err := eng.PanchangEvents(eop, *small.NextFromUTC, to, PanchangIncludeTithi, nil, nil, sank, 0)
	if err != nil {
		t.Fatalf("PanchangEvents resumed: %v", err)
	}
	// Dedup on (kind, start) with a one-second tolerance: separate sweeps
	// re-solve segment boundaries, so starts agree to well under a second
	// but not bit-exactly.
	const tolDays = 1.0 / 86400.0
	var seenStarts []float64
	seen := func(jd float64) bool {
		for _, s := range seenStarts {
			if math.Abs(s-jd) < tolDays {
				return true
			}
		}
		return false
	}
	var combined []TithiInfo
	for _, ti := range small.Tithis {
		seenStarts = append(seenStarts, utcJD(ti.Start))
		combined = append(combined, ti)
	}
	for _, ti := range resumed.Tithis {
		if seen(utcJD(ti.Start)) {
			continue
		}
		seenStarts = append(seenStarts, utcJD(ti.Start))
		combined = append(combined, ti)
	}
	if len(combined) != len(full.Tithis) {
		t.Fatalf("resumed+deduped sweep has %d tithis, full sweep has %d", len(combined), len(full.Tithis))
	}
	for i := range combined {
		want := full.Tithis[i]
		if combined[i].TithiIndex != want.TithiIndex ||
			math.Abs(utcJD(combined[i].Start)-utcJD(want.Start)) > tolDays ||
			math.Abs(utcJD(combined[i].End)-utcJD(want.End)) > tolDays {
			t.Fatalf("resumed tithi %d mismatch: %+v vs %+v", i, combined[i], want)
		}
	}

	// Invalid masks are rejected: zero, and location-dependent bits without
	// a location.
	if _, err := eng.PanchangEvents(eop, from, to, 0, nil, nil, sank, 0); err == nil {
		t.Fatalf("expected error for zero include mask")
	}
	if _, err := eng.PanchangEvents(eop, from, to, PanchangIncludeVaar, nil, nil, sank, 0); err == nil {
		t.Fatalf("expected error for location-dependent include mask without location")
	}
	if _, err := eng.PanchangEvents(eop, from, to, PanchangIncludeTithi|PanchangIncludeHora, nil, nil, sank, 0); err == nil {
		t.Fatalf("expected error for mixed location-dependent include mask without location")
	}
}

func TestPanchangEventsLocationDependent(t *testing.T) {
	eng, eop := newRangeOpsFixtures(t)

	from := UtcTime{Year: 2024, Month: 1, Day: 1}
	to := UtcTime{Year: 2024, Month: 1, Day: 4}
	loc := GeoLocation{LatitudeDeg: 28.6139, LongitudeDeg: 77.2090, AltitudeM: 0}
	sank := SankrantiConfigDefault()

	res, err := eng.PanchangEvents(eop, from, to, PanchangIncludeVaar|PanchangIncludeHora, &loc, nil, sank, 0)
	if err != nil {
		t.Fatalf("PanchangEvents with location: %v", err)
	}

	// Three days cover 3-5 sunrise-to-sunrise Vedic days and their 24
	// horas each.
	if len(res.Vaars) < 3 || len(res.Vaars) > 5 {
		t.Fatalf("unexpected vaar count over 3 days: %d", len(res.Vaars))
	}
	if len(res.Horas) < 72 || len(res.Horas) > 120 {
		t.Fatalf("unexpected hora count over 3 days: %d", len(res.Horas))
	}
	if len(res.Ghatikas) != 0 || len(res.Tithis) != 0 {
		t.Fatalf("unselected kinds must be empty")
	}

	// Vaar segments chain exactly and advance the weekday cyclically.
	for i, v := range res.Vaars {
		if v.VaarIndex < 0 || v.VaarIndex >= 7 {
			t.Fatalf("vaar %d: index out of range: %d", i, v.VaarIndex)
		}
		if i > 0 {
			prev := res.Vaars[i-1]
			if prev.End != v.Start {
				t.Fatalf("vaar %d does not chain: prev.End=%+v start=%+v", i, prev.End, v.Start)
			}
			if v.VaarIndex != (prev.VaarIndex+1)%7 {
				t.Fatalf("vaar %d weekday not consecutive: prev=%d cur=%d", i, prev.VaarIndex, v.VaarIndex)
			}
		}
	}
	if utcJD(res.Vaars[0].Start) > utcJD(from)+1e-5 {
		t.Fatalf("first vaar must start at or before from: %+v", res.Vaars[0].Start)
	}
	if utcJD(res.Vaars[len(res.Vaars)-1].End) < utcJD(to)-1e-5 {
		t.Fatalf("last vaar must end at or after to: %+v", res.Vaars[len(res.Vaars)-1].End)
	}

	// Hora segments chain exactly, including across Vedic-day rolls, and
	// the lord cycles through the Chaldean sequence while the position
	// cycles 0..23.
	for i, h := range res.Horas {
		if h.HoraIndex < 0 || h.HoraIndex >= 7 {
			t.Fatalf("hora %d: lord index out of range: %d", i, h.HoraIndex)
		}
		if h.HoraPosition < 0 || h.HoraPosition >= 24 {
			t.Fatalf("hora %d: position out of range: %d", i, h.HoraPosition)
		}
		if i > 0 {
			prev := res.Horas[i-1]
			if prev.End != h.Start {
				t.Fatalf("hora %d does not chain: prev.End=%+v start=%+v", i, prev.End, h.Start)
			}
			if h.HoraIndex != (prev.HoraIndex+1)%7 {
				t.Fatalf("hora %d lord not cycling: prev=%d cur=%d", i, prev.HoraIndex, h.HoraIndex)
			}
			if h.HoraPosition != (prev.HoraPosition+1)%24 {
				t.Fatalf("hora %d position not cycling: prev=%d cur=%d", i, prev.HoraPosition, h.HoraPosition)
			}
		}
	}

	// The hora stream tiles each Vedic day: day rolls align with vaar
	// boundaries.
	for i, h := range res.Horas {
		if h.HoraPosition == 0 && i > 0 {
			found := false
			for _, v := range res.Vaars {
				if v.Start == h.Start {
					found = true
					break
				}
			}
			if !found {
				t.Fatalf("hora %d day roll at %+v does not match any vaar start", i, h.Start)
			}
		}
	}

	// An explicit rise/set config equal to the defaults reproduces the
	// default-config sweep.
	riseCfg := RiseSetConfigDefault()
	res2, err := eng.PanchangEvents(eop, from, to, PanchangIncludeVaar|PanchangIncludeHora, &loc, &riseCfg, sank, 0)
	if err != nil {
		t.Fatalf("PanchangEvents with explicit riseset config: %v", err)
	}
	if len(res2.Vaars) != len(res.Vaars) || len(res2.Horas) != len(res.Horas) {
		t.Fatalf("explicit default riseset config changed results: vaars %d vs %d, horas %d vs %d",
			len(res2.Vaars), len(res.Vaars), len(res2.Horas), len(res.Horas))
	}

	// Ghatikas: 60 per Vedic day, chaining with values cycling 1..60.
	gh, err := eng.PanchangEvents(eop, from, UtcTime{Year: 2024, Month: 1, Day: 2}, PanchangIncludeGhatika, &loc, nil, sank, 0)
	if err != nil {
		t.Fatalf("PanchangEvents ghatika: %v", err)
	}
	if len(gh.Ghatikas) < 60 || len(gh.Ghatikas) > 120 {
		t.Fatalf("unexpected ghatika count over 1 day: %d", len(gh.Ghatikas))
	}
	for i, g := range gh.Ghatikas {
		if g.Value < 1 || g.Value > 60 {
			t.Fatalf("ghatika %d: value out of range: %d", i, g.Value)
		}
		if i > 0 {
			prev := gh.Ghatikas[i-1]
			if prev.End != g.Start {
				t.Fatalf("ghatika %d does not chain: prev.End=%+v start=%+v", i, prev.End, g.Start)
			}
			if g.Value != prev.Value%60+1 {
				t.Fatalf("ghatika %d value not cycling: prev=%d cur=%d", i, prev.Value, g.Value)
			}
		}
	}
}

func TestAmshaLagnaEventsD1Chaining(t *testing.T) {
	eng, eop := newRangeOpsFixtures(t)

	from := UtcTime{Year: 2024, Month: 1, Day: 15}
	to := UtcTime{Year: 2024, Month: 1, Day: 16}
	loc := GeoLocation{LatitudeDeg: 28.6139, LongitudeDeg: 77.2090, AltitudeM: 0}
	sank := SankrantiConfigDefault()

	// Duplicate D1 requests collapse into one entry.
	res, err := eng.AmshaLagnaEvents(eop, from, to, loc, sank, []AmshaRequest{{AmshaCode: 1}, {AmshaCode: 1}}, 0)
	if err != nil {
		t.Fatalf("AmshaLagnaEvents: %v", err)
	}
	if len(res.Entries) != 1 {
		t.Fatalf("expected duplicate requests collapsed into 1 entry, got %d", len(res.Entries))
	}
	entry := res.Entries[0]
	if entry.AmshaCode != 1 || entry.VariationCode != 0 {
		t.Fatalf("unexpected entry identity: code=%d variation=%d", entry.AmshaCode, entry.VariationCode)
	}
	// The D1 lagna sweeps all 12 rashis in about one sidereal day.
	if len(entry.Segments) < 12 || len(entry.Segments) > 14 {
		t.Fatalf("unexpected D1 segment count over 24h: %d", len(entry.Segments))
	}
	if entry.Segments[0].Start != from {
		t.Fatalf("first segment must start at from: %+v", entry.Segments[0].Start)
	}
	for i, seg := range entry.Segments {
		if seg.RashiIndex > 11 {
			t.Fatalf("segment %d: rashi index out of range: %d", i, seg.RashiIndex)
		}
		if utcJD(seg.End) <= utcJD(seg.Start) {
			t.Fatalf("segment %d: end not after start: %+v", i, seg)
		}
		if i > 0 {
			prev := entry.Segments[i-1]
			if prev.End != seg.Start {
				t.Fatalf("segment %d does not chain: prev.End=%+v start=%+v", i, prev.End, seg.Start)
			}
			if seg.RashiIndex != (prev.RashiIndex+1)%12 {
				t.Fatalf("segment %d rashi not consecutive: prev=%d cur=%d", i, prev.RashiIndex, seg.RashiIndex)
			}
		}
	}
	last := entry.Segments[len(entry.Segments)-1]
	if utcJD(last.End) < utcJD(to)-1e-5 {
		t.Fatalf("last segment must end at or after to: %+v", last.End)
	}
	if res.Truncated || res.NextFromUTC != nil {
		t.Fatalf("expected untruncated result: truncated=%v next=%v", res.Truncated, res.NextFromUTC)
	}

	// Empty request lists are rejected.
	if _, err := eng.AmshaLagnaEvents(eop, from, to, loc, sank, nil, 0); err == nil {
		t.Fatalf("expected error for empty request list")
	}
}

func TestAmshaSelectionFlowsThroughBalaWrappers(t *testing.T) {
	spk, lskPath, eopPath, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}

	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	defer eng.Close()

	eop, err := LoadEOP(eopPath)
	if err != nil {
		t.Fatalf("LoadEOP: %v", err)
	}
	defer eop.Close()

	utc := UtcTime{Year: 2025, Month: 1, Day: 15, Hour: 12, Minute: 0, Second: 0}
	loc := GeoLocation{LatitudeDeg: 12.9716, LongitudeDeg: 77.5946, AltitudeM: 920}
	bhava := BhavaConfigDefault()
	riseset := RiseSetConfigDefault()

	d2Variation := AmshaSelectionConfig{Count: 1}
	d2Variation.Codes[0] = 2
	d2Variation.Variations[0] = 1

	if result, err := eng.ShadbalaForDate(eop, utc, loc, bhava, riseset, 0, true, d2Variation); err != nil {
		t.Fatalf("ShadbalaForDate with amsha selection: %v", err)
	} else if len(result.Entries) != 7 {
		t.Fatalf("expected 7 shadbala entries, got %d", len(result.Entries))
	}

	if result, err := eng.VimsopakaForDate(eop, utc, loc, 0, true, 0, d2Variation); err != nil {
		t.Fatalf("VimsopakaForDate with amsha selection: %v", err)
	} else if len(result.Entries) != 9 {
		t.Fatalf("expected 9 vimsopaka entries, got %d", len(result.Entries))
	}

	if result, err := eng.BalasForDate(eop, utc, loc, bhava, riseset, 0, true, 0, d2Variation); err != nil {
		t.Fatalf("BalasForDate with amsha selection: %v", err)
	} else if len(result.Shadbala.Entries) != 7 || len(result.Vimsopaka.Entries) != 9 {
		t.Fatalf("unexpected bala bundle sizes: shadbala=%d vimsopaka=%d", len(result.Shadbala.Entries), len(result.Vimsopaka.Entries))
	}

	d9Default := AmshaSelectionConfig{Count: 1}
	d9Default.Codes[0] = 9

	if result, err := eng.AvasthaForDate(eop, utc, loc, bhava, riseset, 0, true, 0, d9Default); err != nil {
		t.Fatalf("AvasthaForDate with amsha selection: %v", err)
	} else if len(result.Entries) != 9 {
		t.Fatalf("expected 9 avastha entries, got %d", len(result.Entries))
	}

	cfg := FullKundaliConfigDefault()
	cfg.IncludeAmshas = true
	cfg.IncludeShadbala = true
	cfg.IncludeVimsopaka = true
	cfg.AmshaSelection = d2Variation

	kundali, err := eng.FullKundaliForDate(eop, utc, loc, bhava, riseset, 0, true, cfg)
	if err != nil {
		t.Fatalf("FullKundaliForDate with resolved amsha union: %v", err)
	}
	if len(kundali.Amshas) != 16 {
		t.Fatalf("expected resolved amsha union of 16 charts, got %d", len(kundali.Amshas))
	}
	if kundali.Amshas[0].AmshaCode != 2 || kundali.Amshas[0].VariationCode != 1 {
		t.Fatalf("expected D2 variation override first, got code=%d variation=%d", kundali.Amshas[0].AmshaCode, kundali.Amshas[0].VariationCode)
	}
}

func TestAshtakavargaContributors(t *testing.T) {
	bav, err := CalculateBAV(0, [7]uint8{0, 1, 2, 3, 4, 5, 6}, 0)
	if err != nil {
		t.Fatalf("CalculateBAV: %v", err)
	}
	for i := 0; i < 12; i++ {
		row := 0
		for j := 0; j < 8; j++ {
			if bav.Contributors[i][j] > 1 {
				t.Fatalf("invalid contributor value %d at rashi=%d contributor=%d", bav.Contributors[i][j], i, j)
			}
			row += int(bav.Contributors[i][j])
		}
		if row != int(bav.Points[i]) {
			t.Fatalf("contributor row sum mismatch at rashi=%d: got=%d want=%d", i, row, bav.Points[i])
		}
	}
}

func TestLowTierDashaWrappers(t *testing.T) {
	spk, lskPath, eopPath, ok := kernelPaths(t)
	if !ok {
		t.Skip("kernel files missing; skipping integration test")
	}

	eng, err := NewEngine(EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lskPath,
		CacheCapacity:    64,
		StrictValidation: false,
	})
	if err != nil {
		t.Fatalf("NewEngine: %v", err)
	}
	defer eng.Close()

	eop, err := LoadEOP(eopPath)
	if err != nil {
		t.Fatalf("LoadEOP: %v", err)
	}
	defer eop.Close()

	birthUTC := UtcTime{Year: 1990, Month: 1, Day: 1, Hour: 12, Minute: 0, Second: 0}
	loc := GeoLocation{LatitudeDeg: 12.9716, LongitudeDeg: 77.5946, AltitudeM: 920}
	bhava := BhavaConfigDefault()
	riseset := RiseSetConfigDefault()
	birth := DashaBirthContext{
		TimeKind:        DashaTimeUTC,
		BirthUTC:        birthUTC,
		HasLocation:     true,
		Location:        loc,
		BhavaConfig:     bhava,
		RiseSetConfig:   riseset,
		SankrantiConfig: SankrantiConfigDefault(),
	}

	level0, err := eng.DashaLevel0(eop, DashaLevel0Request{Birth: birth, System: 0})
	if err != nil {
		t.Fatalf("DashaLevel0: %v", err)
	}
	if len(level0) == 0 {
		t.Fatalf("expected level0 periods")
	}

	first := level0[0]
	same, found, err := eng.DashaLevel0Entity(eop, DashaLevel0EntityRequest{
		Birth:       birth,
		System:      0,
		EntityType:  first.EntityType,
		EntityIndex: first.EntityIndex,
	})
	if err != nil {
		t.Fatalf("DashaLevel0Entity: %v", err)
	}
	if !found || same.EntityIndex != first.EntityIndex {
		t.Fatalf("unexpected level0 entity lookup: found=%v same=%+v first=%+v", found, same, first)
	}

	cycleVariation := DashaVariationConfigDefault()
	cycleVariation.Cycles = 2
	level0Cycles, err := eng.DashaLevel0(eop, DashaLevel0Request{Birth: birth, System: 0, Variation: cycleVariation})
	if err != nil {
		t.Fatalf("DashaLevel0 with cycles: %v", err)
	}
	if len(level0Cycles) != 2*len(level0) {
		t.Fatalf("expected %d level0 periods with cycles=2, got=%d", 2*len(level0), len(level0Cycles))
	}

	variation := DashaVariationConfigDefault()
	children, err := eng.DashaChildren(eop, DashaChildrenRequest{
		Birth:     birth,
		System:    0,
		Variation: variation,
		Parent:    first,
	})
	if err != nil {
		t.Fatalf("DashaChildren: %v", err)
	}
	if len(children) == 0 {
		t.Fatalf("expected child periods")
	}

	child, found, err := eng.DashaChildPeriod(eop, DashaChildPeriodRequest{
		Birth:            birth,
		System:           0,
		Variation:        variation,
		Parent:           first,
		ChildEntityType:  children[0].EntityType,
		ChildEntityIndex: children[0].EntityIndex,
	})
	if err != nil {
		t.Fatalf("DashaChildPeriod: %v", err)
	}
	if !found || child.EntityIndex != children[0].EntityIndex {
		t.Fatalf("unexpected child lookup: found=%v child=%+v firstChild=%+v", found, child, children[0])
	}

	complete, err := eng.DashaCompleteLevel(eop, DashaCompleteLevelRequest{
		Birth:      birth,
		System:     0,
		Variation:  variation,
		ChildLevel: 1,
	}, level0)
	if err != nil {
		t.Fatalf("DashaCompleteLevel: %v", err)
	}
	if len(complete) < len(children) {
		t.Fatalf("expected complete child level, got=%d children=%d", len(complete), len(children))
	}
}
