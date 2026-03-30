package dhruv

import (
	"runtime"

	"ctara-dhruv-core/bindings/go-open/internal/cabi"
)

type TaraCatalog struct {
	h cabi.TaraCatalogHandle
}

func LoadTaraCatalog(path string) (*TaraCatalog, error) {
	h, st := cabi.LoadTaraCatalog(path)
	if err := statusErr("tara_catalog_load", st); err != nil {
		return nil, err
	}
	c := &TaraCatalog{h: h}
	runtime.SetFinalizer(c, (*TaraCatalog).Close)
	return c, nil
}

func (c *TaraCatalog) Close() {
	if c == nil {
		return
	}
	runtime.SetFinalizer(c, nil)
	c.h.Free()
}

func (c *TaraCatalog) Compute(req TaraComputeRequest) (TaraComputeResult, error) {
	out, st := cabi.TaraComputeEx(c.h, req)
	return out, statusErr("tara_compute_ex", st)
}

func (c *TaraCatalog) GalacticCenterEcliptic(jdTdb float64) (SphericalCoords, error) {
	out, st := cabi.TaraGalacticCenterEcliptic(c.h, jdTdb)
	return out, statusErr("tara_galactic_center_ecliptic", st)
}

func TaraPropagatePosition(raDeg, decDeg, parallaxMas, pmRaMasYr, pmDecMasYr, rvKmS, dtYears float64) (EquatorialPosition, error) {
	out, st := cabi.TaraPropagatePosition(raDeg, decDeg, parallaxMas, pmRaMasYr, pmDecMasYr, rvKmS, dtYears)
	return out, statusErr("tara_propagate_position", st)
}

func TaraApplyAberration(direction [3]float64, earthVelAUDay [3]float64) ([3]float64, error) {
	out, st := cabi.TaraApplyAberration(direction, earthVelAUDay)
	return out, statusErr("tara_apply_aberration", st)
}

func TaraApplyLightDeflection(direction [3]float64, sunToObserver [3]float64, sunObserverDistanceAU float64) ([3]float64, error) {
	out, st := cabi.TaraApplyLightDeflection(direction, sunToObserver, sunObserverDistanceAU)
	return out, statusErr("tara_apply_light_deflection", st)
}

func TaraGalacticAnticenterICRS() ([3]float64, error) {
	out, st := cabi.TaraGalacticAnticenterICRS()
	return out, statusErr("tara_galactic_anticenter_icrs", st)
}
