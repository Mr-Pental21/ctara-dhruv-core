package dhruv

import "ctara-dhruv-core/bindings/go-open/internal/cabi"

func ConjunctionConfigDefault() ConjunctionConfig { return cabi.ConjunctionConfigDefault() }
func GrahanConfigDefault() GrahanConfig           { return cabi.GrahanConfigDefault() }
func StationaryConfigDefault() StationaryConfig   { return cabi.StationaryConfigDefault() }
func SankrantiConfigDefault() SankrantiConfig     { return cabi.SankrantiConfigDefault() }

func (e *Engine) ConjunctionSearch(req ConjunctionSearchRequest, capacity uint32) (ConjunctionEvent, bool, []ConjunctionEvent, error) {
	ev, found, events, st := cabi.SearchConjunction(e.h, req, capacity)
	return ev, found, events, statusErr("conjunction_search_ex", st)
}

func (e *Engine) GrahanSearch(req GrahanSearchRequest, capacity uint32) (ChandraGrahanResult, SuryaGrahanResult, bool, []ChandraGrahanResult, []SuryaGrahanResult, error) {
	ch, su, found, che, sue, st := cabi.SearchGrahan(e.h, req, capacity)
	return ch, su, found, che, sue, statusErr("grahan_search_ex", st)
}

func (e *Engine) MotionSearch(req MotionSearchRequest, capacity uint32) (StationaryEvent, MaxSpeedEvent, bool, []StationaryEvent, []MaxSpeedEvent, error) {
	se, me, found, ses, mes, st := cabi.SearchMotion(e.h, req, capacity)
	return se, me, found, ses, mes, statusErr("motion_search_ex", st)
}

func (e *Engine) LunarPhaseSearch(req LunarPhaseSearchRequest, capacity uint32) (LunarPhaseEvent, bool, []LunarPhaseEvent, error) {
	ev, found, events, st := cabi.SearchLunarPhase(e.h, req, capacity)
	return ev, found, events, statusErr("lunar_phase_search_ex", st)
}

func (e *Engine) SankrantiSearch(req SankrantiSearchRequest, capacity uint32) (SankrantiEvent, bool, []SankrantiEvent, error) {
	ev, found, events, st := cabi.SearchSankranti(e.h, req, capacity)
	return ev, found, events, statusErr("sankranti_search_ex", st)
}
