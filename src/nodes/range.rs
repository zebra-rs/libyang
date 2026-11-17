use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum RangeVal<T> {
    Min,
    Max,
    Val(T),
}

impl<T: fmt::Display> fmt::Display for RangeVal<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            RangeVal::Min => write!(f, "min"),
            RangeVal::Max => write!(f, "max"),
            RangeVal::Val(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range<T> {
    pub start: RangeVal<T>,
    pub end: Option<RangeVal<T>>,
}

impl<T: fmt::Display> fmt::Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(end) = &self.end {
            write!(f, "{}..{}", self.start, end)
        } else {
            write!(f, "{}", self.start)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RangeNode {
    I8(Vec<Range<i8>>),
    I16(Vec<Range<i16>>),
    I32(Vec<Range<i32>>),
    I64(Vec<Range<i64>>),
    U8(Vec<Range<u8>>),
    U16(Vec<Range<u16>>),
    U32(Vec<Range<u32>>),
    U64(Vec<Range<u64>>),
}

impl fmt::Display for RangeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Every variant renders the same way and differs only in the
        // integer width it carries, so the arms just pick the width.
        fn join<T: fmt::Display>(ranges: &[Range<T>]) -> String {
            ranges
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join("|")
        }
        let out = match self {
            RangeNode::I8(range) => join(range),
            RangeNode::I16(range) => join(range),
            RangeNode::I32(range) => join(range),
            RangeNode::I64(range) => join(range),
            RangeNode::U8(range) => join(range),
            RangeNode::U16(range) => join(range),
            RangeNode::U32(range) => join(range),
            RangeNode::U64(range) => join(range),
        };
        write!(f, "<{out}>")
    }
}

impl RangeNode {
    /// The same ranges as ranges of the integer type `kind`, or `None`
    /// when `kind` is not an integer type or a bound does not fit it.
    ///
    /// A range written on a typedef is read before the typedef is
    /// resolved, as `i64`, or as `u64` when a bound is too large for
    /// that; this gives it the type the typedef turns out to stand for.
    pub fn cast(&self, kind: crate::YangType) -> Option<RangeNode> {
        use crate::YangType;
        fn wide<T: Copy + Into<i128>>(ranges: &[Range<T>]) -> Vec<Range<i128>> {
            let val = |v: &RangeVal<T>| match v {
                RangeVal::Min => RangeVal::Min,
                RangeVal::Max => RangeVal::Max,
                RangeVal::Val(v) => RangeVal::Val((*v).into()),
            };
            ranges
                .iter()
                .map(|r| Range {
                    start: val(&r.start),
                    end: r.end.as_ref().map(val),
                })
                .collect()
        }
        fn narrow<T: TryFrom<i128>>(ranges: &[Range<i128>]) -> Option<Vec<Range<T>>> {
            let val = |v: &RangeVal<i128>| {
                Some(match v {
                    RangeVal::Min => RangeVal::Min,
                    RangeVal::Max => RangeVal::Max,
                    RangeVal::Val(v) => RangeVal::Val(T::try_from(*v).ok()?),
                })
            };
            ranges
                .iter()
                .map(|r| {
                    Some(Range {
                        start: val(&r.start)?,
                        end: match &r.end {
                            Some(end) => Some(val(end)?),
                            None => None,
                        },
                    })
                })
                .collect()
        }
        let ranges = match self {
            RangeNode::I8(r) => wide(r),
            RangeNode::I16(r) => wide(r),
            RangeNode::I32(r) => wide(r),
            RangeNode::I64(r) => wide(r),
            RangeNode::U8(r) => wide(r),
            RangeNode::U16(r) => wide(r),
            RangeNode::U32(r) => wide(r),
            RangeNode::U64(r) => wide(r),
        };
        Some(match kind {
            YangType::Int8 => RangeNode::I8(narrow(&ranges)?),
            YangType::Int16 => RangeNode::I16(narrow(&ranges)?),
            YangType::Int32 => RangeNode::I32(narrow(&ranges)?),
            YangType::Int64 => RangeNode::I64(narrow(&ranges)?),
            YangType::Uint8 => RangeNode::U8(narrow(&ranges)?),
            YangType::Uint16 => RangeNode::U16(narrow(&ranges)?),
            YangType::Uint32 => RangeNode::U32(narrow(&ranges)?),
            YangType::Uint64 => RangeNode::U64(narrow(&ranges)?),
            _ => return None,
        })
    }
}

pub trait RangeExtract<T> {
    fn extract(&self) -> Option<Vec<Range<T>>>;
}

impl RangeExtract<u8> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<u8>>> {
        if let RangeNode::U8(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<u16> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<u16>>> {
        if let RangeNode::U16(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<u32> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<u32>>> {
        if let RangeNode::U32(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<u64> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<u64>>> {
        if let RangeNode::U64(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<i8> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<i8>>> {
        if let RangeNode::I8(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<i16> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<i16>>> {
        if let RangeNode::I16(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<i32> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<i32>>> {
        if let RangeNode::I32(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

impl RangeExtract<i64> for RangeNode {
    fn extract(&self) -> Option<Vec<Range<i64>>> {
        if let RangeNode::I64(node) = self {
            Some(node.to_vec())
        } else {
            None
        }
    }
}

pub trait MinMax<T> {
    fn min_value(&self) -> T;
    fn max_value(&self) -> T;
}

impl MinMax<i8> for i8 {
    fn min_value(&self) -> i8 {
        i8::MIN
    }

    fn max_value(&self) -> i8 {
        i8::MAX
    }
}

impl MinMax<i16> for i16 {
    fn min_value(&self) -> i16 {
        i16::MIN
    }

    fn max_value(&self) -> i16 {
        i16::MAX
    }
}

impl MinMax<i32> for i32 {
    fn min_value(&self) -> i32 {
        i32::MIN
    }

    fn max_value(&self) -> i32 {
        i32::MAX
    }
}

impl MinMax<i64> for i64 {
    fn min_value(&self) -> i64 {
        i64::MIN
    }

    fn max_value(&self) -> i64 {
        i64::MAX
    }
}

impl MinMax<u8> for u8 {
    fn min_value(&self) -> u8 {
        u8::MIN
    }

    fn max_value(&self) -> u8 {
        u8::MAX
    }
}

impl MinMax<u16> for u16 {
    fn min_value(&self) -> u16 {
        u16::MIN
    }

    fn max_value(&self) -> u16 {
        u16::MAX
    }
}

impl MinMax<u32> for u32 {
    fn min_value(&self) -> u32 {
        u32::MIN
    }

    fn max_value(&self) -> u32 {
        u32::MAX
    }
}

impl MinMax<u64> for u64 {
    fn min_value(&self) -> u64 {
        u64::MIN
    }

    fn max_value(&self) -> u64 {
        u64::MAX
    }
}

pub fn range_match<T: MinMax<T> + PartialOrd + Copy>(r: &Range<T>, v: T) -> bool {
    let start: T = match &r.start {
        RangeVal::Val(start) => *start,
        RangeVal::Min => v.min_value(),
        RangeVal::Max => v.max_value(),
    };
    if let Some(end) = &r.end {
        let end: T = match end {
            RangeVal::Val(end) => *end,
            RangeVal::Min => v.min_value(),
            RangeVal::Max => v.max_value(),
        };
        v >= start && v <= end
    } else {
        v == start
    }
}
