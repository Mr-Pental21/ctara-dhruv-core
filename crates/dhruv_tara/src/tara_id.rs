//! Fixed star identifier enum.
//!
//! Each variant maps to a single physical star (no duplicates).
//! Ranges: yogataras 0-27, rashi constellation stars 100+, special stars 200+,
//! galactic reference points 300+.

/// Unique identifier for a fixed star or reference point.
///
/// `#[repr(i32)]` for FFI compatibility.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaraId {
    // ---- Nakshatra Yogataras (0-27) ----
    // 1. Ashvini
    Sheratan = 0,
    // 2. Bharani
    FortyCetAlrescha = 1,
    // 3. Krittika
    Alcyone = 2,
    // 4. Rohini
    Aldebaran = 3,
    // 5. Mrigashira
    LambdaOri = 4,
    // 6. Ardra
    Betelgeuse = 5,
    // 7. Punarvasu
    Pollux = 6,
    // 8. Pushya
    DeltaCnc = 7,
    // 9. Ashlesha
    EpsilonHya = 8,
    // 10. Magha
    Regulus = 9,
    // 11. Purva Phalguni
    DeltaLeo = 10,
    // 12. Uttara Phalguni
    Denebola = 11,
    // 13. Hasta
    DeltaCrv = 12,
    // 14. Chitra
    Chitra = 13,
    // 15. Svati
    Arcturus = 14,
    // 16. Vishakha
    AlphaLib = 15,
    // 17. Anuradha
    DeltaSco = 16,
    // 18. Jyeshtha
    Antares = 17,
    // 19. Mula
    LambdaSco = 18,
    // 20. Purva Ashadha
    DeltaSgr = 19,
    // 21. Uttara Ashadha
    SigmaSgr = 20,
    // 22. Shravana
    Altair = 21,
    // 23. Dhanishta
    BetaDel = 22,
    // 24. Shatabhisha
    LambdaAqr = 23,
    // 25. Purva Bhadrapada
    AlphaPeg = 24,
    // 26. Uttara Bhadrapada
    GammaPeg = 25,
    // 27. Revati
    ZetaPsc = 26,
    // 28. Abhijit (28th nakshatra)
    Vega = 27,

    // ---- Rashi constellation stars (100+) ----
    // Mesha (Aries)
    Hamal = 100,
    Mesarthim = 101,

    // Vrishabha (Taurus)
    ElNath = 102,
    Ain = 103,
    Merope = 104,
    Electra = 105,
    Taygeta = 106,
    Maia = 107,
    Atlas = 108,

    // Mithuna (Gemini)
    Castor = 109,
    Alhena = 110,
    Mebsuta = 111,
    Tejat = 112,
    Propus = 113,

    // Karka (Cancer)
    Acubens = 114,
    Altarf = 115,
    Praesepe = 116,

    // Simha (Leo)
    Algieba = 117,
    Zosma = 118,
    Adhafera = 119,
    RasElased = 120,
    Algenubi = 121,
    Chertan = 122,

    // Kanya (Virgo)
    Zavijava = 123,
    Porrima = 124,
    Auva = 125,
    Vindemiatrix = 126,
    Heze = 127,
    Zaniah = 128,

    // Tula (Libra)
    Zubeneschamali = 129,
    Zubenelgenubi = 130,
    Brachium = 131,

    // Vrischika (Scorpio)
    Shaula = 132,
    Sargas = 133,
    Dschubba = 134,
    Acrab = 135,
    Lesath = 136,
    AlNiyat = 137,
    AlniyatTau = 138,

    // Dhanu (Sagittarius)
    KausMedia = 139,
    KausAustralis = 140,
    KausBorealis = 141,
    Nunki = 142,
    Ascella = 143,
    Rukbat = 144,
    Arkab = 145,

    // Makara (Capricorn)
    DenebAlgedi = 146,
    Dabih = 147,
    Algedi = 148,
    Nashira = 149,

    // Kumbha (Aquarius)
    Sadalsuud = 150,
    Sadalmelik = 151,
    Skat = 152,
    Albali = 153,
    Ancha = 154,

    // Meena (Pisces)
    Fomalhaut = 155,
    EtaPsc = 156,
    OmicronPsc = 157,
    Alrescha = 158,

    // Other rashi constellation stars
    Sirius = 159,
    Canopus = 160,
    Rigel = 161,
    Procyon = 162,
    Capella = 163,
    Bellatrix = 164,
    Mintaka = 165,
    Alnilam = 166,
    Alnitak = 167,
    Saiph = 168,
    Wezen = 169,
    Adhara = 170,
    Mirzam = 171,
    Aludra = 172,
    Menkib = 173,
    Phact = 174,
    Naos = 175,
    Alphard = 176,
    Gienah = 177,
    Minkar = 178,
    Algorab = 179,

    // ---- Special Vedic Stars (200+) ----
    // Dhruva (pole star)
    Polaris = 200,
    // Agastya (Canopus - also in rashi, but special Vedic significance)
    Agastya = 201,
    // Arundhati (Mizar + Alcor system, use Mizar)
    Arundhati = 202,
    // Vasishtha (Mizar, component of Arundhati)
    // Lubdhaka (Sirius, hunter star)
    Lubdhaka = 203,
    // Trishankhu (Southern Cross region, use α Cru)
    Trishankhu = 204,
    // Prajapati (Orion / δ Ori region, use δ Ori)
    Prajapati = 205,
    // Brahma Hridaya (Capella)
    BrahmaHridaya = 206,
    // Apamvatsa
    Apamvatsa = 207,
    // Yama (Southern stars)
    Achernar = 208,
    // Varuna
    Ankaa = 209,
    // Mitra
    Eltanin = 210,
    // Indra / Jyeshtha regent
    Indra = 211,

    // ---- Galactic Reference Points (300+) ----
    GalacticCenter = 300,
    GalacticAntiCenter = 301,
}

/// Star category for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaraCategory {
    /// Nakshatra yogatara (0-27)
    Yogatara,
    /// Rashi constellation star (100+)
    RashiConstellation,
    /// Special Vedic star (200+)
    SpecialVedic,
    /// Galactic reference point (300+)
    GalacticReference,
}

impl TaraId {
    /// Category of this star.
    pub fn category(self) -> TaraCategory {
        let code = self as i32;
        if code < 100 {
            TaraCategory::Yogatara
        } else if code < 200 {
            TaraCategory::RashiConstellation
        } else if code < 300 {
            TaraCategory::SpecialVedic
        } else {
            TaraCategory::GalacticReference
        }
    }

    /// Canonical string name for this star (matches JSON catalog `id` field).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sheratan => "Sheratan",
            Self::FortyCetAlrescha => "41Ari",
            Self::Alcyone => "Alcyone",
            Self::Aldebaran => "Aldebaran",
            Self::LambdaOri => "LambdaOri",
            Self::Betelgeuse => "Betelgeuse",
            Self::Pollux => "Pollux",
            Self::DeltaCnc => "DeltaCnc",
            Self::EpsilonHya => "EpsilonHya",
            Self::Regulus => "Regulus",
            Self::DeltaLeo => "DeltaLeo",
            Self::Denebola => "Denebola",
            Self::DeltaCrv => "DeltaCrv",
            Self::Chitra => "Chitra",
            Self::Arcturus => "Arcturus",
            Self::AlphaLib => "AlphaLib",
            Self::DeltaSco => "DeltaSco",
            Self::Antares => "Antares",
            Self::LambdaSco => "LambdaSco",
            Self::DeltaSgr => "DeltaSgr",
            Self::SigmaSgr => "SigmaSgr",
            Self::Altair => "Altair",
            Self::BetaDel => "BetaDel",
            Self::LambdaAqr => "LambdaAqr",
            Self::AlphaPeg => "AlphaPeg",
            Self::GammaPeg => "GammaPeg",
            Self::ZetaPsc => "ZetaPsc",
            Self::Vega => "Vega",
            Self::Hamal => "Hamal",
            Self::Mesarthim => "Mesarthim",
            Self::ElNath => "ElNath",
            Self::Ain => "Ain",
            Self::Merope => "Merope",
            Self::Electra => "Electra",
            Self::Taygeta => "Taygeta",
            Self::Maia => "Maia",
            Self::Atlas => "Atlas",
            Self::Castor => "Castor",
            Self::Alhena => "Alhena",
            Self::Mebsuta => "Mebsuta",
            Self::Tejat => "Tejat",
            Self::Propus => "Propus",
            Self::Acubens => "Acubens",
            Self::Altarf => "Altarf",
            Self::Praesepe => "Praesepe",
            Self::Algieba => "Algieba",
            Self::Zosma => "Zosma",
            Self::Adhafera => "Adhafera",
            Self::RasElased => "RasElased",
            Self::Algenubi => "Algenubi",
            Self::Chertan => "Chertan",
            Self::Zavijava => "Zavijava",
            Self::Porrima => "Porrima",
            Self::Auva => "Auva",
            Self::Vindemiatrix => "Vindemiatrix",
            Self::Heze => "Heze",
            Self::Zaniah => "Zaniah",
            Self::Zubeneschamali => "Zubeneschamali",
            Self::Zubenelgenubi => "Zubenelgenubi",
            Self::Brachium => "Brachium",
            Self::Shaula => "Shaula",
            Self::Sargas => "Sargas",
            Self::Dschubba => "Dschubba",
            Self::Acrab => "Acrab",
            Self::Lesath => "Lesath",
            Self::AlNiyat => "AlNiyat",
            Self::AlniyatTau => "AlniyatTau",
            Self::KausMedia => "KausMedia",
            Self::KausAustralis => "KausAustralis",
            Self::KausBorealis => "KausBorealis",
            Self::Nunki => "Nunki",
            Self::Ascella => "Ascella",
            Self::Rukbat => "Rukbat",
            Self::Arkab => "Arkab",
            Self::DenebAlgedi => "DenebAlgedi",
            Self::Dabih => "Dabih",
            Self::Algedi => "Algedi",
            Self::Nashira => "Nashira",
            Self::Sadalsuud => "Sadalsuud",
            Self::Sadalmelik => "Sadalmelik",
            Self::Skat => "Skat",
            Self::Albali => "Albali",
            Self::Ancha => "Ancha",
            Self::Fomalhaut => "Fomalhaut",
            Self::EtaPsc => "EtaPsc",
            Self::OmicronPsc => "OmicronPsc",
            Self::Alrescha => "Alrescha",
            Self::Sirius => "Sirius",
            Self::Canopus => "Canopus",
            Self::Rigel => "Rigel",
            Self::Procyon => "Procyon",
            Self::Capella => "Capella",
            Self::Bellatrix => "Bellatrix",
            Self::Mintaka => "Mintaka",
            Self::Alnilam => "Alnilam",
            Self::Alnitak => "Alnitak",
            Self::Saiph => "Saiph",
            Self::Wezen => "Wezen",
            Self::Adhara => "Adhara",
            Self::Mirzam => "Mirzam",
            Self::Aludra => "Aludra",
            Self::Menkib => "Menkib",
            Self::Phact => "Phact",
            Self::Naos => "Naos",
            Self::Alphard => "Alphard",
            Self::Gienah => "Gienah",
            Self::Minkar => "Minkar",
            Self::Algorab => "Algorab",
            Self::Polaris => "Polaris",
            Self::Agastya => "Agastya",
            Self::Arundhati => "Arundhati",
            Self::Lubdhaka => "Lubdhaka",
            Self::Trishankhu => "Trishankhu",
            Self::Prajapati => "Prajapati",
            Self::BrahmaHridaya => "BrahmaHridaya",
            Self::Apamvatsa => "Apamvatsa",
            Self::Achernar => "Achernar",
            Self::Ankaa => "Ankaa",
            Self::Eltanin => "Eltanin",
            Self::Indra => "Indra",
            Self::GalacticCenter => "GalacticCenter",
            Self::GalacticAntiCenter => "GalacticAntiCenter",
        }
    }

    /// Parse a `TaraId` from its canonical string name.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        ALL_TARA_IDS.iter().find(|id| id.as_str() == s).copied()
    }

    /// Look up a `TaraId` from its integer code.
    pub fn from_code(code: i32) -> Option<Self> {
        ALL_TARA_IDS.iter().find(|id| **id as i32 == code).copied()
    }

    /// Whether this is a galactic reference point (no proper motion).
    pub fn is_galactic_reference(self) -> bool {
        matches!(self, Self::GalacticCenter | Self::GalacticAntiCenter)
    }
}

/// All TaraId variants in definition order.
pub const ALL_TARA_IDS: &[TaraId] = &[
    // Yogataras
    TaraId::Sheratan,
    TaraId::FortyCetAlrescha,
    TaraId::Alcyone,
    TaraId::Aldebaran,
    TaraId::LambdaOri,
    TaraId::Betelgeuse,
    TaraId::Pollux,
    TaraId::DeltaCnc,
    TaraId::EpsilonHya,
    TaraId::Regulus,
    TaraId::DeltaLeo,
    TaraId::Denebola,
    TaraId::DeltaCrv,
    TaraId::Chitra,
    TaraId::Arcturus,
    TaraId::AlphaLib,
    TaraId::DeltaSco,
    TaraId::Antares,
    TaraId::LambdaSco,
    TaraId::DeltaSgr,
    TaraId::SigmaSgr,
    TaraId::Altair,
    TaraId::BetaDel,
    TaraId::LambdaAqr,
    TaraId::AlphaPeg,
    TaraId::GammaPeg,
    TaraId::ZetaPsc,
    TaraId::Vega,
    // Rashi constellation stars
    TaraId::Hamal,
    TaraId::Mesarthim,
    TaraId::ElNath,
    TaraId::Ain,
    TaraId::Merope,
    TaraId::Electra,
    TaraId::Taygeta,
    TaraId::Maia,
    TaraId::Atlas,
    TaraId::Castor,
    TaraId::Alhena,
    TaraId::Mebsuta,
    TaraId::Tejat,
    TaraId::Propus,
    TaraId::Acubens,
    TaraId::Altarf,
    TaraId::Praesepe,
    TaraId::Algieba,
    TaraId::Zosma,
    TaraId::Adhafera,
    TaraId::RasElased,
    TaraId::Algenubi,
    TaraId::Chertan,
    TaraId::Zavijava,
    TaraId::Porrima,
    TaraId::Auva,
    TaraId::Vindemiatrix,
    TaraId::Heze,
    TaraId::Zaniah,
    TaraId::Zubeneschamali,
    TaraId::Zubenelgenubi,
    TaraId::Brachium,
    TaraId::Shaula,
    TaraId::Sargas,
    TaraId::Dschubba,
    TaraId::Acrab,
    TaraId::Lesath,
    TaraId::AlNiyat,
    TaraId::AlniyatTau,
    TaraId::KausMedia,
    TaraId::KausAustralis,
    TaraId::KausBorealis,
    TaraId::Nunki,
    TaraId::Ascella,
    TaraId::Rukbat,
    TaraId::Arkab,
    TaraId::DenebAlgedi,
    TaraId::Dabih,
    TaraId::Algedi,
    TaraId::Nashira,
    TaraId::Sadalsuud,
    TaraId::Sadalmelik,
    TaraId::Skat,
    TaraId::Albali,
    TaraId::Ancha,
    TaraId::Fomalhaut,
    TaraId::EtaPsc,
    TaraId::OmicronPsc,
    TaraId::Alrescha,
    TaraId::Sirius,
    TaraId::Canopus,
    TaraId::Rigel,
    TaraId::Procyon,
    TaraId::Capella,
    TaraId::Bellatrix,
    TaraId::Mintaka,
    TaraId::Alnilam,
    TaraId::Alnitak,
    TaraId::Saiph,
    TaraId::Wezen,
    TaraId::Adhara,
    TaraId::Mirzam,
    TaraId::Aludra,
    TaraId::Menkib,
    TaraId::Phact,
    TaraId::Naos,
    TaraId::Alphard,
    TaraId::Gienah,
    TaraId::Minkar,
    TaraId::Algorab,
    // Special Vedic stars
    TaraId::Polaris,
    TaraId::Agastya,
    TaraId::Arundhati,
    TaraId::Lubdhaka,
    TaraId::Trishankhu,
    TaraId::Prajapati,
    TaraId::BrahmaHridaya,
    TaraId::Apamvatsa,
    TaraId::Achernar,
    TaraId::Ankaa,
    TaraId::Eltanin,
    TaraId::Indra,
    // Galactic reference points
    TaraId::GalacticCenter,
    TaraId::GalacticAntiCenter,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tara_ids_count() {
        assert_eq!(ALL_TARA_IDS.len(), 122);
    }

    #[test]
    fn from_str_roundtrip() {
        for &id in ALL_TARA_IDS {
            let name = id.as_str();
            let parsed = TaraId::from_str(name);
            assert_eq!(parsed, Some(id), "failed roundtrip for {name}");
        }
    }

    #[test]
    fn from_code_roundtrip() {
        for &id in ALL_TARA_IDS {
            let code = id as i32;
            let parsed = TaraId::from_code(code);
            assert_eq!(parsed, Some(id), "failed roundtrip for code {code}");
        }
    }

    #[test]
    fn no_duplicate_codes() {
        let mut codes: Vec<i32> = ALL_TARA_IDS.iter().map(|id| *id as i32).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), ALL_TARA_IDS.len(), "duplicate codes found");
    }

    #[test]
    fn categories() {
        assert_eq!(TaraId::Chitra.category(), TaraCategory::Yogatara);
        assert_eq!(TaraId::Sirius.category(), TaraCategory::RashiConstellation);
        assert_eq!(TaraId::Polaris.category(), TaraCategory::SpecialVedic);
        assert_eq!(
            TaraId::GalacticCenter.category(),
            TaraCategory::GalacticReference
        );
    }

    #[test]
    fn galactic_reference_flag() {
        assert!(TaraId::GalacticCenter.is_galactic_reference());
        assert!(TaraId::GalacticAntiCenter.is_galactic_reference());
        assert!(!TaraId::Chitra.is_galactic_reference());
    }
}
