//! Role: The fixture records of the data table and the data display demos.

use super::state::Record;

pub(crate) fn records() -> Vec<Record> {
    let mut records = recent_records();
    records.extend(older_records());
    records
}

fn recent_records() -> Vec<Record> {
    [
        (
            "OPS-2041",
            "Section 3 periodic inspection",
            "12 switchboards / 4 cooling units",
            "Sou Mashiba",
            "RUNNING",
            kivact::interaction::Tone::Info,
            2,
            68.,
            "10-03 09:38",
        ),
        (
            "OPS-2040",
            "Storage stocktake (Q3)",
            "2,184 items in stock",
            "Mio Kuon",
            "PASSED",
            kivact::interaction::Tone::Success,
            1,
            100.,
            "10-03 08:12",
        ),
        (
            "OPS-2039",
            "Replace perimeter sensor",
            "6 units on the north side",
            "Riku Hayase",
            "PENDING",
            kivact::interaction::Tone::Warning,
            3,
            20.,
            "10-02 22:47",
        ),
        (
            "OPS-2038",
            "Payment gateway migration",
            "upstream v3 → v4",
            "Nagi Shiraishi",
            "FAILED",
            kivact::interaction::Tone::Danger,
            3,
            54.,
            "10-02 19:05",
        ),
    ]
    .into_iter()
    .map(
        |(id, title, description, owner, status, tone, priority, progress, updated)| Record {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            owner: owner.into(),
            status: status.into(),
            tone,
            priority,
            progress,
            updated: updated.into(),
        },
    )
    .collect()
}

fn older_records() -> Vec<Record> {
    [
        (
            "OPS-2037",
            "Extend audit log retention",
            "180 days → 400 days",
            "Sou Mashiba",
            "RUNNING",
            kivact::interaction::Tone::Info,
            2,
            41.,
            "10-02 16:30",
        ),
        (
            "OPS-2036",
            "Optimize night patrol routes",
            "8 routes",
            "Kai Shinonome",
            "PENDING",
            kivact::interaction::Tone::Warning,
            1,
            0.,
            "10-01 11:20",
        ),
        (
            "OPS-2035",
            "Relay firmware update",
            "fw 4.2.1",
            "Mio Kuon",
            "PASSED",
            kivact::interaction::Tone::Success,
            2,
            100.,
            "09-30 17:44",
        ),
        (
            "OPS-2034",
            "Archive legacy site data",
            "1.8TB",
            "Riku Hayase",
            "ARCHIVED",
            kivact::interaction::Tone::Neutral,
            1,
            100.,
            "09-28 09:02",
        ),
    ]
    .into_iter()
    .map(
        |(id, title, description, owner, status, tone, priority, progress, updated)| Record {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            owner: owner.into(),
            status: status.into(),
            tone,
            priority,
            progress,
            updated: updated.into(),
        },
    )
    .collect()
}
