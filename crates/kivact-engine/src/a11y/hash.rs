//! Role: Complete payload fingerprints; never an authority for accepting actions.
use kivact_types::protocol::{SemanticsNode, SemanticsProperties, Transform};
use kivact_types::{Rect, SemanticsText};
use std::hash::{DefaultHasher, Hash, Hasher};
fn rect(r: Rect, h: &mut DefaultHasher) {
    [r.x, r.y, r.width, r.height].map(f32::to_bits).hash(h);
}
fn text(t: &SemanticsText, h: &mut DefaultHasher) {
    let SemanticsText {
        runs,
        start,
        selection,
        caret,
    } = t;
    start.hash(h);
    selection.hash(h);
    rect(*caret, h);
    runs.len().hash(h);
    for r in runs.iter() {
        let kivact_types::SemanticsTextRun {
            id,
            value,
            bounds,
            carets,
        } = r.as_ref();
        id.hash(h);
        value.hash(h);
        rect(*bounds, h);
        carets.len().hash(h);
        for c in carets {
            let kivact_types::TextCaret {
                offset,
                rect: bounds,
            } = c;
            offset.hash(h);
            rect(*bounds, h);
        }
    }
}
fn props(p: &SemanticsProperties, h: &mut DefaultHasher) {
    let SemanticsProperties {
        text: t,
        indeterminate,
        value_min,
        value_max,
        value_now,
        live_region,
        sort,
        orientation,
        active_descendant,
        described_by,
        controls,
        labelled_by,
        level,
        set_size,
        pos_in_set,
        multiselectable,
        invalid,
        read_only,
        protected,
    } = p;
    t.is_some().hash(h);
    if let Some(t) = t {
        text(t, h);
    }
    indeterminate.hash(h);
    for v in [value_min, value_max, value_now] {
        v.map(f64::to_bits).hash(h);
    }
    live_region.hash(h);
    sort.hash(h);
    orientation.hash(h);
    active_descendant.hash(h);
    described_by.hash(h);
    controls.hash(h);
    labelled_by.hash(h);
    level.hash(h);
    set_size.hash(h);
    pos_in_set.hash(h);
    multiselectable.hash(h);
    invalid.hash(h);
    read_only.hash(h);
    protected.hash(h);
}
pub(super) fn fingerprint(n: &SemanticsNode) -> u64 {
    let SemanticsNode {
        id,
        role,
        children,
        label,
        description,
        value,
        bounds,
        scroll,
        transform,
        disabled,
        hidden,
        focusable,
        checked,
        selected,
        expanded,
        test_id,
        actions,
        properties,
    } = n;
    let mut h = DefaultHasher::new();
    id.hash(&mut h);
    role.hash(&mut h);
    children.hash(&mut h);
    label.hash(&mut h);
    description.hash(&mut h);
    value.hash(&mut h);
    rect(*bounds, &mut h);
    [scroll.x.to_bits(), scroll.y.to_bits()].hash(&mut h);
    let Transform { a, b, c, d, e, f } = transform;
    [a, b, c, d, e, f].map(|v| v.to_bits()).hash(&mut h);
    disabled.hash(&mut h);
    hidden.hash(&mut h);
    focusable.hash(&mut h);
    checked.hash(&mut h);
    selected.hash(&mut h);
    expanded.hash(&mut h);
    test_id.hash(&mut h);
    actions.hash(&mut h);
    properties.is_some().hash(&mut h);
    if let Some(p) = properties {
        props(p, &mut h);
    }
    h.finish()
}
