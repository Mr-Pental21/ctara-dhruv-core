package dhruv

import (
	"runtime"

	"ctara-dhruv-core/bindings/go-open/internal/cabi"
)

type DashaHierarchy struct {
	h cabi.DashaHierarchyHandle
}

func (e *Engine) DashaHierarchyUTC(ep *EOP, birthUTC UtcTime, loc GeoLocation, bhavaCfg BhavaConfig, riseCfg RiseSetConfig, ayanamshaSystem uint32, useNutation bool, system uint8, maxLevel uint8) (*DashaHierarchy, error) {
	h, st := cabi.DashaHierarchyUTC(e.h, ep.h, birthUTC, loc, bhavaCfg, riseCfg, ayanamshaSystem, useNutation, system, maxLevel)
	if err := statusErr("dasha_hierarchy_utc", st); err != nil {
		return nil, err
	}
	d := &DashaHierarchy{h: h}
	runtime.SetFinalizer(d, (*DashaHierarchy).Close)
	return d, nil
}

func (d *DashaHierarchy) Close() {
	if d == nil {
		return
	}
	runtime.SetFinalizer(d, nil)
	d.h.Free()
}

func (d *DashaHierarchy) LevelCount() (uint8, error) {
	out, st := d.h.LevelCount()
	return out, statusErr("dasha_hierarchy_level_count", st)
}

func (d *DashaHierarchy) PeriodCount(level uint8) (uint32, error) {
	out, st := d.h.PeriodCount(level)
	return out, statusErr("dasha_hierarchy_period_count", st)
}

func (d *DashaHierarchy) PeriodAt(level uint8, idx uint32) (DashaPeriod, error) {
	out, st := d.h.PeriodAt(level, idx)
	return out, statusErr("dasha_hierarchy_period_at", st)
}

func (e *Engine) DashaSnapshotUTC(ep *EOP, birthUTC, queryUTC UtcTime, loc GeoLocation, bhavaCfg BhavaConfig, riseCfg RiseSetConfig, ayanamshaSystem uint32, useNutation bool, system uint8, maxLevel uint8) (DashaSnapshot, error) {
	out, st := cabi.DashaSnapshotUTC(e.h, ep.h, birthUTC, queryUTC, loc, bhavaCfg, riseCfg, ayanamshaSystem, useNutation, system, maxLevel)
	return out, statusErr("dasha_snapshot_utc", st)
}
