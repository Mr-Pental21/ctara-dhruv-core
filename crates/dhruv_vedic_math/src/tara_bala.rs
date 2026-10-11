//! Tara bala: the nine-fold tara of a nakshatra counted from a reference
//! (usually the natal Moon's) nakshatra.
//!
//! Counting from the reference nakshatra inclusively, the 27 nakshatras fall
//! into three cycles of nine taras: Janma, Sampat, Vipat, Kshema, Pratyak,
//! Sadhaka, Vadha, Mitra, Parama Mitra. The tara index is the count modulo
//! nine; the cycle (1-3) records which pass through the sequence the target
//! falls in.
//!
//! Clean-room implementation of the universal classical rule (Muhurta
//! texts); no external code consulted.

/// The nine taras in classical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tara {
    Janma,
    Sampat,
    Vipat,
    Kshema,
    Pratyak,
    Sadhaka,
    Vadha,
    Mitra,
    ParamaMitra,
}

/// All nine taras in order, for indexing.
pub const ALL_TARAS: [Tara; 9] = [
    Tara::Janma,
    Tara::Sampat,
    Tara::Vipat,
    Tara::Kshema,
    Tara::Pratyak,
    Tara::Sadhaka,
    Tara::Vadha,
    Tara::Mitra,
    Tara::ParamaMitra,
];

impl Tara {
    /// Sanskrit name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Janma => "Janma",
            Self::Sampat => "Sampat",
            Self::Vipat => "Vipat",
            Self::Kshema => "Kshema",
            Self::Pratyak => "Pratyak",
            Self::Sadhaka => "Sadhaka",
            Self::Vadha => "Vadha",
            Self::Mitra => "Mitra",
            Self::ParamaMitra => "Parama Mitra",
        }
    }

    /// 0-based index in the nine-fold sequence (Janma=0 .. Parama Mitra=8).
    pub const fn index(self) -> u8 {
        match self {
            Self::Janma => 0,
            Self::Sampat => 1,
            Self::Vipat => 2,
            Self::Kshema => 3,
            Self::Pratyak => 4,
            Self::Sadhaka => 5,
            Self::Vadha => 6,
            Self::Mitra => 7,
            Self::ParamaMitra => 8,
        }
    }

    /// Classical favourability: Sampat, Kshema, Sadhaka, Mitra and Parama
    /// Mitra are favourable; Janma, Vipat, Pratyak and Vadha are not.
    pub const fn is_favorable(self) -> bool {
        matches!(
            self,
            Self::Sampat | Self::Kshema | Self::Sadhaka | Self::Mitra | Self::ParamaMitra
        )
    }
}

/// Tara bala of one nakshatra relative to a reference nakshatra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaraBala {
    /// The tara.
    pub tara: Tara,
    /// 0-based tara index (0=Janma .. 8=Parama Mitra).
    pub tara_index: u8,
    /// Inclusive count from the reference nakshatra to the target (1..27).
    pub count: u8,
    /// Which pass through the nine-fold sequence the target falls in (1..3).
    pub cycle: u8,
    /// Classical favourability of the tara.
    pub favorable: bool,
}

/// Compute the tara bala of `target_nakshatra_index` counted from
/// `reference_nakshatra_index` (both 0-based, 0=Ashwini .. 26=Revati; values
/// are reduced modulo 27).
pub fn tara_bala(reference_nakshatra_index: u8, target_nakshatra_index: u8) -> TaraBala {
    let reference = (reference_nakshatra_index % 27) as i16;
    let target = (target_nakshatra_index % 27) as i16;
    let count = (((target - reference) + 27) % 27 + 1) as u8;
    let tara_index = (count - 1) % 9;
    let cycle = (count - 1) / 9 + 1;
    let tara = ALL_TARAS[tara_index as usize];
    TaraBala {
        tara,
        tara_index,
        count,
        cycle,
        favorable: tara.is_favorable(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tara_indices_sequential() {
        for (i, t) in ALL_TARAS.iter().enumerate() {
            assert_eq!(t.index() as usize, i);
            assert!(!t.name().is_empty());
        }
    }

    #[test]
    fn same_nakshatra_is_janma() {
        let tb = tara_bala(4, 4);
        assert_eq!(tb.tara, Tara::Janma);
        assert_eq!(tb.count, 1);
        assert_eq!(tb.cycle, 1);
        assert!(!tb.favorable);
    }

    #[test]
    fn full_sequence_from_ashwini() {
        let expected = [
            Tara::Janma,
            Tara::Sampat,
            Tara::Vipat,
            Tara::Kshema,
            Tara::Pratyak,
            Tara::Sadhaka,
            Tara::Vadha,
            Tara::Mitra,
            Tara::ParamaMitra,
        ];
        for target in 0u8..27 {
            let tb = tara_bala(0, target);
            assert_eq!(tb.count, target + 1);
            assert_eq!(tb.tara, expected[(target % 9) as usize]);
            assert_eq!(tb.cycle, target / 9 + 1);
        }
    }

    #[test]
    fn wraps_around_revati() {
        // Reference Revati (26), target Ashwini (0): count 2 = Sampat.
        let tb = tara_bala(26, 0);
        assert_eq!(tb.count, 2);
        assert_eq!(tb.tara, Tara::Sampat);
        assert!(tb.favorable);
        // Reference Rohini (3), target Krittika (2): count 27 = Parama Mitra, cycle 3.
        let tb = tara_bala(3, 2);
        assert_eq!(tb.count, 27);
        assert_eq!(tb.tara, Tara::ParamaMitra);
        assert_eq!(tb.cycle, 3);
    }

    #[test]
    fn favourability_set() {
        let fav: Vec<bool> = ALL_TARAS.iter().map(|t| t.is_favorable()).collect();
        assert_eq!(
            fav,
            vec![false, true, false, true, false, true, false, true, true]
        );
    }

    #[test]
    fn out_of_range_indices_reduce_mod_27() {
        assert_eq!(tara_bala(27, 0), tara_bala(0, 0));
        assert_eq!(tara_bala(0, 28), tara_bala(0, 1));
    }
}
