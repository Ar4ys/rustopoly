use std::ops::RangeBounds;

use js_sys::Math;

pub fn get_usize(range: impl RangeBounds<usize>) -> usize {
    let min = match range.start_bound() {
        std::ops::Bound::Included(min) | std::ops::Bound::Excluded(min) => *min,
        std::ops::Bound::Unbounded => usize::MIN,
    };

    let (max, inclusivity) = match range.end_bound() {
        std::ops::Bound::Included(max) => (*max, 1),
        std::ops::Bound::Excluded(max) => (*max, 0),
        std::ops::Bound::Unbounded => (usize::MAX, 0),
    };

    (Math::random() * (max - min + inclusivity) as f64 + min as f64) as usize
}
