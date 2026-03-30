package dhruv

import "ctara-dhruv-core/bindings/go-open/internal/cabi"

func ConjunctionConfigDefault() ConjunctionConfig { return cabi.ConjunctionConfigDefault() }
func GrahanConfigDefault() GrahanConfig           { return cabi.GrahanConfigDefault() }
func StationaryConfigDefault() StationaryConfig   { return cabi.StationaryConfigDefault() }
func SankrantiConfigDefault() SankrantiConfig     { return cabi.SankrantiConfigDefault() }

const (
	searchRangeMode       = 2
	defaultSearchPageSize = 8
)

func normalizeSearchPageSize(pageSize []uint32) uint32 {
	if len(pageSize) == 0 || pageSize[0] == 0 {
		return defaultSearchPageSize
	}
	return pageSize[0]
}

func nextSearchPageSize(current uint32) uint32 {
	if current > ^uint32(0)/2 {
		return ^uint32(0)
	}
	return current * 2
}

func (e *Engine) ConjunctionSearch(req ConjunctionSearchRequest, pageSize ...uint32) (ConjunctionEvent, bool, []ConjunctionEvent, error) {
	capacity := normalizeSearchPageSize(pageSize)
	ev, found, events, st := cabi.SearchConjunction(e.h, req, capacity)
	for st == 0 && req.QueryMode == searchRangeMode && len(events) >= int(capacity) && capacity != ^uint32(0) {
		capacity = nextSearchPageSize(capacity)
		ev, found, events, st = cabi.SearchConjunction(e.h, req, capacity)
	}
	return ev, found, events, statusErr("conjunction_search_ex", st)
}

func (e *Engine) GrahanSearch(req GrahanSearchRequest, pageSize ...uint32) (ChandraGrahanResult, SuryaGrahanResult, bool, []ChandraGrahanResult, []SuryaGrahanResult, error) {
	capacity := normalizeSearchPageSize(pageSize)
	ch, su, found, che, sue, st := cabi.SearchGrahan(e.h, req, capacity)
	for st == 0 && req.QueryMode == searchRangeMode && len(che) >= int(capacity) && capacity != ^uint32(0) {
		capacity = nextSearchPageSize(capacity)
		ch, su, found, che, sue, st = cabi.SearchGrahan(e.h, req, capacity)
	}
	return ch, su, found, che, sue, statusErr("grahan_search_ex", st)
}

func (e *Engine) MotionSearch(req MotionSearchRequest, pageSize ...uint32) (StationaryEvent, MaxSpeedEvent, bool, []StationaryEvent, []MaxSpeedEvent, error) {
	capacity := normalizeSearchPageSize(pageSize)
	se, me, found, ses, mes, st := cabi.SearchMotion(e.h, req, capacity)
	for st == 0 && req.QueryMode == searchRangeMode && len(ses) >= int(capacity) && capacity != ^uint32(0) {
		capacity = nextSearchPageSize(capacity)
		se, me, found, ses, mes, st = cabi.SearchMotion(e.h, req, capacity)
	}
	return se, me, found, ses, mes, statusErr("motion_search_ex", st)
}

func (e *Engine) LunarPhaseSearch(req LunarPhaseSearchRequest, pageSize ...uint32) (LunarPhaseEvent, bool, []LunarPhaseEvent, error) {
	capacity := normalizeSearchPageSize(pageSize)
	ev, found, events, st := cabi.SearchLunarPhase(e.h, req, capacity)
	for st == 0 && req.QueryMode == searchRangeMode && len(events) >= int(capacity) && capacity != ^uint32(0) {
		capacity = nextSearchPageSize(capacity)
		ev, found, events, st = cabi.SearchLunarPhase(e.h, req, capacity)
	}
	return ev, found, events, statusErr("lunar_phase_search_ex", st)
}

func (e *Engine) SankrantiSearch(req SankrantiSearchRequest, pageSize ...uint32) (SankrantiEvent, bool, []SankrantiEvent, error) {
	capacity := normalizeSearchPageSize(pageSize)
	ev, found, events, st := cabi.SearchSankranti(e.h, req, capacity)
	for st == 0 && req.QueryMode == searchRangeMode && len(events) >= int(capacity) && capacity != ^uint32(0) {
		capacity = nextSearchPageSize(capacity)
		ev, found, events, st = cabi.SearchSankranti(e.h, req, capacity)
	}
	return ev, found, events, statusErr("sankranti_search_ex", st)
}
