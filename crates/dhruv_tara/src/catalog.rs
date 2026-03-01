//! Star catalog loading and lookup.
//!
//! Parses JSON catalog files with the schema described in the plan.
//! Hand-rolled parser for the flat catalog schema (no serde dependency).

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use crate::error::TaraError;
use crate::tara_id::{TaraCategory, TaraId};

const EMBEDDED_CATALOG_JSON: &str = include_str!("../../../kernels/data/hgca_tara.json");

static EMBEDDED_CATALOG: LazyLock<TaraCatalog> = LazyLock::new(|| {
    TaraCatalog::parse(EMBEDDED_CATALOG_JSON)
        .expect("embedded HGCA catalog must parse — this is a compile-time resource")
});

/// A single star entry from the catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct TaraEntry {
    /// TaraId for this star.
    pub id: TaraId,
    /// Bayer/Flamsteed designation (e.g., "alf Vir").
    pub bayer: String,
    /// Common name (e.g., "Spica").
    pub common_name: String,
    /// Hipparcos catalog number.
    pub hip_id: u32,
    /// Right ascension at reference epoch (ICRS, degrees).
    pub ra_deg: f64,
    /// Declination at reference epoch (ICRS, degrees).
    pub dec_deg: f64,
    /// Parallax (milliarcseconds).
    pub parallax_mas: f64,
    /// Proper motion in RA (μα* = μα cos δ, mas/yr).
    pub pm_ra_mas_yr: f64,
    /// Proper motion in Dec (mas/yr).
    pub pm_dec_mas_yr: f64,
    /// Radial velocity (km/s). 0.0 if unknown.
    pub radial_velocity_km_s: f64,
    /// Visual magnitude.
    pub v_mag: f64,
    /// Star category (derived from TaraId code range).
    pub category: TaraCategory,
    /// Nakshatra index (0-27) if this star is a yogatara. None for non-yogataras.
    pub nakshatra: Option<u8>,
    /// Rashi constellation name (e.g., "Aries", "Taurus") if this star is a rashi
    /// constellation star. None for yogataras, special stars, and galactic references.
    pub rashi_constellation: Option<&'static str>,
}

/// Immutable star catalog. Send + Sync after construction.
#[derive(Debug, Clone)]
pub struct TaraCatalog {
    /// Catalog source identifier (e.g., "HGCA_vEDR3").
    pub source: String,
    /// Reference epoch in Julian years (e.g., 2016.0).
    pub reference_epoch_jy: f64,
    /// Lookup by TaraId.
    entries: HashMap<TaraId, TaraEntry>,
}

impl TaraCatalog {
    /// Returns the embedded HGCA star catalog (HGCA J2016.0, compiled in).
    ///
    /// Parsed once on first access. Panics at initialization if the embedded
    /// JSON is corrupt (should never happen — it's a compile-time resource).
    pub fn embedded() -> &'static TaraCatalog {
        &EMBEDDED_CATALOG
    }

    /// Load a catalog from a JSON file on disk.
    pub fn load(path: &Path) -> Result<Self, TaraError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| TaraError::CatalogLoad(e.to_string()))?;
        Self::parse(&content)
    }

    /// Parse a catalog from a JSON string.
    pub fn parse(content: &str) -> Result<Self, TaraError> {
        parse_catalog_json(content)
    }

    /// Look up a star by its TaraId.
    pub fn get(&self, id: TaraId) -> Option<&TaraEntry> {
        self.entries.get(&id)
    }

    /// Number of stars in the catalog.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&TaraId, &TaraEntry)> {
        self.entries.iter()
    }
}

// ---- Hand-rolled JSON parser for the flat catalog schema ----

fn parse_catalog_json(content: &str) -> Result<TaraCatalog, TaraError> {
    let err = |msg: &str| TaraError::CatalogLoad(msg.to_string());

    let source = extract_string_field(content, "source").ok_or_else(|| err("missing 'source'"))?;
    let epoch_str = extract_number_field(content, "reference_epoch_jy")
        .ok_or_else(|| err("missing 'reference_epoch_jy'"))?;
    let reference_epoch_jy: f64 = epoch_str
        .parse()
        .map_err(|_| err("invalid reference_epoch_jy"))?;

    let stars_start = content
        .find("\"stars\"")
        .ok_or_else(|| err("missing 'stars' array"))?;
    let arr_start = content[stars_start..]
        .find('[')
        .ok_or_else(|| err("missing '[' for stars array"))?
        + stars_start;

    let mut entries = HashMap::new();
    let mut pos = arr_start + 1;
    let bytes = content.as_bytes();

    loop {
        // Find next '{' or ']'
        while pos < bytes.len() && bytes[pos] != b'{' && bytes[pos] != b']' {
            pos += 1;
        }
        if pos >= bytes.len() || bytes[pos] == b']' {
            break;
        }

        // Find matching '}'
        let obj_start = pos;
        let mut depth = 0i32;
        let mut obj_end = pos;
        for (i, &b) in bytes[pos..].iter().enumerate() {
            if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    obj_end = pos + i;
                    break;
                }
            }
        }
        let obj_str = &content[obj_start..=obj_end];

        if let Some(entry) = parse_star_entry(obj_str) {
            entries.insert(entry.id, entry);
        }

        pos = obj_end + 1;
    }

    Ok(TaraCatalog {
        source,
        reference_epoch_jy,
        entries,
    })
}

fn parse_star_entry(obj: &str) -> Option<TaraEntry> {
    let id_str = extract_string_field(obj, "id")?;
    let id = TaraId::from_str(&id_str)?;

    let bayer = extract_string_field(obj, "bayer").unwrap_or_default();
    let common_name = extract_string_field(obj, "common_name").unwrap_or_default();
    let hip_id: u32 = extract_number_field(obj, "hip_id")?.parse().ok()?;
    let ra_deg: f64 = extract_number_field(obj, "ra_deg")?.parse().ok()?;
    let dec_deg: f64 = extract_number_field(obj, "dec_deg")?.parse().ok()?;
    let parallax_mas: f64 = extract_number_field(obj, "parallax_mas")?.parse().ok()?;
    let pm_ra_mas_yr: f64 = extract_number_field(obj, "pm_ra_mas_yr")?.parse().ok()?;
    let pm_dec_mas_yr: f64 = extract_number_field(obj, "pm_dec_mas_yr")?.parse().ok()?;
    let radial_velocity_km_s: f64 = extract_number_field(obj, "radial_velocity_km_s")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let v_mag: f64 = extract_number_field(obj, "v_mag")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let category = id.category();
    let nakshatra = if category == TaraCategory::Yogatara {
        Some(id as i32 as u8)
    } else {
        None
    };
    let rashi_constellation = rashi_for_id(id);

    Some(TaraEntry {
        id,
        bayer,
        common_name,
        hip_id,
        ra_deg,
        dec_deg,
        parallax_mas,
        pm_ra_mas_yr,
        pm_dec_mas_yr,
        radial_velocity_km_s,
        v_mag,
        category,
        nakshatra,
        rashi_constellation,
    })
}

/// Map rashi constellation stars to their constellation name.
fn rashi_for_id(id: TaraId) -> Option<&'static str> {
    use TaraId::*;
    match id {
        Hamal | Mesarthim => Some("Aries"),
        ElNath | Ain | Merope | Electra | Taygeta | Maia | Atlas => Some("Taurus"),
        Castor | Alhena | Mebsuta | Tejat | Propus => Some("Gemini"),
        Acubens | Altarf | Praesepe => Some("Cancer"),
        Algieba | Zosma | Adhafera | RasElased | Algenubi | Chertan => Some("Leo"),
        Zavijava | Porrima | Auva | Vindemiatrix | Heze | Zaniah => Some("Virgo"),
        Zubeneschamali | Zubenelgenubi | Brachium => Some("Libra"),
        Shaula | Sargas | Dschubba | Acrab | Lesath | AlNiyat | AlniyatTau => Some("Scorpio"),
        KausMedia | KausAustralis | KausBorealis | Nunki | Ascella | Rukbat | Arkab => {
            Some("Sagittarius")
        }
        DenebAlgedi | Dabih | Algedi | Nashira => Some("Capricorn"),
        Sadalsuud | Sadalmelik | Skat | Albali | Ancha => Some("Aquarius"),
        Fomalhaut | EtaPsc | OmicronPsc | Alrescha => Some("Pisces"),
        // Other rashi constellation stars (cross-constellation)
        Sirius | Canopus | Rigel | Procyon | Capella | Bellatrix | Mintaka | Alnilam | Alnitak
        | Saiph | Wezen | Adhara | Mirzam | Aludra | Menkib | Phact | Naos | Alphard | Gienah
        | Minkar | Algorab => Some("Other"),
        _ => None,
    }
}

/// Extract a string value for a given key from a JSON object.
fn extract_string_field(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let key_pos = obj.find(&pattern)?;
    let after_key = &obj[key_pos + pattern.len()..];
    // Skip whitespace and colon
    let after_colon = after_key.find(':')? + 1;
    let value_region = after_key[after_colon..].trim_start();
    if !value_region.starts_with('"') {
        return None;
    }
    let value_start = 1; // skip opening quote
    let value_end = value_region[value_start..].find('"')?;
    Some(value_region[value_start..value_start + value_end].to_string())
}

/// Extract a numeric value (as string) for a given key from a JSON object.
fn extract_number_field(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let key_pos = obj.find(&pattern)?;
    let after_key = &obj[key_pos + pattern.len()..];
    let after_colon = after_key.find(':')? + 1;
    let value_region = after_key[after_colon..].trim_start();
    // Number ends at comma, whitespace, or closing brace
    let end = value_region
        .find([',', '}', ']', '\n', '\r'])
        .unwrap_or(value_region.len());
    let num_str = value_region[..end].trim();
    if num_str.is_empty() {
        return None;
    }
    Some(num_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_catalog_json() -> &'static str {
        r#"{
  "source": "TEST",
  "reference_epoch_jy": 2016.0,
  "reference_frame": "ICRS",
  "stars": [
    {
      "id": "Chitra",
      "bayer": "alf Vir",
      "common_name": "Spica",
      "hip_id": 65474,
      "ra_deg": 201.29825,
      "dec_deg": -11.16132,
      "parallax_mas": 13.06,
      "pm_ra_mas_yr": -42.50,
      "pm_dec_mas_yr": -31.73,
      "radial_velocity_km_s": 1.0,
      "v_mag": 0.97
    },
    {
      "id": "Arcturus",
      "bayer": "alf Boo",
      "common_name": "Arcturus",
      "hip_id": 69673,
      "ra_deg": 213.91530,
      "dec_deg": 19.18240,
      "parallax_mas": 88.83,
      "pm_ra_mas_yr": -1093.45,
      "pm_dec_mas_yr": -1999.40,
      "radial_velocity_km_s": -5.19,
      "v_mag": -0.05
    }
  ]
}"#
    }

    #[test]
    fn parse_minimal_catalog() {
        let catalog = TaraCatalog::parse(minimal_catalog_json()).unwrap();
        assert_eq!(catalog.source, "TEST");
        assert!((catalog.reference_epoch_jy - 2016.0).abs() < 1e-10);
        assert_eq!(catalog.len(), 2);
    }

    #[test]
    fn lookup_by_id() {
        let catalog = TaraCatalog::parse(minimal_catalog_json()).unwrap();
        let spica = catalog.get(TaraId::Chitra).unwrap();
        assert_eq!(spica.hip_id, 65474);
        assert!((spica.ra_deg - 201.29825).abs() < 1e-5);
        assert_eq!(spica.common_name, "Spica");
    }

    #[test]
    fn missing_star_returns_none() {
        let catalog = TaraCatalog::parse(minimal_catalog_json()).unwrap();
        assert!(catalog.get(TaraId::Polaris).is_none());
    }

    #[test]
    fn empty_stars_array() {
        let json = r#"{"source":"EMPTY","reference_epoch_jy":2016.0,"stars":[]}"#;
        let catalog = TaraCatalog::parse(json).unwrap();
        assert_eq!(catalog.len(), 0);
        assert!(catalog.is_empty());
    }

    #[test]
    fn missing_source_field() {
        let json = r#"{"reference_epoch_jy":2016.0,"stars":[]}"#;
        let result = TaraCatalog::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn embedded_catalog_parses() {
        let cat = TaraCatalog::embedded();
        assert!(!cat.is_empty());
        assert_eq!(cat.source, "HGCA_EDR3");
        assert!((cat.reference_epoch_jy - 2016.0).abs() < 1e-10);
    }

    #[test]
    fn embedded_catalog_has_anchor_stars() {
        let cat = TaraCatalog::embedded();
        for id in [
            TaraId::Chitra,
            TaraId::Aldebaran,
            TaraId::DeltaCnc,
            TaraId::LambdaSco,
        ] {
            let entry = cat.get(id).unwrap_or_else(|| panic!("{id:?} must be in embedded catalog"));
            assert!(entry.ra_deg > 0.0, "{id:?} RA");
            assert!(entry.pm_ra_mas_yr.abs() > 0.0 || entry.pm_dec_mas_yr.abs() > 0.0, "{id:?} PM");
        }
    }

    #[test]
    fn embedded_catalog_star_count() {
        let cat = TaraCatalog::embedded();
        assert!(cat.len() >= 100, "embedded catalog has {} stars, expected >= 100", cat.len());
    }

    #[test]
    fn unknown_star_id_skipped() {
        let json = r#"{
  "source": "TEST",
  "reference_epoch_jy": 2016.0,
  "stars": [
    {
      "id": "UnknownStar",
      "bayer": "xxx",
      "common_name": "Unknown",
      "hip_id": 99999,
      "ra_deg": 0.0,
      "dec_deg": 0.0,
      "parallax_mas": 1.0,
      "pm_ra_mas_yr": 0.0,
      "pm_dec_mas_yr": 0.0,
      "radial_velocity_km_s": 0.0,
      "v_mag": 10.0
    }
  ]
}"#;
        let catalog = TaraCatalog::parse(json).unwrap();
        assert_eq!(catalog.len(), 0); // unknown star silently skipped
    }
}
