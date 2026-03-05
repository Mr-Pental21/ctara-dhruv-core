package dhruv

import (
	"fmt"

	"ctara-dhruv-core/bindings/go-open/internal/cabi"
)

type Error struct {
	Op     string
	Status cabi.Status
}

func (e *Error) Error() string {
	return fmt.Sprintf("%s failed with status=%d", e.Op, e.Status)
}

func statusErr(op string, st cabi.Status) error {
	if st == cabi.StatusOK {
		return nil
	}
	return &Error{Op: op, Status: st}
}

func wrapStatus(op string, st cabi.Status, prior error) error {
	if prior != nil {
		return fmt.Errorf("%s: %w", op, prior)
	}
	return statusErr(op, st)
}
