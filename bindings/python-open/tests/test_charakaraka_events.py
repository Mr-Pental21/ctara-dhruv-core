"""Tests for charakaraka ranking-change events and build identity."""

import pytest
from conftest import skip_no_kernels, skip_no_eop


class TestBuildInfo:
    def test_library_version_non_empty(self):
        import ctara_dhruv
        version = ctara_dhruv.library_version()
        assert isinstance(version, str)
        assert len(version) > 0

    def test_build_git_hash_non_empty(self):
        import ctara_dhruv
        git_hash = ctara_dhruv.build_git_hash()
        assert isinstance(git_hash, str)
        assert len(git_hash) > 0


class TestCharakarakaEventsExports:
    def test_ceiling_constant(self):
        import ctara_dhruv
        assert ctara_dhruv.MAX_CHARAKARAKA_EVENTS == 50000

    def test_types_exported(self):
        from ctara_dhruv import CharakarakaChangeEvent, CharakarakaEventsResult
        assert CharakarakaChangeEvent is not None
        assert CharakarakaEventsResult is not None


@skip_no_kernels
@skip_no_eop
class TestCharakarakaEvents:
    _FROM = (2024, 1, 1, 0, 0, 0.0)
    _TO = (2024, 1, 5, 0, 0, 0.0)

    def test_range_scheme_eight_smoke(self, engine_handles):
        """Scheme-EIGHT sweep over a few days yields ascending, complete events."""
        from ctara_dhruv.kundali import charakaraka_events
        from ctara_dhruv.engine import engine, eop

        result = charakaraka_events(engine(), eop(), self._FROM, self._TO)
        assert not result.truncated
        assert result.next_from is None

        events = result.events
        assert len(events) > 0
        for e in events:
            assert e.trigger in (0, 1, 2)
            assert e.trigger_name in (
                "degree_crossing", "rashi_ingress", "scheme_mode_change",
            )
            assert len(e.changed_roles) > 0
            assert all(0 <= role <= 8 for role in e.changed_roles)
            assert e.before.entries
            assert e.after.entries
            assert e.utc.year == 2024
        for a, b in zip(events, events[1:]):
            assert a.jd_tdb < b.jd_tdb
            assert a.utc.to_datetime() < b.utc.to_datetime()

    def test_truncation_carries_resume_point(self, engine_handles):
        """max_events=3 truncates the sweep and yields a resume point."""
        from ctara_dhruv.kundali import charakaraka_events
        from ctara_dhruv.engine import engine, eop

        result = charakaraka_events(
            engine(), eop(), self._FROM, self._TO, max_events=3,
        )
        assert result.truncated
        assert len(result.events) == 3
        assert result.next_from is not None

    def test_next_prev_smoke(self, engine_handles):
        """next/prev bracket the query instant with real change events."""
        from ctara_dhruv.kundali import next_charakaraka_event, prev_charakaraka_event
        from ctara_dhruv.engine import engine, eop

        at = (2024, 1, 3, 0, 0, 0.0)
        nxt = next_charakaraka_event(engine(), eop(), at)
        prv = prev_charakaraka_event(engine(), eop(), at)
        assert nxt is not None
        assert prv is not None
        assert prv.jd_tdb < nxt.jd_tdb
        assert prv.changed_roles
        assert nxt.changed_roles
