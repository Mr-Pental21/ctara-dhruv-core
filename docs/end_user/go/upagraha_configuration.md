# Go Upagraha Configuration

Go exposes `TimeUpagrahaConfig` plus constants for point and planet selection.

```go
cfg := dhruv.TimeUpagrahaConfigDefault()
cfg.GulikaPoint = dhruv.UpagrahaPointMiddle
cfg.GulikaPlanet = dhruv.GulikaMaandiPlanetSaturn
cfg.MaandiPoint = dhruv.UpagrahaPointEnd
cfg.MaandiPlanet = dhruv.GulikaMaandiPlanetRahu
cfg.OtherPoint = dhruv.UpagrahaPointStart

upagrahas, err := engine.AllUpagrahasForDateWithConfig(
    eop, utc, loc, 0, false, cfg,
)

bindusCfg := dhruv.BindusConfig{
    IncludeNakshatra: true,
    IncludeBhava:     true,
    UpagrahaConfig:   cfg,
}

bindus, err := engine.CoreBindusForDate(
    eop, utc, loc, bhavaCfg, riseCfg, 0, false, bindusCfg,
)
```
