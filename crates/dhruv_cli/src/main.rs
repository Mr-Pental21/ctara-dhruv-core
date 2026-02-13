use std::path::PathBuf;

use clap::{Parser, Subcommand};
use dhruv_core::{Body, Engine, EngineConfig};
use dhruv_frames::nutation_iau2000b;
use dhruv_search::conjunction_types::{ConjunctionConfig, ConjunctionEvent};
use dhruv_search::grahan_types::GrahanConfig;
use dhruv_search::sankranti_types::SankrantiConfig;
use dhruv_search::stationary_types::StationaryConfig;
use dhruv_time::{EopKernel, UtcTime, calendar_to_jd};
use dhruv_vedic_base::BhavaConfig;
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig, RiseSetResult};
use dhruv_vedic_base::{
    AyanamshaSystem, LunarNode, NodeMode, Rashi, ayanamsha_deg, deg_to_dms, jd_tdb_to_centuries,
    nakshatra_from_longitude, nakshatra_from_tropical, nakshatra28_from_longitude,
    nakshatra28_from_tropical, rashi_from_longitude, rashi_from_tropical,
};

#[derive(Parser)]
#[command(name = "dhruv", about = "Dhruv ephemeris CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rashi from sidereal longitude
    Rashi {
        /// Sidereal ecliptic longitude in degrees
        lon: f64,
    },
    /// Nakshatra from sidereal longitude
    Nakshatra {
        /// Sidereal ecliptic longitude in degrees
        lon: f64,
        /// Scheme: 27 (default) or 28
        #[arg(long, default_value = "27")]
        scheme: u32,
    },
    /// Rashi from tropical longitude + ayanamsha
    RashiTropical {
        /// Tropical ecliptic longitude in degrees
        lon: f64,
        /// Ayanamsha system code (0-19)
        #[arg(long)]
        ayanamsha: i32,
        /// Julian Date TDB
        #[arg(long)]
        jd: f64,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
    },
    /// Nakshatra from tropical longitude + ayanamsha
    NakshatraTropical {
        /// Tropical ecliptic longitude in degrees
        lon: f64,
        /// Ayanamsha system code (0-19)
        #[arg(long)]
        ayanamsha: i32,
        /// Julian Date TDB
        #[arg(long)]
        jd: f64,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Scheme: 27 (default) or 28
        #[arg(long, default_value = "27")]
        scheme: u32,
    },
    /// Convert degrees to DMS
    Dms {
        /// Angle in decimal degrees
        deg: f64,
    },
    /// Find next Purnima (full moon)
    NextPurnima {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Path to SPK kernel (de442s.bsp)
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel (naif0012.tls)
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next Amavasya (new moon)
    NextAmavasya {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next Sankranti (Sun entering a rashi)
    NextSankranti {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Masa (lunar month) for a date
    Masa {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Ayana (Uttarayana/Dakshinayana) for a date
    Ayana {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Varsha (60-year samvatsara cycle) for a date
    Varsha {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Tithi (lunar day) for a date
    Tithi {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Karana (half-tithi) for a date
    Karana {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Yoga (luni-solar yoga) for a date
    Yoga {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Moon's Nakshatra (27-scheme) with start/end times for a date
    MoonNakshatra {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Determine the Vaar (Vedic weekday) for a date and location
    Vaar {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Determine the Hora (planetary hour) for a date and location
    Hora {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Determine the Ghatika (1-60) for a date and location
    Ghatika {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute all 16 sphutas for a date and location
    Sphutas {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute all 8 special lagnas for a date and location
    SpecialLagnas {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute all 12 arudha padas for a date and location
    ArudhaPadas {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Combined panchang: tithi, karana, yoga, vaar, hora, ghatika
    Panchang {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Include calendar elements (masa, ayana, varsha)
        #[arg(long)]
        calendar: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute Ashtakavarga (BAV + SAV) for a date and location
    Ashtakavarga {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute all 11 upagrahas for a date and location
    Upagrahas {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute comprehensive graha positions
    GrahaPositions {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Include nakshatra and pada
        #[arg(long)]
        nakshatra: bool,
        /// Include lagna (ascendant)
        #[arg(long)]
        lagna: bool,
        /// Include outer planets (Uranus, Neptune, Pluto)
        #[arg(long)]
        outer: bool,
        /// Include bhava placement
        #[arg(long)]
        bhava: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute curated sensitive points (bindus) with optional nakshatra/bhava
    CoreBindus {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Include nakshatra and pada
        #[arg(long)]
        nakshatra: bool,
        /// Include bhava placement
        #[arg(long)]
        bhava: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute graha drishti (planetary aspects) with virupa strength
    Drishti {
        /// UTC datetime (YYYY-MM-DDThh:mm:ssZ)
        #[arg(long)]
        date: String,
        /// Latitude in degrees (north positive)
        #[arg(long)]
        lat: f64,
        /// Longitude in degrees (east positive)
        #[arg(long)]
        lon: f64,
        /// Altitude in meters (default 0)
        #[arg(long, default_value = "0")]
        alt: f64,
        /// Ayanamsha system code (0-19, default 0=Lahiri)
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        /// Apply nutation correction
        #[arg(long)]
        nutation: bool,
        /// Include graha-to-bhava-cusp drishti
        #[arg(long)]
        bhava: bool,
        /// Include graha-to-lagna drishti
        #[arg(long)]
        lagna: bool,
        /// Include graha-to-core-bindus drishti
        #[arg(long)]
        bindus: bool,
        /// Path to SPK kernel
        #[arg(long)]
        bsp: PathBuf,
        /// Path to leap second kernel
        #[arg(long)]
        lsk: PathBuf,
        /// Path to IERS EOP file (finals2000A.all)
        #[arg(long)]
        eop: PathBuf,
    },
    /// Find previous Purnima (full moon)
    PrevPurnima {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous Amavasya (new moon)
    PrevAmavasya {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous Sankranti
    PrevSankranti {
        #[arg(long)]
        date: String,
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        #[arg(long)]
        nutation: bool,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search Purnimas in a date range
    SearchPurnimas {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search Amavasyas in a date range
    SearchAmavasyas {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search Sankrantis in a date range
    SearchSankrantis {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        #[arg(long)]
        nutation: bool,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next entry of Sun into a specific Rashi
    NextSpecificSankranti {
        #[arg(long)]
        date: String,
        /// Rashi index (0=Mesha .. 11=Meena)
        #[arg(long)]
        rashi: u8,
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        #[arg(long)]
        nutation: bool,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous entry of Sun into a specific Rashi
    PrevSpecificSankranti {
        #[arg(long)]
        date: String,
        /// Rashi index (0=Mesha .. 11=Meena)
        #[arg(long)]
        rashi: u8,
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        #[arg(long)]
        nutation: bool,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Compute ayanamsha for a date
    AyanamshaCompute {
        #[arg(long)]
        date: String,
        #[arg(long, default_value = "0")]
        ayanamsha: i32,
        #[arg(long)]
        nutation: bool,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Compute nutation (dpsi, deps) for a date
    NutationCompute {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Compute sunrise/sunset and twilight events
    Sunrise {
        #[arg(long)]
        date: String,
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
        #[arg(long, default_value = "0")]
        alt: f64,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute bhava (house) cusps
    Bhavas {
        #[arg(long)]
        date: String,
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
        #[arg(long, default_value = "0")]
        alt: f64,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute Lagna (Ascendant), MC, and RAMC
    LagnaCompute {
        #[arg(long)]
        date: String,
        #[arg(long)]
        lat: f64,
        #[arg(long)]
        lon: f64,
        #[arg(long, default_value = "0")]
        alt: f64,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
        #[arg(long)]
        eop: PathBuf,
    },
    /// Compute Rahu/Ketu (lunar node) longitude
    LunarNode {
        #[arg(long)]
        date: String,
        /// Node: rahu or ketu
        #[arg(long, default_value = "rahu")]
        node: String,
        /// Mode: mean or true
        #[arg(long, default_value = "mean")]
        mode: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next conjunction between two bodies
    NextConjunction {
        #[arg(long)]
        date: String,
        /// NAIF body code for first body (e.g. 10=Sun, 301=Moon)
        #[arg(long)]
        body1: i32,
        /// NAIF body code for second body
        #[arg(long)]
        body2: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous conjunction between two bodies
    PrevConjunction {
        #[arg(long)]
        date: String,
        #[arg(long)]
        body1: i32,
        #[arg(long)]
        body2: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search conjunctions between two bodies in a date range
    SearchConjunctions {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        body1: i32,
        #[arg(long)]
        body2: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next lunar eclipse
    NextChandraGrahan {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous lunar eclipse
    PrevChandraGrahan {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search lunar eclipses in a date range
    SearchChandraGrahan {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next solar eclipse
    NextSuryaGrahan {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous solar eclipse
    PrevSuryaGrahan {
        #[arg(long)]
        date: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search solar eclipses in a date range
    SearchSuryaGrahan {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next stationary point of a planet
    NextStationary {
        #[arg(long)]
        date: String,
        /// NAIF body code (e.g. 499=Mars, 599=Jupiter)
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous stationary point of a planet
    PrevStationary {
        #[arg(long)]
        date: String,
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search stationary points of a planet in a date range
    SearchStationary {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find next max-speed event of a planet
    NextMaxSpeed {
        #[arg(long)]
        date: String,
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Find previous max-speed event of a planet
    PrevMaxSpeed {
        #[arg(long)]
        date: String,
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
    /// Search max-speed events of a planet in a date range
    SearchMaxSpeed {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        body: i32,
        #[arg(long)]
        bsp: PathBuf,
        #[arg(long)]
        lsk: PathBuf,
    },
}

fn aya_system_from_code(code: i32) -> Option<AyanamshaSystem> {
    let all = AyanamshaSystem::all();
    let idx = usize::try_from(code).ok()?;
    all.get(idx).copied()
}

fn parse_utc(s: &str) -> Result<UtcTime, String> {
    // Parse "YYYY-MM-DDThh:mm:ssZ" or "YYYY-MM-DDThh:mm:ss"
    let s = s.trim_end_matches('Z');
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return Err(format!("expected YYYY-MM-DDThh:mm:ssZ, got {s}"));
    }
    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return Err(format!("invalid date/time format: {s}"));
    }
    let year: i32 = date_parts[0].parse().map_err(|e| format!("{e}"))?;
    let month: u32 = date_parts[1].parse().map_err(|e| format!("{e}"))?;
    let day: u32 = date_parts[2].parse().map_err(|e| format!("{e}"))?;
    let hour: u32 = time_parts[0].parse().map_err(|e| format!("{e}"))?;
    let minute: u32 = time_parts[1].parse().map_err(|e| format!("{e}"))?;
    let second: f64 = time_parts[2].parse().map_err(|e| format!("{e}"))?;
    Ok(UtcTime::new(year, month, day, hour, minute, second))
}

fn load_engine(bsp: &PathBuf, lsk: &PathBuf) -> Engine {
    let config = EngineConfig::with_single_spk(bsp.clone(), lsk.clone(), 256, true);
    Engine::new(config).unwrap_or_else(|e| {
        eprintln!("Failed to load engine: {e}");
        std::process::exit(1);
    })
}

fn require_aya_system(code: i32) -> AyanamshaSystem {
    aya_system_from_code(code).unwrap_or_else(|| {
        eprintln!("Invalid ayanamsha code: {code} (0-19)");
        std::process::exit(1);
    })
}

fn load_eop(path: &PathBuf) -> EopKernel {
    EopKernel::load(path).unwrap_or_else(|e| {
        eprintln!("Failed to load EOP: {e}");
        std::process::exit(1);
    })
}

fn require_body(code: i32) -> Body {
    Body::from_code(code).unwrap_or_else(|| {
        eprintln!("Invalid body code: {code}");
        std::process::exit(1);
    })
}

fn utc_to_jd_utc(utc: &UtcTime) -> f64 {
    let day_frac = utc.day as f64
        + utc.hour as f64 / 24.0
        + utc.minute as f64 / 1440.0
        + utc.second / 86_400.0;
    calendar_to_jd(utc.year, utc.month, day_frac)
}

fn rashi_from_index(idx: u8) -> Rashi {
    dhruv_vedic_base::ALL_RASHIS
        .get(idx as usize)
        .copied()
        .unwrap_or_else(|| {
            eprintln!("Invalid rashi index: {idx} (0-11)");
            std::process::exit(1);
        })
}

fn parse_lunar_node(s: &str) -> LunarNode {
    match s.to_lowercase().as_str() {
        "rahu" => LunarNode::Rahu,
        "ketu" => LunarNode::Ketu,
        _ => {
            eprintln!("Invalid node: {s} (rahu or ketu)");
            std::process::exit(1);
        }
    }
}

fn parse_node_mode(s: &str) -> NodeMode {
    match s.to_lowercase().as_str() {
        "mean" => NodeMode::Mean,
        "true" => NodeMode::True,
        _ => {
            eprintln!("Invalid mode: {s} (mean or true)");
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rashi { lon } => {
            let info = rashi_from_longitude(lon);
            let dms = info.dms;
            println!(
                "{} ({}) - {} deg {} min {:.1} sec ({:.4} deg in rashi)",
                info.rashi.name(),
                info.rashi.western_name(),
                dms.degrees,
                dms.minutes,
                dms.seconds,
                info.degrees_in_rashi
            );
        }

        Commands::Nakshatra { lon, scheme } => match scheme {
            27 => {
                let info = nakshatra_from_longitude(lon);
                println!(
                    "{} (index {}) - Pada {} ({:.4} deg in nakshatra, {:.4} deg in pada)",
                    info.nakshatra.name(),
                    info.nakshatra_index,
                    info.pada,
                    info.degrees_in_nakshatra,
                    info.degrees_in_pada
                );
            }
            28 => {
                let info = nakshatra28_from_longitude(lon);
                println!(
                    "{} (index {}) - Pada {} ({:.4} deg in nakshatra)",
                    info.nakshatra.name(),
                    info.nakshatra_index,
                    info.pada,
                    info.degrees_in_nakshatra
                );
            }
            _ => {
                eprintln!("Invalid scheme: {scheme}. Use 27 or 28.");
                std::process::exit(1);
            }
        },

        Commands::RashiTropical {
            lon,
            ayanamsha,
            jd,
            nutation,
        } => {
            let system = require_aya_system(ayanamsha);
            let t = jd_tdb_to_centuries(jd);
            let aya = ayanamsha_deg(system, t, nutation);
            let info = rashi_from_tropical(lon, system, jd, nutation);
            let dms = info.dms;
            println!("Ayanamsha: {:.4} deg", aya);
            println!("Sidereal: {:.4} deg", lon - aya);
            println!(
                "{} ({}) - {} deg {} min {:.1} sec ({:.4} deg in rashi)",
                info.rashi.name(),
                info.rashi.western_name(),
                dms.degrees,
                dms.minutes,
                dms.seconds,
                info.degrees_in_rashi
            );
        }

        Commands::NakshatraTropical {
            lon,
            ayanamsha,
            jd,
            nutation,
            scheme,
        } => {
            let system = require_aya_system(ayanamsha);
            let t = jd_tdb_to_centuries(jd);
            let aya = ayanamsha_deg(system, t, nutation);
            println!("Ayanamsha: {:.4} deg", aya);
            println!("Sidereal: {:.4} deg", lon - aya);
            match scheme {
                27 => {
                    let info = nakshatra_from_tropical(lon, system, jd, nutation);
                    println!(
                        "{} (index {}) - Pada {} ({:.4} deg in nakshatra, {:.4} deg in pada)",
                        info.nakshatra.name(),
                        info.nakshatra_index,
                        info.pada,
                        info.degrees_in_nakshatra,
                        info.degrees_in_pada
                    );
                }
                28 => {
                    let info = nakshatra28_from_tropical(lon, system, jd, nutation);
                    println!(
                        "{} (index {}) - Pada {} ({:.4} deg in nakshatra)",
                        info.nakshatra.name(),
                        info.nakshatra_index,
                        info.pada,
                        info.degrees_in_nakshatra
                    );
                }
                _ => {
                    eprintln!("Invalid scheme: {scheme}. Use 27 or 28.");
                    std::process::exit(1);
                }
            }
        }

        Commands::Dms { deg } => {
            let d = deg_to_dms(deg);
            println!("{} deg {} min {:.2} sec", d.degrees, d.minutes, d.seconds);
        }

        Commands::NextPurnima { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::next_purnima(&engine, &utc) {
                Ok(Some(ev)) => {
                    println!("Next Purnima: {}", ev.utc);
                    println!(
                        "  Moon lon: {:.4} deg  Sun lon: {:.4} deg",
                        ev.moon_longitude_deg, ev.sun_longitude_deg
                    );
                }
                Ok(None) => println!("No Purnima found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextAmavasya { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::next_amavasya(&engine, &utc) {
                Ok(Some(ev)) => {
                    println!("Next Amavasya: {}", ev.utc);
                    println!(
                        "  Moon lon: {:.4} deg  Sun lon: {:.4} deg",
                        ev.moon_longitude_deg, ev.sun_longitude_deg
                    );
                }
                Ok(None) => println!("No Amavasya found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextSankranti {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::next_sankranti(&engine, &utc, &config) {
                Ok(Some(ev)) => {
                    println!(
                        "Next Sankranti: {} ({})",
                        ev.rashi.name(),
                        ev.rashi.western_name()
                    );
                    println!("  Time: {}", ev.utc);
                    println!(
                        "  Sidereal lon: {:.4} deg  Tropical lon: {:.4} deg",
                        ev.sun_sidereal_longitude_deg, ev.sun_tropical_longitude_deg
                    );
                }
                Ok(None) => println!("No Sankranti found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Masa {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::masa_for_date(&engine, &utc, &config) {
                Ok(info) => {
                    let adhika_str = if info.adhika { " (Adhika)" } else { "" };
                    println!("Masa: {}{}", info.masa.name(), adhika_str);
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Ayana {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::ayana_for_date(&engine, &utc, &config) {
                Ok(info) => {
                    println!("Ayana: {}", info.ayana.name());
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Varsha {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::varsha_for_date(&engine, &utc, &config) {
                Ok(info) => {
                    println!(
                        "Samvatsara: {} (#{} in 60-year cycle)",
                        info.samvatsara.name(),
                        info.order
                    );
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Tithi { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::tithi_for_date(&engine, &utc) {
                Ok(info) => {
                    println!("Tithi: {} (index {})", info.tithi.name(), info.tithi_index);
                    println!(
                        "  Paksha: {}  Tithi in paksha: {}",
                        info.paksha.name(),
                        info.tithi_in_paksha
                    );
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Karana { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::karana_for_date(&engine, &utc) {
                Ok(info) => {
                    println!(
                        "Karana: {} (sequence index {})",
                        info.karana.name(),
                        info.karana_index
                    );
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Yoga {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::yoga_for_date(&engine, &utc, &config) {
                Ok(info) => {
                    println!("Yoga: {} (index {})", info.yoga.name(), info.yoga_index);
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::MoonNakshatra {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::nakshatra_for_date(&engine, &utc, &config) {
                Ok(info) => {
                    println!(
                        "Nakshatra: {} (index {}, pada {})",
                        info.nakshatra.name(),
                        info.nakshatra_index,
                        info.pada
                    );
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Vaar {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            match dhruv_search::vaar_for_date(&engine, &eop_kernel, &utc, &location, &rs_config) {
                Ok(info) => {
                    println!("Vaar: {}", info.vaar.name());
                    println!("  Start (sunrise): {}", info.start);
                    println!("  End (next sunrise): {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Hora {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            match dhruv_search::hora_for_date(&engine, &eop_kernel, &utc, &location, &rs_config) {
                Ok(info) => {
                    println!(
                        "Hora: {} (position {} of 24)",
                        info.hora.name(),
                        info.hora_index
                    );
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Ghatika {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            match dhruv_search::ghatika_for_date(&engine, &eop_kernel, &utc, &location, &rs_config)
            {
                Ok(info) => {
                    println!("Ghatika: {}/60", info.value);
                    println!("  Start: {}", info.start);
                    println!("  End:   {}", info.end);
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Sphutas {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);

            // Get graha sidereal longitudes
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let graha_lons =
                dhruv_search::graha_sidereal_longitudes(&engine, jd_tdb, system, nutation)
                    .unwrap_or_else(|e| {
                        eprintln!("Error computing graha longitudes: {e}");
                        std::process::exit(1);
                    });

            // Get lagna (sidereal)
            let jd_utc = jd_tdb; // approximate; for more precision would use LSK
            let asc_rad =
                dhruv_vedic_base::lagna_longitude_rad(engine.lsk(), &eop_kernel, &location, jd_utc)
                    .unwrap_or_else(|e| {
                        eprintln!("Error computing lagna: {e}");
                        std::process::exit(1);
                    });
            let t = dhruv_vedic_base::jd_tdb_to_centuries(jd_tdb);
            let aya = dhruv_vedic_base::ayanamsha_deg(system, t, nutation);
            let lagna_sid = (asc_rad.to_degrees() - aya).rem_euclid(360.0);

            // Get 8th lord longitude
            let lagna_rashi_idx = (lagna_sid / 30.0).floor() as u8;
            let eighth_rashi_idx = dhruv_vedic_base::nth_rashi_from(lagna_rashi_idx, 8);
            let eighth_lord = dhruv_vedic_base::rashi_lord_by_index(eighth_rashi_idx).unwrap();
            let eighth_lord_lon = graha_lons.longitude(eighth_lord);

            // Build sphuta inputs (gulika = 0 for now, as it requires upagraha computation)
            let inputs = dhruv_vedic_base::SphutalInputs {
                sun: graha_lons.longitude(dhruv_vedic_base::Graha::Surya),
                moon: graha_lons.longitude(dhruv_vedic_base::Graha::Chandra),
                mars: graha_lons.longitude(dhruv_vedic_base::Graha::Mangal),
                jupiter: graha_lons.longitude(dhruv_vedic_base::Graha::Guru),
                venus: graha_lons.longitude(dhruv_vedic_base::Graha::Shukra),
                rahu: graha_lons.longitude(dhruv_vedic_base::Graha::Rahu),
                lagna: lagna_sid,
                eighth_lord: eighth_lord_lon,
                gulika: 0.0,
            };

            let results = dhruv_vedic_base::all_sphutas(&inputs);
            println!("Sphutas for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);
            println!(
                "Graha longitudes (sidereal, aya code={} {}):",
                ayanamsha,
                if nutation { "+nutation" } else { "" }
            );
            for graha in dhruv_vedic_base::graha::ALL_GRAHAS {
                println!("  {:8} {:>8.4}°", graha.name(), graha_lons.longitude(graha));
            }
            println!("  {:8} {:>8.4}°\n", "Lagna", lagna_sid);
            println!("Sphutas:");
            for (sphuta, lon) in &results {
                let rashi_info = dhruv_vedic_base::rashi_from_longitude(*lon);
                println!(
                    "  {:24} {:>8.4}° ({} {}°{:02}'{:04.1}\")",
                    sphuta.name(),
                    lon,
                    rashi_info.rashi.name(),
                    rashi_info.dms.degrees,
                    rashi_info.dms.minutes,
                    rashi_info.dms.seconds,
                );
            }
            println!("\nNote: Gulika=0° (placeholder until upagraha computation is available).");
            println!(
                "  TriSphuta, ChatusSphuta, PanchaSphuta, SookshmaTrisphuta depend on Gulika."
            );
        }

        Commands::SpecialLagnas {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bsp,
            lsk,
            eop,
        } => {
            let system = aya_system_from_code(ayanamsha)
                .unwrap_or_else(|| panic!("Invalid ayanamsha code: {ayanamsha}"));
            let utc = parse_utc(&date).unwrap_or_else(|e| panic!("Invalid date: {e}"));
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            let config = SankrantiConfig::new(system, nutation);

            let result = dhruv_search::special_lagnas_for_date(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &rs_config,
                &config,
            )
            .unwrap_or_else(|e| panic!("special_lagnas_for_date failed: {e}"));

            println!(
                "Special Lagnas for {} at {:.4}°N, {:.4}°E\n",
                date, lat, lon
            );
            println!("  Bhava Lagna:     {:>10.4}°", result.bhava_lagna);
            println!("  Hora Lagna:      {:>10.4}°", result.hora_lagna);
            println!("  Ghati Lagna:     {:>10.4}°", result.ghati_lagna);
            println!("  Vighati Lagna:   {:>10.4}°", result.vighati_lagna);
            println!("  Varnada Lagna:   {:>10.4}°", result.varnada_lagna);
            println!("  Sree Lagna:      {:>10.4}°", result.sree_lagna);
            println!("  Pranapada Lagna: {:>10.4}°", result.pranapada_lagna);
            println!("  Indu Lagna:      {:>10.4}°", result.indu_lagna);
        }

        Commands::ArudhaPadas {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let bhava_config = BhavaConfig::default();
            let aya_config = SankrantiConfig::new(system, nutation);

            let results = dhruv_search::arudha_padas_for_date(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &bhava_config,
                &aya_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!("Arudha Padas for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);
            for r in &results {
                let rashi_info = dhruv_vedic_base::rashi_from_longitude(r.longitude_deg);
                println!(
                    "  {:16} {:>8.4}° ({} {}°{:02}'{:04.1}\")",
                    r.pada.name(),
                    r.longitude_deg,
                    rashi_info.rashi.name(),
                    rashi_info.dms.degrees,
                    rashi_info.dms.minutes,
                    rashi_info.dms.seconds,
                );
            }
        }

        Commands::Panchang {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            calendar,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::panchang_for_date(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &rs_config,
                &config,
                calendar,
            ) {
                Ok(info) => {
                    println!("Panchang for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);
                    println!(
                        "Tithi:    {} (index {})",
                        info.tithi.tithi.name(),
                        info.tithi.tithi_index
                    );
                    println!(
                        "  Paksha: {}  Tithi in paksha: {}",
                        info.tithi.paksha.name(),
                        info.tithi.tithi_in_paksha
                    );
                    println!("  Start:  {}  End: {}", info.tithi.start, info.tithi.end);
                    println!(
                        "Karana:   {} (sequence {})",
                        info.karana.karana.name(),
                        info.karana.karana_index
                    );
                    println!("  Start:  {}  End: {}", info.karana.start, info.karana.end);
                    println!(
                        "Yoga:     {} (index {})",
                        info.yoga.yoga.name(),
                        info.yoga.yoga_index
                    );
                    println!("  Start:  {}  End: {}", info.yoga.start, info.yoga.end);
                    println!("Vaar:     {}", info.vaar.vaar.name());
                    println!("  Start:  {}  End: {}", info.vaar.start, info.vaar.end);
                    println!(
                        "Hora:     {} (position {} of 24)",
                        info.hora.hora.name(),
                        info.hora.hora_index
                    );
                    println!("  Start:  {}  End: {}", info.hora.start, info.hora.end);
                    println!("Ghatika:  {}/60", info.ghatika.value);
                    println!(
                        "  Start:  {}  End: {}",
                        info.ghatika.start, info.ghatika.end
                    );
                    println!(
                        "Nakshatra: {} (index {}, pada {})",
                        info.nakshatra.nakshatra.name(),
                        info.nakshatra.nakshatra_index,
                        info.nakshatra.pada
                    );
                    println!(
                        "  Start:  {}  End: {}",
                        info.nakshatra.start, info.nakshatra.end
                    );
                    if let Some(m) = info.masa {
                        let adhika_str = if m.adhika { " (Adhika)" } else { "" };
                        println!("Masa:     {}{}", m.masa.name(), adhika_str);
                        println!("  Start:  {}  End: {}", m.start, m.end);
                    }
                    if let Some(a) = info.ayana {
                        println!("Ayana:    {}", a.ayana.name());
                        println!("  Start:  {}  End: {}", a.start, a.end);
                    }
                    if let Some(v) = info.varsha {
                        println!(
                            "Varsha:   {} (order {} of 60)",
                            v.samvatsara.name(),
                            v.order
                        );
                        println!("  Start:  {}  End: {}", v.start, v.end);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Ashtakavarga {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let config = dhruv_search::sankranti_types::SankrantiConfig::new(system, nutation);

            let result =
                dhruv_search::ashtakavarga_for_date(&engine, &eop_kernel, &utc, &location, &config)
                    .unwrap_or_else(|e| {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    });

            let graha_names = [
                "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn",
            ];
            let rashi_names = [
                "Mes", "Vrs", "Mit", "Kar", "Sim", "Kan", "Tul", "Vri", "Dha", "Mak", "Kum", "Mee",
            ];

            println!("Ashtakavarga for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);

            // BAV tables
            println!("Bhinna Ashtakavarga (BAV):\n");
            print!("{:>10}", "");
            for name in &rashi_names {
                print!("{:>5}", name);
            }
            println!("  Total");
            println!("{}", "-".repeat(10 + 5 * 12 + 7));

            for (i, bav) in result.bavs.iter().enumerate() {
                print!("{:>10}", graha_names[i]);
                for &p in &bav.points {
                    print!("{:>5}", p);
                }
                let total: u8 = bav.points.iter().sum();
                println!("{:>7}", total);
            }

            // SAV
            println!("\nSarva Ashtakavarga (SAV):\n");
            print!("{:>10}", "");
            for name in &rashi_names {
                print!("{:>5}", name);
            }
            println!("  Total");
            println!("{}", "-".repeat(10 + 5 * 12 + 7));

            print!("{:>10}", "SAV");
            for &p in &result.sav.total_points {
                print!("{:>5}", p);
            }
            let sav_total: u16 = result.sav.total_points.iter().map(|&p| p as u16).sum();
            println!("{:>7}", sav_total);

            print!("{:>10}", "Trikona");
            for &p in &result.sav.after_trikona {
                print!("{:>5}", p);
            }
            let tri_total: u16 = result.sav.after_trikona.iter().map(|&p| p as u16).sum();
            println!("{:>7}", tri_total);

            print!("{:>10}", "Ekadhi");
            for &p in &result.sav.after_ekadhipatya {
                print!("{:>5}", p);
            }
            let ek_total: u16 = result.sav.after_ekadhipatya.iter().map(|&p| p as u16).sum();
            println!("{:>7}", ek_total);
        }

        Commands::Upagrahas {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            let config = dhruv_search::sankranti_types::SankrantiConfig::new(system, nutation);

            let result = dhruv_search::all_upagrahas_for_date(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &rs_config,
                &config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!("Upagrahas for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);
            println!("Time-based:");
            for (name, lon) in [
                ("Gulika", result.gulika),
                ("Maandi", result.maandi),
                ("Kaala", result.kaala),
                ("Mrityu", result.mrityu),
                ("Artha Prahara", result.artha_prahara),
                ("Yama Ghantaka", result.yama_ghantaka),
            ] {
                let rashi_info = dhruv_vedic_base::rashi_from_longitude(lon);
                println!(
                    "  {:16} {:>8.4}° ({} {}°{:02}'{:04.1}\")",
                    name,
                    lon,
                    rashi_info.rashi.name(),
                    rashi_info.dms.degrees,
                    rashi_info.dms.minutes,
                    rashi_info.dms.seconds,
                );
            }
            println!("\nSun-based:");
            for (name, lon) in [
                ("Dhooma", result.dhooma),
                ("Vyatipata", result.vyatipata),
                ("Parivesha", result.parivesha),
                ("Indra Chapa", result.indra_chapa),
                ("Upaketu", result.upaketu),
            ] {
                let rashi_info = dhruv_vedic_base::rashi_from_longitude(lon);
                println!(
                    "  {:16} {:>8.4}° ({} {}°{:02}'{:04.1}\")",
                    name,
                    lon,
                    rashi_info.rashi.name(),
                    rashi_info.dms.degrees,
                    rashi_info.dms.minutes,
                    rashi_info.dms.seconds,
                );
            }
        }
        Commands::GrahaPositions {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            nakshatra,
            lagna,
            outer,
            bhava,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let bhava_config = BhavaConfig::default();
            let aya_config = SankrantiConfig::new(system, nutation);
            let gp_config = dhruv_search::GrahaPositionsConfig {
                include_nakshatra: nakshatra,
                include_lagna: lagna,
                include_outer_planets: outer,
                include_bhava: bhava,
            };

            let result = dhruv_search::graha_positions(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &bhava_config,
                &aya_config,
                &gp_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!(
                "Graha Positions for {} at {:.4}°N, {:.4}°E\n",
                date, lat, lon
            );

            // Header
            let graha_names = [
                "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu",
            ];
            print!("{:<10} {:>10}  {:<10}", "Graha", "Longitude", "Rashi");
            if nakshatra {
                print!("  {:<18} {:>4}", "Nakshatra", "Pada");
            }
            if bhava {
                print!("  {:>5}", "Bhava");
            }
            println!();
            let width = 32 + if nakshatra { 24 } else { 0 } + if bhava { 7 } else { 0 };
            println!("{}", "-".repeat(width));

            let print_entry =
                |name: &str, entry: &dhruv_search::GrahaEntry, force_bhava: Option<u8>| {
                    print!(
                        "{:<10} {:>9.4}°  {:<10}",
                        name,
                        entry.sidereal_longitude,
                        entry.rashi.name(),
                    );
                    if nakshatra {
                        print!(
                            "  {:<18} {:>4}",
                            entry.nakshatra.name(),
                            if entry.pada > 0 {
                                entry.pada.to_string()
                            } else {
                                "-".into()
                            },
                        );
                    }
                    if bhava {
                        let bh = force_bhava.unwrap_or(entry.bhava_number);
                        print!("  {:>5}", if bh > 0 { bh.to_string() } else { "-".into() },);
                    }
                    println!();
                };

            for (i, entry) in result.grahas.iter().enumerate() {
                print_entry(graha_names[i], entry, None);
            }

            if lagna {
                print_entry("Lagna", &result.lagna, Some(1));
            }

            if outer {
                let planet_names = ["Uranus", "Neptune", "Pluto"];
                for (i, entry) in result.outer_planets.iter().enumerate() {
                    print_entry(planet_names[i], entry, None);
                }
            }
        }
        Commands::CoreBindus {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            nakshatra,
            bhava,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let bhava_config = BhavaConfig::default();
            let rs_config = RiseSetConfig::default();
            let aya_config = SankrantiConfig::new(system, nutation);
            let bindus_config = dhruv_search::BindusConfig {
                include_nakshatra: nakshatra,
                include_bhava: bhava,
            };

            let result = dhruv_search::core_bindus(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &bhava_config,
                &rs_config,
                &aya_config,
                &bindus_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!("Core Bindus for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);

            // Header
            print!("{:<16} {:>10}  {:<10}", "Name", "Longitude", "Rashi");
            if nakshatra {
                print!("  {:<18} {:>4}", "Nakshatra", "Pada");
            }
            if bhava {
                print!("  {:>5}", "Bhava");
            }
            println!();
            let width = 38 + if nakshatra { 24 } else { 0 } + if bhava { 7 } else { 0 };
            println!("{}", "-".repeat(width));

            let print_entry = |name: &str, entry: &dhruv_search::GrahaEntry| {
                print!(
                    "{:<16} {:>9.4}°  {:<10}",
                    name,
                    entry.sidereal_longitude,
                    entry.rashi.name(),
                );
                if nakshatra {
                    print!(
                        "  {:<18} {:>4}",
                        entry.nakshatra.name(),
                        if entry.pada > 0 {
                            entry.pada.to_string()
                        } else {
                            "-".into()
                        },
                    );
                }
                if bhava {
                    print!(
                        "  {:>5}",
                        if entry.bhava_number > 0 {
                            entry.bhava_number.to_string()
                        } else {
                            "-".into()
                        },
                    );
                }
                println!();
            };

            println!("\nArudha Padas:");
            let pada_names = [
                "A1 (Lagna)",
                "A2 (Dhana)",
                "A3 (Sahaja)",
                "A4 (Sukha)",
                "A5 (Putra)",
                "A6 (Ari)",
                "A7 (Dara)",
                "A8 (Mrityu)",
                "A9 (Dharma)",
                "A10 (Karma)",
                "A11 (Labha)",
                "A12 (UL)",
            ];
            for (i, entry) in result.arudha_padas.iter().enumerate() {
                print_entry(pada_names[i], entry);
            }

            println!("\nSensitive Points:");
            print_entry("Bhrigu Bindu", &result.bhrigu_bindu);
            print_entry("Pranapada", &result.pranapada_lagna);
            print_entry("Gulika", &result.gulika);
            print_entry("Maandi", &result.maandi);
            print_entry("Hora Lagna", &result.hora_lagna);
            print_entry("Ghati Lagna", &result.ghati_lagna);
            print_entry("Sree Lagna", &result.sree_lagna);
        }
        Commands::Drishti {
            date,
            lat,
            lon,
            alt,
            ayanamsha,
            nutation,
            bhava,
            lagna,
            bindus,
            bsp,
            lsk,
            eop,
        } => {
            let system = require_aya_system(ayanamsha);
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let bhava_config = BhavaConfig::default();
            let rs_config = RiseSetConfig::default();
            let aya_config = SankrantiConfig::new(system, nutation);
            let drishti_config = dhruv_search::DrishtiConfig {
                include_bhava: bhava,
                include_lagna: lagna,
                include_bindus: bindus,
            };

            let result = dhruv_search::drishti_for_date(
                &engine,
                &eop_kernel,
                &utc,
                &location,
                &bhava_config,
                &rs_config,
                &aya_config,
                &drishti_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            let graha_names = [
                "Sun", "Moon", "Mars", "Merc", "Jup", "Ven", "Sat", "Rahu", "Ketu",
            ];

            println!("Graha Drishti for {} at {:.4}°N, {:.4}°E\n", date, lat, lon);

            // 9x9 graha-to-graha matrix
            println!("Graha-to-Graha (total virupa):");
            print!("{:<8}", "From\\To");
            for name in &graha_names {
                print!("{:>8}", name);
            }
            println!();
            println!("{}", "-".repeat(8 + 8 * 9));
            for i in 0..9 {
                print!("{:<8}", graha_names[i]);
                for j in 0..9 {
                    let v = result.graha_to_graha.entries[i][j].total_virupa;
                    if i == j {
                        print!("{:>8}", "-");
                    } else {
                        print!("{:>8.1}", v);
                    }
                }
                println!();
            }

            if lagna {
                println!("\nGraha-to-Lagna:");
                println!(
                    "{:<8} {:>8} {:>8} {:>8} {:>8}",
                    "Graha", "Dist", "Base", "Special", "Total"
                );
                println!("{}", "-".repeat(44));
                for i in 0..9 {
                    let e = &result.graha_to_lagna[i];
                    println!(
                        "{:<8} {:>7.1}° {:>8.1} {:>8.1} {:>8.1}",
                        graha_names[i],
                        e.angular_distance,
                        e.base_virupa,
                        e.special_virupa,
                        e.total_virupa
                    );
                }
            }

            if bhava {
                println!("\nGraha-to-Bhava Cusps (total virupa):");
                print!("{:<8}", "Graha");
                for b in 1..=12 {
                    print!("{:>6}", format!("B{b}"));
                }
                println!();
                println!("{}", "-".repeat(8 + 6 * 12));
                for i in 0..9 {
                    print!("{:<8}", graha_names[i]);
                    for j in 0..12 {
                        print!("{:>6.1}", result.graha_to_bhava[i][j].total_virupa);
                    }
                    println!();
                }
            }

            if bindus {
                let bindu_names = [
                    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11", "A12",
                    "BhrBin", "Prana", "Gulik", "Maand", "HoraL", "GhatiL", "SreeL",
                ];
                println!("\nGraha-to-Core Bindus (total virupa):");
                print!("{:<8}", "Graha");
                for name in &bindu_names {
                    print!("{:>7}", name);
                }
                println!();
                println!("{}", "-".repeat(8 + 7 * 19));
                for i in 0..9 {
                    print!("{:<8}", graha_names[i]);
                    for j in 0..19 {
                        print!("{:>7.1}", result.graha_to_bindus[i][j].total_virupa);
                    }
                    println!();
                }
            }
        }

        Commands::PrevPurnima { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::prev_purnima(&engine, &utc) {
                Ok(Some(ev)) => {
                    println!("Previous Purnima: {}", ev.utc);
                    println!(
                        "  Moon lon: {:.4} deg  Sun lon: {:.4} deg",
                        ev.moon_longitude_deg, ev.sun_longitude_deg
                    );
                }
                Ok(None) => println!("No Purnima found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevAmavasya { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::prev_amavasya(&engine, &utc) {
                Ok(Some(ev)) => {
                    println!("Previous Amavasya: {}", ev.utc);
                    println!(
                        "  Moon lon: {:.4} deg  Sun lon: {:.4} deg",
                        ev.moon_longitude_deg, ev.sun_longitude_deg
                    );
                }
                Ok(None) => println!("No Amavasya found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevSankranti {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::prev_sankranti(&engine, &utc, &config) {
                Ok(Some(ev)) => {
                    println!(
                        "Previous Sankranti: {} ({})",
                        ev.rashi.name(),
                        ev.rashi.western_name()
                    );
                    println!("  Time: {}", ev.utc);
                    println!(
                        "  Sidereal lon: {:.4} deg  Tropical lon: {:.4} deg",
                        ev.sun_sidereal_longitude_deg, ev.sun_tropical_longitude_deg
                    );
                }
                Ok(None) => println!("No Sankranti found in search range"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchPurnimas {
            start,
            end,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::search_purnimas(&engine, &s, &e) {
                Ok(events) => {
                    println!("Found {} Purnimas:", events.len());
                    for ev in &events {
                        println!(
                            "  {}  Moon: {:.4}°  Sun: {:.4}°",
                            ev.utc, ev.moon_longitude_deg, ev.sun_longitude_deg
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchAmavasyas {
            start,
            end,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            match dhruv_search::search_amavasyas(&engine, &s, &e) {
                Ok(events) => {
                    println!("Found {} Amavasyas:", events.len());
                    for ev in &events {
                        println!(
                            "  {}  Moon: {:.4}°  Sun: {:.4}°",
                            ev.utc, ev.moon_longitude_deg, ev.sun_longitude_deg
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchSankrantis {
            start,
            end,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::search_sankrantis(&engine, &s, &e, &config) {
                Ok(events) => {
                    println!("Found {} Sankrantis:", events.len());
                    for ev in &events {
                        println!(
                            "  {} ({}) at {}  sid: {:.4}°  trop: {:.4}°",
                            ev.rashi.name(),
                            ev.rashi.western_name(),
                            ev.utc,
                            ev.sun_sidereal_longitude_deg,
                            ev.sun_tropical_longitude_deg
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextSpecificSankranti {
            date,
            rashi,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let target = rashi_from_index(rashi);
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::next_specific_sankranti(&engine, &utc, target, &config) {
                Ok(Some(ev)) => {
                    println!("Next {} Sankranti: {}", ev.rashi.name(), ev.utc);
                    println!(
                        "  Sidereal lon: {:.4}°  Tropical lon: {:.4}°",
                        ev.sun_sidereal_longitude_deg, ev.sun_tropical_longitude_deg
                    );
                }
                Ok(None) => println!("No {} Sankranti found", target.name()),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevSpecificSankranti {
            date,
            rashi,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let target = rashi_from_index(rashi);
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let config = SankrantiConfig::new(system, nutation);
            match dhruv_search::prev_specific_sankranti(&engine, &utc, target, &config) {
                Ok(Some(ev)) => {
                    println!("Previous {} Sankranti: {}", ev.rashi.name(), ev.utc);
                    println!(
                        "  Sidereal lon: {:.4}°  Tropical lon: {:.4}°",
                        ev.sun_sidereal_longitude_deg, ev.sun_tropical_longitude_deg
                    );
                }
                Ok(None) => println!("No {} Sankranti found", target.name()),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::AyanamshaCompute {
            date,
            ayanamsha,
            nutation,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let system = require_aya_system(ayanamsha);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let t = jd_tdb_to_centuries(jd_tdb);
            let aya = ayanamsha_deg(system, t, nutation);
            println!(
                "Ayanamsha ({:?}): {:.6}°{}",
                system,
                aya,
                if nutation { " (with nutation)" } else { "" }
            );
        }

        Commands::NutationCompute { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let t = jd_tdb_to_centuries(jd_tdb);
            let (dpsi, deps) = nutation_iau2000b(t);
            println!("Nutation at {}:", date);
            println!("  dpsi (longitude): {:.6} arcsec", dpsi);
            println!("  deps (obliquity): {:.6} arcsec", deps);
        }

        Commands::Sunrise {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let rs_config = RiseSetConfig::default();
            let jd_utc = utc_to_jd_utc(&utc);
            let jd_noon =
                dhruv_vedic_base::approximate_local_noon_jd(jd_utc.floor(), location.longitude_deg);

            let events = dhruv_vedic_base::compute_all_events(
                &engine,
                engine.lsk(),
                &eop_kernel,
                &location,
                jd_noon,
                &rs_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!(
                "Rise/Set events for {} at {:.4}°N, {:.4}°E:\n",
                date, lat, lon
            );
            for result in &events {
                match result {
                    RiseSetResult::Event { jd_tdb, event } => {
                        println!("  {:20} JD TDB {:.6}", format!("{event:?}"), jd_tdb);
                    }
                    RiseSetResult::NeverRises => println!("  Sun never rises (polar night)"),
                    RiseSetResult::NeverSets => println!("  Sun never sets (midnight sun)"),
                }
            }
        }

        Commands::Bhavas {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let bhava_config = BhavaConfig::default();
            let jd_utc = utc_to_jd_utc(&utc);

            let result = dhruv_vedic_base::compute_bhavas(
                &engine,
                engine.lsk(),
                &eop_kernel,
                &location,
                jd_utc,
                &bhava_config,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!(
                "Bhavas for {} at {:.4}°N, {:.4}°E\n",
                date, lat, lon
            );
            println!("  Lagna: {:.4}°  MC: {:.4}°\n", result.lagna_deg, result.mc_deg);
            println!("{:>6} {:>10} {:>10} {:>10}", "Bhava", "Cusp", "Start", "End");
            println!("{}", "-".repeat(40));
            for b in &result.bhavas {
                println!(
                    "{:>6} {:>9.4}° {:>9.4}° {:>9.4}°",
                    b.number, b.cusp_deg, b.start_deg, b.end_deg
                );
            }
        }

        Commands::LagnaCompute {
            date,
            lat,
            lon,
            alt,
            bsp,
            lsk,
            eop,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let eop_kernel = load_eop(&eop);
            let location = GeoLocation::new(lat, lon, alt);
            let jd_utc = utc_to_jd_utc(&utc);

            let lagna = dhruv_vedic_base::lagna_longitude_rad(
                engine.lsk(),
                &eop_kernel,
                &location,
                jd_utc,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            let mc = dhruv_vedic_base::mc_longitude_rad(
                engine.lsk(),
                &eop_kernel,
                &location,
                jd_utc,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });
            let ramc = dhruv_vedic_base::ramc_rad(
                engine.lsk(),
                &eop_kernel,
                &location,
                jd_utc,
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {e}");
                std::process::exit(1);
            });

            println!("Lagna (tropical): {:.6}°", lagna.to_degrees().rem_euclid(360.0));
            println!("MC (tropical):    {:.6}°", mc.to_degrees().rem_euclid(360.0));
            println!("RAMC:             {:.6}°", ramc.to_degrees().rem_euclid(360.0));
        }

        Commands::LunarNode {
            date,
            node,
            mode,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let t = jd_tdb_to_centuries(jd_tdb);
            let lunar_node = parse_lunar_node(&node);
            let node_mode = parse_node_mode(&mode);
            let lon = dhruv_vedic_base::lunar_node_deg(lunar_node, t, node_mode);
            println!("{:?} ({:?}): {:.6}°", lunar_node, node_mode, lon);
        }

        Commands::NextConjunction {
            date,
            body1,
            body2,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b1 = require_body(body1);
            let b2 = require_body(body2);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = ConjunctionConfig::conjunction(1.0);
            match dhruv_search::next_conjunction(&engine, b1, b2, jd_tdb, &config) {
                Ok(Some(ev)) => print_conjunction_event("Next conjunction", &ev),
                Ok(None) => println!("No conjunction found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevConjunction {
            date,
            body1,
            body2,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b1 = require_body(body1);
            let b2 = require_body(body2);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = ConjunctionConfig::conjunction(1.0);
            match dhruv_search::prev_conjunction(&engine, b1, b2, jd_tdb, &config) {
                Ok(Some(ev)) => print_conjunction_event("Previous conjunction", &ev),
                Ok(None) => println!("No conjunction found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchConjunctions {
            start,
            end,
            body1,
            body2,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b1 = require_body(body1);
            let b2 = require_body(body2);
            let engine = load_engine(&bsp, &lsk);
            let jd_start = s.to_jd_tdb(engine.lsk());
            let jd_end = e.to_jd_tdb(engine.lsk());
            let config = ConjunctionConfig::conjunction(1.0);
            match dhruv_search::search_conjunctions(&engine, b1, b2, jd_start, jd_end, &config) {
                Ok(events) => {
                    println!("Found {} conjunctions:", events.len());
                    for ev in &events {
                        print_conjunction_event("  Conjunction", ev);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextChandraGrahan { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::next_chandra_grahan(&engine, jd_tdb, &config) {
                Ok(Some(ev)) => print_chandra_grahan("Next Chandra Grahan", &ev),
                Ok(None) => println!("No lunar eclipse found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevChandraGrahan { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::prev_chandra_grahan(&engine, jd_tdb, &config) {
                Ok(Some(ev)) => print_chandra_grahan("Previous Chandra Grahan", &ev),
                Ok(None) => println!("No lunar eclipse found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchChandraGrahan {
            start,
            end,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_start = s.to_jd_tdb(engine.lsk());
            let jd_end = e.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::search_chandra_grahan(&engine, jd_start, jd_end, &config) {
                Ok(events) => {
                    println!("Found {} lunar eclipses:", events.len());
                    for ev in &events {
                        print_chandra_grahan("  Chandra Grahan", ev);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextSuryaGrahan { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::next_surya_grahan(&engine, jd_tdb, &config) {
                Ok(Some(ev)) => print_surya_grahan("Next Surya Grahan", &ev),
                Ok(None) => println!("No solar eclipse found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevSuryaGrahan { date, bsp, lsk } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::prev_surya_grahan(&engine, jd_tdb, &config) {
                Ok(Some(ev)) => print_surya_grahan("Previous Surya Grahan", &ev),
                Ok(None) => println!("No solar eclipse found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchSuryaGrahan {
            start,
            end,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let engine = load_engine(&bsp, &lsk);
            let jd_start = s.to_jd_tdb(engine.lsk());
            let jd_end = e.to_jd_tdb(engine.lsk());
            let config = GrahanConfig {
                include_penumbral: true,
                include_peak_details: true,
            };
            match dhruv_search::search_surya_grahan(&engine, jd_start, jd_end, &config) {
                Ok(events) => {
                    println!("Found {} solar eclipses:", events.len());
                    for ev in &events {
                        print_surya_grahan("  Surya Grahan", ev);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextStationary {
            date,
            body,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::next_stationary(&engine, b, jd_tdb, &config) {
                Ok(Some(ev)) => print_stationary_event("Next stationary", &ev),
                Ok(None) => println!("No stationary point found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevStationary {
            date,
            body,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::prev_stationary(&engine, b, jd_tdb, &config) {
                Ok(Some(ev)) => print_stationary_event("Previous stationary", &ev),
                Ok(None) => println!("No stationary point found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchStationary {
            start,
            end,
            body,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_start = s.to_jd_tdb(engine.lsk());
            let jd_end = e.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::search_stationary(&engine, b, jd_start, jd_end, &config) {
                Ok(events) => {
                    println!("Found {} stationary points:", events.len());
                    for ev in &events {
                        print_stationary_event("  Station", ev);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::NextMaxSpeed {
            date,
            body,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::next_max_speed(&engine, b, jd_tdb, &config) {
                Ok(Some(ev)) => print_max_speed_event("Next max-speed", &ev),
                Ok(None) => println!("No max-speed event found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::PrevMaxSpeed {
            date,
            body,
            bsp,
            lsk,
        } => {
            let utc = parse_utc(&date).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_tdb = utc.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::prev_max_speed(&engine, b, jd_tdb, &config) {
                Ok(Some(ev)) => print_max_speed_event("Previous max-speed", &ev),
                Ok(None) => println!("No max-speed event found"),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::SearchMaxSpeed {
            start,
            end,
            body,
            bsp,
            lsk,
        } => {
            let s = parse_utc(&start).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let e = parse_utc(&end).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            });
            let b = require_body(body);
            let engine = load_engine(&bsp, &lsk);
            let jd_start = s.to_jd_tdb(engine.lsk());
            let jd_end = e.to_jd_tdb(engine.lsk());
            let config = StationaryConfig::inner_planet();
            match dhruv_search::search_max_speed(&engine, b, jd_start, jd_end, &config) {
                Ok(events) => {
                    println!("Found {} max-speed events:", events.len());
                    for ev in &events {
                        print_max_speed_event("  Max-speed", ev);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_conjunction_event(label: &str, ev: &ConjunctionEvent) {
    println!(
        "{}: JD TDB {:.6}  sep: {:.4}°",
        label, ev.jd_tdb, ev.actual_separation_deg
    );
    println!(
        "  Body1 lon: {:.4}°  Body2 lon: {:.4}°",
        ev.body1_longitude_deg, ev.body2_longitude_deg
    );
}

fn print_chandra_grahan(label: &str, ev: &dhruv_search::grahan_types::ChandraGrahan) {
    println!(
        "{}: {:?}  mag: {:.4}  penumbral mag: {:.4}",
        label, ev.grahan_type, ev.magnitude, ev.penumbral_magnitude
    );
    println!("  Greatest: JD TDB {:.6}", ev.greatest_grahan_jd);
    println!("  P1: JD TDB {:.6}", ev.p1_jd);
    if let Some(u1) = ev.u1_jd {
        println!("  U1: JD TDB {:.6}", u1);
    }
    if let Some(u2) = ev.u2_jd {
        println!("  U2: JD TDB {:.6}", u2);
    }
}

fn print_surya_grahan(label: &str, ev: &dhruv_search::grahan_types::SuryaGrahan) {
    println!(
        "{}: {:?}  mag: {:.4}",
        label, ev.grahan_type, ev.magnitude
    );
    println!("  Greatest: JD TDB {:.6}", ev.greatest_grahan_jd);
    if let Some(c1) = ev.c1_jd {
        println!("  C1: JD TDB {:.6}", c1);
    }
    if let Some(c2) = ev.c2_jd {
        println!("  C2: JD TDB {:.6}", c2);
    }
    if let Some(c3) = ev.c3_jd {
        println!("  C3: JD TDB {:.6}", c3);
    }
    if let Some(c4) = ev.c4_jd {
        println!("  C4: JD TDB {:.6}", c4);
    }
}

fn print_stationary_event(label: &str, ev: &dhruv_search::stationary_types::StationaryEvent) {
    println!(
        "{}: {:?} {:?} at JD TDB {:.6}",
        label, ev.body, ev.station_type, ev.jd_tdb
    );
    println!(
        "  Longitude: {:.4}°  Latitude: {:.4}°",
        ev.longitude_deg, ev.latitude_deg
    );
}

fn print_max_speed_event(label: &str, ev: &dhruv_search::stationary_types::MaxSpeedEvent) {
    println!(
        "{}: {:?} {:?} at JD TDB {:.6}",
        label, ev.body, ev.speed_type, ev.jd_tdb
    );
    println!(
        "  Longitude: {:.4}°  Speed: {:.6} deg/day",
        ev.longitude_deg, ev.speed_deg_per_day
    );
}
