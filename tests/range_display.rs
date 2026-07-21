// `RangeNode`'s rendering used to be an inherent `to_string`; it is now
// a `Display` impl (which still provides `.to_string()`, via the
// standard blanket impl). These cases pin the output format so the
// conversion cannot silently change what callers see — the first three
// were checked to pass against the previous implementation as well.

use libyang::{Range, RangeNode, RangeVal};

fn bounded<T>(start: T, end: T) -> Range<T> {
    Range {
        start: RangeVal::Val(start),
        end: Some(RangeVal::Val(end)),
    }
}

fn single<T>(v: T) -> Range<T> {
    Range {
        start: RangeVal::Val(v),
        end: None,
    }
}

#[test]
fn renders_single_and_multiple_ranges() {
    assert_eq!(RangeNode::U8(vec![bounded(0u8, 9u8)]).to_string(), "<0..9>");
    assert_eq!(RangeNode::I32(vec![single(42i32)]).to_string(), "<42>");

    // Several ranges are joined with '|', in order.
    let multi = RangeNode::I16(vec![
        bounded(-5i16, -1i16),
        single(0i16),
        bounded(1i16, 5i16),
    ]);
    assert_eq!(multi.to_string(), "<-5..-1|0|1..5>");
}

#[test]
fn renders_min_max_keywords() {
    let node = RangeNode::I64(vec![Range {
        start: RangeVal::Min,
        end: Some(RangeVal::Max),
    }]);
    assert_eq!(node.to_string(), "<min..max>");
}

#[test]
fn renders_empty_range_list() {
    // No ranges: the delimiters are still emitted, matching the
    // previous inherent implementation.
    assert_eq!(RangeNode::U64(vec![]).to_string(), "<>");
}

#[test]
fn usable_through_the_display_trait() {
    // The point of the conversion: RangeNode now works anywhere a
    // Display value is accepted, not just via an inherent method.
    let node = RangeNode::U32(vec![bounded(1u32, 2u32)]);
    assert_eq!(format!("{node}"), "<1..2>");
}
