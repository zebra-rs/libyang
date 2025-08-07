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

impl RangeNode {
    pub fn to_string(&self) -> String {
        let mut out = String::from("");
        match &self {
            RangeNode::I8(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::I16(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::I32(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::I64(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::U8(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::U16(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::U32(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
            RangeNode::U64(range) => {
                for r in range.iter() {
                    if !out.is_empty() {
                        out.push('|');
                    }
                    out.push_str(&format!("{r}"));
                }
            }
        }
        format!("<{out}>")
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
