package main

import (
	"fmt"
	"log"
	"os"

	"ctara-dhruv-core/bindings/go-open/dhruv"
)

func main() {
	spk := os.Getenv("DHRUV_SPK_PATH")
	lsk := os.Getenv("DHRUV_LSK_PATH")
	if spk == "" || lsk == "" {
		log.Fatal("set DHRUV_SPK_PATH and DHRUV_LSK_PATH")
	}

	if err := dhruv.VerifyABI(); err != nil {
		log.Fatalf("ABI check failed: %v", err)
	}

	engine, err := dhruv.NewEngine(dhruv.EngineConfig{
		SpkPaths:         []string{spk},
		LskPath:          lsk,
		CacheCapacity:    128,
		StrictValidation: false,
	})
	if err != nil {
		log.Fatalf("engine init: %v", err)
	}
	defer engine.Close()

	state, err := engine.Query(dhruv.Query{Target: 301, Observer: 399, Frame: 1, EpochTdbJD: 2451545.0})
	if err != nil {
		log.Fatalf("query failed: %v", err)
	}

	fmt.Printf("Moon position km: [%.3f %.3f %.3f]\n", state.PositionKm[0], state.PositionKm[1], state.PositionKm[2])
}
