//! Dasha variation configuration: sub-period method selection per level.

/// How child periods are divided within a parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SubPeriodMethod {
    /// child_duration = (child_full_period / total_period) * parent_duration.
    /// Sequence starts from parent's entity in the cyclic order.
    ProportionalFromParent = 0,
    /// child_duration = parent_duration / num_children.
    /// Sequence starts from entity after parent.
    EqualFromNext = 1,
    /// child_duration = parent_duration / num_children.
    /// Sequence starts from parent's entity.
    EqualFromSame = 2,
    /// Proportional but sequence starts from next entity after parent.
    ProportionalFromNext = 3,
}

impl SubPeriodMethod {
    /// Create from raw u8 value.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::ProportionalFromParent),
            1 => Some(Self::EqualFromNext),
            2 => Some(Self::EqualFromSame),
            3 => Some(Self::ProportionalFromNext),
            _ => None,
        }
    }
}

/// Yogini dasha scheme variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum YoginiScheme {
    #[default]
    Default = 0,
    LaDeepanshuGiri = 1,
}

impl YoginiScheme {
    /// Create from raw u8 value.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Default),
            1 => Some(Self::LaDeepanshuGiri),
            _ => None,
        }
    }
}

/// Per-level variation overrides.
///
/// Array indices 0-4 correspond to DashaLevel 0-4.
/// Index i controls how children of level-i are generated.
/// Index 4 (Pranadasha) is reserved (it has no children).
#[derive(Debug, Clone, Copy)]
pub struct DashaVariationConfig {
    /// Override sub-period method per level.
    /// None = use system default for that level.
    pub level_methods: [Option<SubPeriodMethod>; 5],
    /// Yogini scheme variant.
    pub yogini_scheme: YoginiScheme,
    /// For Ashtottari: use 28-nakshatra Abhijit detection.
    pub use_abhijit: bool,
    /// Level-0 cycle repetition: explicit whole-cycle count (1-255).
    /// None = use the system's default cycle count. Takes precedence over
    /// `min_span_years`. Applies to nakshatra-based and Yogini systems only;
    /// other systems ignore it.
    pub cycles: Option<u8>,
    /// Level-0 cycle repetition: repeat whole cycles until level-0 coverage
    /// from birth reaches at least this many years (the final cycle completes
    /// even if it overshoots). None = use the system's default cycle count.
    /// Applies to nakshatra-based and Yogini systems only.
    pub min_span_years: Option<f64>,
}

impl Default for DashaVariationConfig {
    fn default() -> Self {
        Self {
            level_methods: [None; 5],
            yogini_scheme: YoginiScheme::Default,
            use_abhijit: true,
            cycles: None,
            min_span_years: None,
        }
    }
}

impl DashaVariationConfig {
    /// Get effective sub-period method for a level, with system default fallback.
    pub fn method_for_level(&self, level: u8, system_default: SubPeriodMethod) -> SubPeriodMethod {
        if level <= 4 {
            self.level_methods[level as usize].unwrap_or(system_default)
        } else {
            system_default
        }
    }

    /// Resolve the effective level-0 cycle count for a cyclic system.
    ///
    /// `default_count` is the system's built-in cycle count.
    /// `first_cycle_span_days` is the level-0 coverage of the first cycle
    /// (birth balance plus the remaining periods of that cycle) and
    /// `full_cycle_days` the coverage of every subsequent cycle.
    /// Explicit `cycles` wins over `min_span_years`; both absent (or
    /// non-positive/non-finite span inputs) fall back to `default_count`.
    pub fn effective_cycle_count(
        &self,
        default_count: u8,
        first_cycle_span_days: f64,
        full_cycle_days: f64,
    ) -> u8 {
        if let Some(cycles) = self.cycles {
            return cycles.max(1);
        }
        if let Some(years) = self.min_span_years {
            let target_days = years * super::types::DAYS_PER_YEAR;
            if !target_days.is_finite() || target_days <= 0.0 || full_cycle_days <= 0.0 {
                return default_count.max(1);
            }
            if first_cycle_span_days >= target_days {
                return 1;
            }
            let extra = ((target_days - first_cycle_span_days) / full_cycle_days).ceil();
            return (1.0 + extra).min(u8::MAX as f64) as u8;
        }
        default_count.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_period_method_from_u8() {
        assert_eq!(
            SubPeriodMethod::from_u8(0),
            Some(SubPeriodMethod::ProportionalFromParent)
        );
        assert_eq!(SubPeriodMethod::from_u8(4), None);
    }

    #[test]
    fn yogini_scheme_from_u8() {
        assert_eq!(YoginiScheme::from_u8(0), Some(YoginiScheme::Default));
        assert_eq!(YoginiScheme::from_u8(2), None);
    }

    #[test]
    fn default_variation_uses_system_default() {
        let cfg = DashaVariationConfig::default();
        assert_eq!(
            cfg.method_for_level(0, SubPeriodMethod::ProportionalFromParent),
            SubPeriodMethod::ProportionalFromParent,
        );
    }

    #[test]
    fn effective_cycle_count_default() {
        let cfg = DashaVariationConfig::default();
        assert_eq!(cfg.effective_cycle_count(2, 30.0 * 365.25, 36.0 * 365.25), 2);
    }

    #[test]
    fn effective_cycle_count_explicit_cycles_win() {
        let cfg = DashaVariationConfig {
            cycles: Some(4),
            min_span_years: Some(1000.0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(1, 30.0 * 365.25, 36.0 * 365.25), 4);
    }

    #[test]
    fn effective_cycle_count_zero_cycles_clamps_to_one() {
        let cfg = DashaVariationConfig {
            cycles: Some(0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(3, 30.0 * 365.25, 36.0 * 365.25), 1);
    }

    #[test]
    fn effective_cycle_count_min_span() {
        // 36y cycles, first cycle covers 30y (partial balance). Target 100y:
        // 30 + 2*36 = 102 >= 100 → 3 cycles.
        let cfg = DashaVariationConfig {
            min_span_years: Some(100.0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(1, 30.0 * 365.25, 36.0 * 365.25), 3);
    }

    #[test]
    fn effective_cycle_count_min_span_within_first_cycle() {
        let cfg = DashaVariationConfig {
            min_span_years: Some(20.0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(2, 30.0 * 365.25, 36.0 * 365.25), 1);
    }

    #[test]
    fn effective_cycle_count_min_span_exact_boundary() {
        // 30 + 1*36 = 66 exactly → 2 cycles.
        let cfg = DashaVariationConfig {
            min_span_years: Some(66.0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(1, 30.0 * 365.25, 36.0 * 365.25), 2);
    }

    #[test]
    fn effective_cycle_count_invalid_span_falls_back() {
        let cfg = DashaVariationConfig {
            min_span_years: Some(-5.0),
            ..DashaVariationConfig::default()
        };
        assert_eq!(cfg.effective_cycle_count(2, 30.0 * 365.25, 36.0 * 365.25), 2);
        let nan = DashaVariationConfig {
            min_span_years: Some(f64::NAN),
            ..DashaVariationConfig::default()
        };
        assert_eq!(nan.effective_cycle_count(2, 30.0 * 365.25, 36.0 * 365.25), 2);
    }

    #[test]
    fn override_works() {
        let mut cfg = DashaVariationConfig::default();
        cfg.level_methods[1] = Some(SubPeriodMethod::EqualFromNext);
        assert_eq!(
            cfg.method_for_level(1, SubPeriodMethod::ProportionalFromParent),
            SubPeriodMethod::EqualFromNext,
        );
    }
}
