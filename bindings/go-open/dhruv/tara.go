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
