//! Samvatsara office-bearers (varsha adhikaris) of a lunar year.
//!
//! Each office is held by the lord of the Vedic weekday (vaar) on which one
//! fixed annual event of the lunar year falls:
//!
//! | office    | event                                   |
//! |-----------|-----------------------------------------|
//! | Raja      | Chaitra Shukla Pratipada (year start)    |
//! | Mantri    | Mesha sankranti                         |
//! | Sasyesha  | Karka sankranti                         |
//! | Dhanyesha | Dhanu sankranti                         |
//! | Meghesha  | Sun entering Ardra (sidereal 66°40')    |
//! | Rasesha   | Tula sankranti                          |
//! | Nirasesha | Makara sankranti                        |
//! | Phalesha  | Meena sankranti                         |
//! | Durgesha  | Simha sankranti                         |
//!
//! Conventions:
//! - The lunar year is the [`VarshaInfo`] containing the request moment
//!   (see `varsha_for_date`). Every event is the first occurrence at or after
//!   the year start; for Meena sankranti that may fall just after the year's
//!   own end when the next Chaitra pratipada precedes it.
//! - The vaar is the sunrise-to-sunrise Vedic day at the request location.
//!   Instant-defined events (sankrantis, the Ardra entry) take the Vedic day
//!   containing the instant.
//! - The tithi-defined Raja day follows the udaya rule: the Vedic day whose
//!   sunrise falls inside the pratipada tithi. When no sunrise does (kshaya
//!   pratipada) the Vedic day containing the tithi start is used.
//!
//! Clean-room implementation of the classical rule set as supplied in the
//! project requirements; no external code consulted.

use dhruv_core::{Body, Engine};
use dhruv_time::{EopKernel, UtcTime};
use dhruv_vedic_base::riseset_types::{GeoLocation, RiseSetConfig};
use dhruv_vedic_base::{Graha, Rashi, Vaar, vaar_from_jd, vaar_lord};

use crate::error::SearchError;
use crate::fixed_longitude::next_fixed_longitude;
use crate::panchang::{tithi_for_date, varsha_for_date_with_eop, vedic_day_sunrises};
use crate::panchang_types::VarshaInfo;
use crate::sankranti::next_specific_sankranti;
use crate::sankranti_types::SankrantiConfig;
use crate::transit_body::TransitBody;

/// Sidereal longitude at which the Sun enters Ardra (6th nakshatra).
const ARDRA_START_DEG: f64 = 5.0 * 360.0 / 27.0;

/// Offset (days, ~2 minutes) past the year-start instant used to classify
/// the pratipada tithi.
const PRATIPADA_PROBE_DAYS: f64 = 2.0 / 1440.0;

/// The nine samvatsara offices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamvatsaraOffice {
    Raja,
    Mantri,
    Sasyesha,
    Dhanyesha,
    Meghesha,
    Rasesha,
    Nirasesha,
    Phalesha,
    Durgesha,
}

/// All offices in output order.
pub const ALL_SAMVATSARA_OFFICES: [SamvatsaraOffice; 9] = [
    SamvatsaraOffice::Raja,
    SamvatsaraOffice::Mantri,
    SamvatsaraOffice::Sasyesha,
    SamvatsaraOffice::Dhanyesha,
    SamvatsaraOffice::Meghesha,
    SamvatsaraOffice::Rasesha,
    SamvatsaraOffice::Nirasesha,
    SamvatsaraOffice::Phalesha,
    SamvatsaraOffice::Durgesha,
];

impl SamvatsaraOffice {
    /// Sanskrit name of the office.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Raja => "Raja",
            Self::Mantri => "Mantri",
            Self::Sasyesha => "Sasyesha",
            Self::Dhanyesha => "Dhanyesha",
            Self::Meghesha => "Meghesha",
            Self::Rasesha => "Rasesha",
            Self::Nirasesha => "Nirasesha",
            Self::Phalesha => "Phalesha",
            Self::Durgesha => "Durgesha",
        }
    }

    /// 0-based index in [`ALL_SAMVATSARA_OFFICES`].
    pub const fn index(self) -> u8 {
        match self {
            Self::Raja => 0,
            Self::Mantri => 1,
            Self::Sasyesha => 2,
            Self::Dhanyesha => 3,
            Self::Meghesha => 4,
            Self::Rasesha => 5,
            Self::Nirasesha => 6,
            Self::Phalesha => 7,
            Self::Durgesha => 8,
        }
    }

    /// Short description of the defining event.
    pub const fn event_name(self) -> &'static str {
        match self {
            Self::Raja => "Chaitra Shukla Pratipada",
            Self::Mantri => "Mesha Sankranti",
            Self::Sasyesha => "Karka Sankranti",
            Self::Dhanyesha => "Dhanu Sankranti",
            Self::Meghesha => "Surya Ardra Pravesha",
            Self::Rasesha => "Tula Sankranti",
            Self::Nirasesha => "Makara Sankranti",
            Self::Phalesha => "Meena Sankranti",
            Self::Durgesha => "Simha Sankranti",
        }
    }

    /// Rashi whose sankranti defines the office, if it is sankranti-defined.
    const fn sankranti_rashi(self) -> Option<Rashi> {
        match self {
            Self::Raja | Self::Meghesha => None,
            Self::Mantri => Some(Rashi::Mesha),
            Self::Sasyesha => Some(Rashi::Karka),
            Self::Dhanyesha => Some(Rashi::Dhanu),
            Self::Rasesha => Some(Rashi::Tula),
            Self::Nirasesha => Some(Rashi::Makara),
            Self::Phalesha => Some(Rashi::Meena),
            Self::Durgesha => Some(Rashi::Simha),
        }
    }
}

/// One office-bearer of the lunar year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamvatsaraOfficer {
    /// The office.
    pub office: SamvatsaraOffice,
    /// Instant of the defining event (UTC).
    pub event_utc: UtcTime,
    /// Sunrise (UTC) starting the Vedic day whose lord holds the office.
    pub vedic_day_start: UtcTime,
    /// Vaar of that Vedic day.
    pub vaar: Vaar,
    /// Lord of the vaar: the office-bearer.
    pub lord: Graha,
}

/// All office-bearers of one lunar year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamvatsaraOfficers {
    /// The lunar year the offices belong to.
    pub varsha: VarshaInfo,
    /// Office-bearers in [`ALL_SAMVATSARA_OFFICES`] order.
    pub officers: [SamvatsaraOfficer; 9],
}

impl SamvatsaraOfficers {
    /// Office-bearer for `office`.
    pub fn officer(&self, office: SamvatsaraOffice) -> &SamvatsaraOfficer {
        &self.officers[office.index() as usize]
    }
}

/// Compute the samvatsara office-bearers of the lunar year containing `utc`,
/// with Vedic days evaluated at `location`.
pub fn samvatsara_officers(
    engine: &Engine,
    eop: &EopKernel,
    utc: &UtcTime,
    location: &GeoLocation,
    riseset_config: &RiseSetConfig,
    aya_config: &SankrantiConfig,
) -> Result<SamvatsaraOfficers, SearchError> {
    let varsha = varsha_for_date_with_eop(engine, Some(eop), utc, aya_config)?;
    let year_start_jd =
        crate::search_util::utc_to_jd_tdb_with_eop(engine, Some(eop), &varsha.start);

    let officer_for_instant =
        |office: SamvatsaraOffice, event_utc: UtcTime| -> Result<SamvatsaraOfficer, SearchError> {
            let (sunrise_jd, _next) =
                vedic_day_sunrises(engine, eop, &event_utc, location, riseset_config)?;
            Ok(officer_at_sunrise(office, event_utc, sunrise_jd, engine))
        };

    let mut officers = [SamvatsaraOfficer {
        office: SamvatsaraOffice::Raja,
        event_utc: varsha.start,
        vedic_day_start: varsha.start,
        vaar: Vaar::Ravivaar,
        lord: Graha::Surya,
    }; 9];

    for office in ALL_SAMVATSARA_OFFICES {
        let officer = match office {
            SamvatsaraOffice::Raja => raja_officer(
                engine,
                eop,
                &varsha,
                year_start_jd,
                location,
                riseset_config,
            )?,
            SamvatsaraOffice::Meghesha => {
                let event = next_fixed_longitude(
                    engine,
                    TransitBody::Body(Body::Sun),
                    year_start_jd,
                    ARDRA_START_DEG,
                    &[0.0],
                    aya_config,
                )?
                .ok_or(SearchError::NoConvergence(
                    "could not find the Sun's Ardra entry",
                ))?;
                officer_for_instant(office, event.utc)?
            }
            _ => {
                let rashi = office.sankranti_rashi().expect("sankranti-defined office");
                let event = next_specific_sankranti(engine, &varsha.start, rashi, aya_config)?
                    .ok_or(SearchError::NoConvergence(
                        "could not find sankranti for samvatsara office",
                    ))?;
                officer_for_instant(office, event.utc)?
            }
        };
        officers[office.index() as usize] = officer;
    }

    Ok(SamvatsaraOfficers { varsha, officers })
}

fn officer_at_sunrise(
    office: SamvatsaraOffice,
    event_utc: UtcTime,
    sunrise_jd: f64,
    engine: &Engine,
) -> SamvatsaraOfficer {
    let vaar = vaar_from_jd(sunrise_jd);
    SamvatsaraOfficer {
        office,
        event_utc,
        vedic_day_start: UtcTime::from_jd_tdb(sunrise_jd, engine.lsk()),
        vaar,
        lord: vaar_lord(vaar),
    }
}

/// Raja: lord of the Vedic day of Chaitra Shukla Pratipada under the udaya
/// rule (first sunrise inside the tithi; the day containing the tithi start
/// when the tithi is kshaya).
fn raja_officer(
    engine: &Engine,
    eop: &EopKernel,
    varsha: &VarshaInfo,
    year_start_jd: f64,
    location: &GeoLocation,
    riseset_config: &RiseSetConfig,
) -> Result<SamvatsaraOfficer, SearchError> {
    let probe = UtcTime::from_jd_tdb(year_start_jd + PRATIPADA_PROBE_DAYS, engine.lsk());
    let tithi = tithi_for_date(engine, &probe)?;
    let tithi_end_jd = crate::search_util::utc_to_jd_tdb_with_eop(engine, Some(eop), &tithi.end);
    let (sunrise_jd, next_sunrise_jd) =
        vedic_day_sunrises(engine, eop, &varsha.start, location, riseset_config)?;
    let day_sunrise_jd = if next_sunrise_jd < tithi_end_jd {
        next_sunrise_jd
    } else {
        sunrise_jd
    };
    Ok(officer_at_sunrise(
        SamvatsaraOffice::Raja,
        varsha.start,
        day_sunrise_jd,
        engine,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn office_indices_sequential() {
        for (i, office) in ALL_SAMVATSARA_OFFICES.iter().enumerate() {
            assert_eq!(office.index() as usize, i);
            assert!(!office.name().is_empty());
            assert!(!office.event_name().is_empty());
        }
    }

    #[test]
    fn sankranti_table_matches_spec() {
        assert_eq!(SamvatsaraOffice::Raja.sankranti_rashi(), None);
        assert_eq!(SamvatsaraOffice::Meghesha.sankranti_rashi(), None);
        assert_eq!(
            SamvatsaraOffice::Mantri.sankranti_rashi(),
            Some(Rashi::Mesha)
        );
        assert_eq!(
            SamvatsaraOffice::Sasyesha.sankranti_rashi(),
            Some(Rashi::Karka)
        );
        assert_eq!(
            SamvatsaraOffice::Dhanyesha.sankranti_rashi(),
            Some(Rashi::Dhanu)
        );
        assert_eq!(
            SamvatsaraOffice::Rasesha.sankranti_rashi(),
            Some(Rashi::Tula)
        );
        assert_eq!(
            SamvatsaraOffice::Nirasesha.sankranti_rashi(),
            Some(Rashi::Makara)
        );
        assert_eq!(
            SamvatsaraOffice::Phalesha.sankranti_rashi(),
            Some(Rashi::Meena)
        );
        assert_eq!(
            SamvatsaraOffice::Durgesha.sankranti_rashi(),
            Some(Rashi::Simha)
        );
    }

    #[test]
    fn ardra_start_is_66_40() {
        assert!((ARDRA_START_DEG - 66.666_666_67).abs() < 1e-6);
    }
}
