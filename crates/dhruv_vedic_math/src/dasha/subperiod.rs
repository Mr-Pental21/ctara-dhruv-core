//! Shared sub-period generation patterns used by dasha engines.
//!
//! Two core patterns:
//! - Proportional: child duration = (child_full_period / total_period) * parent_duration
//! - Equal: child duration = parent_duration / num_children

use super::types::{DashaEntity, DashaLevel, DashaPeriod};
use super::variation::SubPeriodMethod;

/// Snap the last child's end_jd to parent's end_jd to absorb floating-point drift.
pub fn snap_last_child_end(children: &mut [DashaPeriod], parent_end_jd: f64) {
    if let Some(last) = children.last_mut() {
        last.end_jd = parent_end_jd;
    }
}

/// Generate proportional children for a parent period.
///
/// `sequence`: ordered entities and their full-cycle periods (in days).
/// `total_period_days`: sum of all sequence periods.
/// `child_level`: the level to assign to generated children.
/// `parent_idx`: index of parent in its level array.
pub fn proportional_children(
    parent: &DashaPeriod,
    sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    child_level: DashaLevel,
    parent_idx: u32,
) -> Vec<DashaPeriod> {
    let parent_duration = parent.end_jd - parent.start_jd;
    let mut children = Vec::with_capacity(sequence.len());
    let mut cursor = parent.start_jd;

    for (order_0, &(entity, full_period)) in sequence.iter().enumerate() {
        let duration = (full_period / total_period_days) * parent_duration;
        let end = cursor + duration;
        children.push(DashaPeriod {
            entity,
            start_jd: cursor,
            end_jd: end,
            level: child_level,
            order: (order_0 as u16) + 1,
            parent_idx,
        });
        cursor = end;
    }

    snap_last_child_end(&mut children, parent.end_jd);
    children
}

/// Generate equal-duration children for a parent period.
///
/// `sequence`: ordered entities (periods are ignored; all get equal time).
/// `child_level`: the level to assign to generated children.
/// `parent_idx`: index of parent in its level array.
pub fn equal_children(
    parent: &DashaPeriod,
    sequence: &[DashaEntity],
    child_level: DashaLevel,
    parent_idx: u32,
) -> Vec<DashaPeriod> {
    let n = sequence.len();
    if n == 0 {
        return Vec::new();
    }
    let parent_duration = parent.end_jd - parent.start_jd;
    let child_duration = parent_duration / n as f64;
    let mut children = Vec::with_capacity(n);
    let mut cursor = parent.start_jd;

    for (order_0, &entity) in sequence.iter().enumerate() {
        let end = cursor + child_duration;
        children.push(DashaPeriod {
            entity,
            start_jd: cursor,
            end_jd: end,
            level: child_level,
            order: (order_0 as u16) + 1,
            parent_idx,
        });
        cursor = end;
    }

    snap_last_child_end(&mut children, parent.end_jd);
    children
}

/// Build a child sequence for a nakshatra-based system starting from a given
/// position in the cyclic graha list.
///
/// `graha_sequence`: the full ordered list of entities with their periods.
/// `start_from`: whether to start from the parent's entity or the next one.
/// `parent_entity`: the parent's entity.
pub fn build_cyclic_sequence(
    graha_sequence: &[(DashaEntity, f64)],
    parent_entity: DashaEntity,
    method: SubPeriodMethod,
) -> Vec<(DashaEntity, f64)> {
    let n = graha_sequence.len();
    // Find the parent's position in the sequence
    let parent_pos = graha_sequence
        .iter()
        .position(|(e, _)| *e == parent_entity)
        .unwrap_or(0);

    let start = match method {
        SubPeriodMethod::ProportionalFromParent | SubPeriodMethod::EqualFromSame => parent_pos,
        SubPeriodMethod::ProportionalFromNext | SubPeriodMethod::EqualFromNext => {
            (parent_pos + 1) % n
        }
    };

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let idx = (start + i) % n;
        result.push(graha_sequence[idx]);
    }
    result
}

/// Generate children for a parent using a specific sub-period method.
pub fn generate_children(
    parent: &DashaPeriod,
    graha_sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    child_level: DashaLevel,
    parent_idx: u32,
    method: SubPeriodMethod,
) -> Vec<DashaPeriod> {
    let seq = build_cyclic_sequence(graha_sequence, parent.entity, method);

    match method {
        SubPeriodMethod::ProportionalFromParent | SubPeriodMethod::ProportionalFromNext => {
            proportional_children(parent, &seq, total_period_days, child_level, parent_idx)
        }
        SubPeriodMethod::EqualFromNext | SubPeriodMethod::EqualFromSame => {
            let entities: Vec<DashaEntity> = seq.iter().map(|(e, _)| *e).collect();
            equal_children(parent, &entities, child_level, parent_idx)
        }
    }
}

// ── Birth-anchored sub-periods ───────────────────────────────────────
//
// The first period at every level is clipped at its start by the birth
// instant (the birth balance). Its children must be laid out over the FULL
// period, anchored at `end − full_duration`, not squeezed into the visible
// remainder. Children that end before birth are dropped and the one
// straddling birth is clipped, which in turn makes it a clipped parent for
// the next level. The "birth chain" is this clipped period at each level
// together with its unclipped length.

/// Tolerance (days) when matching period boundaries. ~86 µs.
pub const JD_MATCH_EPS: f64 = 1e-9;

/// A period on the birth chain and the length it would have had without
/// the birth clip.
#[derive(Debug, Clone, Copy)]
pub struct BirthChainLink {
    /// The clipped period as it appears in its level.
    pub period: DashaPeriod,
    /// Unclipped duration in days.
    pub full_duration_days: f64,
}

impl BirthChainLink {
    /// True when `parent` is this link's period (same level, entity, bounds).
    pub fn matches(&self, parent: &DashaPeriod) -> bool {
        self.period.level == parent.level
            && self.period.entity == parent.entity
            && (self.period.start_jd - parent.start_jd).abs() < JD_MATCH_EPS
            && (self.period.end_jd - parent.end_jd).abs() < JD_MATCH_EPS
    }
}

/// Generate children for a parent whose start was clipped by birth.
///
/// `full_duration_days` is the parent's unclipped length. Children are laid
/// out over the virtual parent `[end − full, end)`; those ending at or before
/// `parent.start_jd` are dropped and the one straddling it is clipped to start
/// there. `order` keeps the position in the full sequence so callers can see
/// which siblings elapsed before birth.
///
/// Returns the retained children and the link for the clipped first child
/// (None when nothing is retained).
pub fn generate_children_anchored(
    parent: &DashaPeriod,
    graha_sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    child_level: DashaLevel,
    parent_idx: u32,
    method: SubPeriodMethod,
    full_duration_days: f64,
) -> (Vec<DashaPeriod>, Option<BirthChainLink>) {
    let actual = parent.end_jd - parent.start_jd;
    if full_duration_days <= actual + JD_MATCH_EPS {
        // Not actually clipped: plain layout, first child is the next link.
        let children = generate_children(
            parent,
            graha_sequence,
            total_period_days,
            child_level,
            parent_idx,
            method,
        );
        let link = children.first().map(|c| BirthChainLink {
            period: *c,
            full_duration_days: c.end_jd - c.start_jd,
        });
        return (children, link);
    }

    let virtual_parent = DashaPeriod {
        start_jd: parent.end_jd - full_duration_days,
        ..*parent
    };
    let all = generate_children(
        &virtual_parent,
        graha_sequence,
        total_period_days,
        child_level,
        parent_idx,
        method,
    );

    let mut children = Vec::with_capacity(all.len());
    let mut link = None;
    for child in all {
        if child.end_jd <= parent.start_jd + JD_MATCH_EPS {
            continue;
        }
        if link.is_none() {
            let full = child.end_jd - child.start_jd;
            let clipped = DashaPeriod {
                start_jd: parent.start_jd,
                ..child
            };
            link = Some(BirthChainLink {
                period: clipped,
                full_duration_days: full,
            });
            children.push(clipped);
        } else {
            children.push(child);
        }
    }
    (children, link)
}

/// Children of `parent`, anchored when `anchor` is the parent's chain link.
///
/// Returns the children and the next chain link (the clipped first child)
/// when the parent was on the chain, else None.
pub fn generate_children_maybe_anchored(
    parent: &DashaPeriod,
    graha_sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    child_level: DashaLevel,
    parent_idx: u32,
    method: SubPeriodMethod,
    anchor: Option<&BirthChainLink>,
) -> (Vec<DashaPeriod>, Option<BirthChainLink>) {
    match anchor {
        Some(link) if link.matches(parent) => generate_children_anchored(
            parent,
            graha_sequence,
            total_period_days,
            child_level,
            parent_idx,
            method,
            link.full_duration_days,
        ),
        _ => (
            generate_children(
                parent,
                graha_sequence,
                total_period_days,
                child_level,
                parent_idx,
                method,
            ),
            None,
        ),
    }
}

/// Build the birth chain from the level-0 link down to `upto_level`
/// (inclusive). `method_for_parent_depth(d)` gives the sub-period method used
/// to split level `d` into level `d + 1`.
pub fn birth_chain(
    level0: BirthChainLink,
    graha_sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    upto_level: u8,
    method_for_parent_depth: &dyn Fn(u8) -> SubPeriodMethod,
) -> Vec<BirthChainLink> {
    let mut chain = vec![level0];
    for depth in 0..upto_level {
        let parent = chain[depth as usize];
        let child_level = match parent.period.level.child_level() {
            Some(l) => l,
            None => break,
        };
        let (_, next) = generate_children_anchored(
            &parent.period,
            graha_sequence,
            total_period_days,
            child_level,
            0,
            method_for_parent_depth(depth),
            parent.full_duration_days,
        );
        match next {
            Some(link) => chain.push(link),
            None => break,
        }
    }
    chain
}

/// Generate a complete child level, anchoring the parent that matches
/// `anchor`. Returns the children and the next chain link.
pub fn complete_level_anchored(
    parent_level: &[DashaPeriod],
    graha_sequence: &[(DashaEntity, f64)],
    total_period_days: f64,
    child_level: DashaLevel,
    method: SubPeriodMethod,
    anchor: Option<&BirthChainLink>,
) -> (Vec<DashaPeriod>, Option<BirthChainLink>) {
    let mut result = Vec::with_capacity(parent_level.len() * graha_sequence.len());
    let mut next = None;
    for (pidx, parent) in parent_level.iter().enumerate() {
        let (children, link) = generate_children_maybe_anchored(
            parent,
            graha_sequence,
            total_period_days,
            child_level,
            pidx as u32,
            method,
            anchor,
        );
        if link.is_some() {
            next = link;
        }
        result.extend(children);
    }
    (result, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graha::Graha;

    fn test_sequence() -> Vec<(DashaEntity, f64)> {
        vec![
            (DashaEntity::Graha(Graha::Ketu), 100.0),
            (DashaEntity::Graha(Graha::Shukra), 200.0),
            (DashaEntity::Graha(Graha::Surya), 100.0),
        ]
    }

    #[test]
    fn proportional_children_sum_to_parent() {
        let parent = DashaPeriod {
            entity: DashaEntity::Graha(Graha::Ketu),
            start_jd: 2451545.0,
            end_jd: 2451545.0 + 400.0,
            level: DashaLevel::Mahadasha,
            order: 1,
            parent_idx: 0,
        };
        let seq = test_sequence();
        let children = proportional_children(&parent, &seq, 400.0, DashaLevel::Antardasha, 0);
        assert_eq!(children.len(), 3);
        // First child starts at parent start
        assert!((children[0].start_jd - parent.start_jd).abs() < 1e-10);
        // Last child ends at parent end
        assert!((children[2].end_jd - parent.end_jd).abs() < 1e-10);
        // Adjacent: child[n].end == child[n+1].start
        assert!((children[0].end_jd - children[1].start_jd).abs() < 1e-10);
    }

    #[test]
    fn equal_children_uniform_duration() {
        let parent = DashaPeriod {
            entity: DashaEntity::Graha(Graha::Ketu),
            start_jd: 2451545.0,
            end_jd: 2451545.0 + 300.0,
            level: DashaLevel::Mahadasha,
            order: 1,
            parent_idx: 0,
        };
        let entities = vec![
            DashaEntity::Graha(Graha::Ketu),
            DashaEntity::Graha(Graha::Shukra),
            DashaEntity::Graha(Graha::Surya),
        ];
        let children = equal_children(&parent, &entities, DashaLevel::Antardasha, 0);
        assert_eq!(children.len(), 3);
        let expected_dur = 100.0;
        for c in &children {
            assert!((c.duration_days() - expected_dur).abs() < 1e-6);
        }
        assert!((children[2].end_jd - parent.end_jd).abs() < 1e-10);
    }

    #[test]
    fn cyclic_sequence_from_parent() {
        let seq = test_sequence();
        let result = build_cyclic_sequence(
            &seq,
            DashaEntity::Graha(Graha::Shukra),
            SubPeriodMethod::ProportionalFromParent,
        );
        assert_eq!(result[0].0, DashaEntity::Graha(Graha::Shukra));
        assert_eq!(result[1].0, DashaEntity::Graha(Graha::Surya));
        assert_eq!(result[2].0, DashaEntity::Graha(Graha::Ketu));
    }

    #[test]
    fn cyclic_sequence_from_next() {
        let seq = test_sequence();
        let result = build_cyclic_sequence(
            &seq,
            DashaEntity::Graha(Graha::Shukra),
            SubPeriodMethod::ProportionalFromNext,
        );
        assert_eq!(result[0].0, DashaEntity::Graha(Graha::Surya));
        assert_eq!(result[1].0, DashaEntity::Graha(Graha::Ketu));
        assert_eq!(result[2].0, DashaEntity::Graha(Graha::Shukra));
    }
}
