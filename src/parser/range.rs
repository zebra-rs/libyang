use crate::modules::*;
use nom::branch::{alt, permutation};
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit0, multispace0, one_of};
use nom::combinator::{opt, recognize};
use nom::error::ErrorKind;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::Err::Error;
use nom::IResult;

// We owe integer parsing logic from
// https://codeandbitters.com/lets-build-a-parser/.

fn digit1to9(input: &str) -> IResult<&str, char> {
    one_of("123456789")(input)
}

fn uint_parse(input: &str) -> IResult<&str, &str> {
    alt((tag("0"), recognize(pair(digit1to9, digit0))))(input)
}

fn int_parse(input: &str) -> IResult<&str, &str> {
    recognize(pair(opt(tag("-")), uint_parse))(input)
}

fn range_value_parse<T>(s: &str) -> IResult<&str, RangeVal<T>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    match s {
        "max" => Ok((s, RangeVal::<T>::Max)),
        "min" => Ok((s, RangeVal::<T>::Min)),
        v => {
            if let Ok(n) = v.parse::<T>() {
                Ok((s, RangeVal::<T>::Val(n)))
            } else {
                Err(Error(nom::error::Error::new(s, ErrorKind::Digit)))
            }
        }
    }
}

fn range_single_parse<T>(
    s: &str,
    digit_parse: fn(&str) -> IResult<&str, &str>,
) -> IResult<&str, Range<T>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let (s, _) = multispace0(s)?;
    let (s, r) = alt((tag("min"), tag("max"), digit_parse))(s)?;
    let (_, val) = range_value_parse::<T>(r)?;
    let (s, _) = multispace0(s)?;
    let range = Range::<T> {
        start: val,
        end: RangeVal::None,
    };
    Ok((s, range))
}

fn range_pair_parse<T>(
    s: &str,
    digit_parse: fn(&str) -> IResult<&str, &str>,
) -> IResult<&str, Range<T>>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let (s, _) = multispace0(s)?;
    let (s, r1) = alt((tag("min"), digit_parse))(s)?;
    let (_, start) = range_value_parse::<T>(r1)?;
    let (s, _) = multispace0(s)?;
    let (s, _) = tag("..")(s)?;
    let (s, _) = multispace0(s)?;
    let (s, r2) = alt((tag("max"), digit_parse))(s)?;
    let (_, end) = range_value_parse::<T>(r2)?;
    let range = Range::<T> { start, end };
    Ok((s, range))
}

fn range_int_single_parse(s: &str) -> IResult<&str, Range<i64>> {
    range_single_parse::<i64>(s, int_parse)
}

fn range_uint_single_parse(s: &str) -> IResult<&str, Range<u64>> {
    range_single_parse::<u64>(s, uint_parse)
}

fn range_int_pair_parse(s: &str) -> IResult<&str, Range<i64>> {
    range_pair_parse::<i64>(s, int_parse)
}

fn range_uint_pair_parse(s: &str) -> IResult<&str, Range<u64>> {
    range_pair_parse::<u64>(s, uint_parse)
}

pub fn range_int_parse(s: &str) -> IResult<&str, Vec<Range<i64>>> {
    separated_list1(
        permutation((multispace0, char('|'), multispace0)),
        alt((range_int_pair_parse, range_int_single_parse)),
    )(s)
}

pub fn range_uint_parse(s: &str) -> IResult<&str, Vec<Range<u64>>> {
    separated_list1(
        permutation((multispace0, char('|'), multispace0)),
        alt((range_uint_pair_parse, range_uint_single_parse)),
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC7951
    //
    // 9.2.4.  The "range" Statement
    //
    // The "range" statement, which is an optional substatement to the "type"
    // statement, takes as an argument a range expression string. It is used to
    // restrict integer and decimal built-in types, or types derived from them.
    //
    // A range consists of an explicit value, or a lower-inclusive bound, two
    // consecutive dots "..", and an upper-inclusive bound. Multiple values or
    // ranges can be given, separated by "|". If multiple values or ranges are
    // given, they all MUST be disjoint and MUST be in ascending order. If a
    // range restriction is applied to a type that is already range-restricted,
    // the new restriction MUST be equally limiting or more limiting, i.e.,
    // raising the lower bounds, reducing the upper bounds, removing explicit
    // values or ranges, or splitting ranges into multiple ranges with
    // intermediate gaps. Each explicit value and range boundary value given in
    // the range expression MUST match the type being restricted or be one of
    // the special values "min" or "max". "min" and "max" mean the minimum and
    // maximum values accepted for the type being restricted, respectively.
    //
    // 9.2.5.  Usage Example
    //
    // typedef my-base-int32-type {
    //     type int32 {
    //         range "1..4 | 10..20";
    //     }
    // }
    //
    // typedef my-type1 {
    //     type my-base-int32-type {
    //         // legal range restriction
    //         range "11..max"; // 11..20
    //     }
    // }
    //
    // typedef my-type2 {
    //     type my-base-int32-type {
    //         // illegal range restriction
    //         range "11..100";
    //     }
    // }
    #[test]
    fn test_int_parse() {
        struct Test {
            input: &'static str,
            output: IResult<&'static str, &'static str>,
        }
        let tests = [
            Test {
                input: "0",
                output: Ok(("", "0")),
            },
            Test {
                input: "-0",
                output: Ok(("", "-0")),
            },
            Test {
                input: "1",
                output: Ok(("", "1")),
            },
            Test {
                input: "-1",
                output: Ok(("", "-1")),
            },
            Test {
                input: "123",
                output: Ok(("", "123")),
            },
            Test {
                input: "-123",
                output: Ok(("", "-123")),
            },
            Test {
                input: "-1020",
                output: Ok(("", "-1020")),
            },
            Test {
                input: "2020",
                output: Ok(("", "2020")),
            },
        ];
        for t in &tests {
            let result = int_parse(t.input);
            assert_eq!(result, t.output);
        }
    }

    #[test]
    fn test_uint_parse() {
        struct Test {
            input: &'static str,
            output: IResult<&'static str, &'static str>,
        }
        let tests = [
            Test {
                input: "0",
                output: Ok(("", "0")),
            },
            Test {
                input: "00",
                output: Ok(("0", "0")),
            },
            Test {
                input: "0123",
                output: Ok(("123", "0")),
            },
            Test {
                input: "123",
                output: Ok(("", "123")),
            },
            Test {
                input: "2020",
                output: Ok(("", "2020")),
            },
            Test {
                input: "-2020",
                output: Err(Error(nom::error::Error::new("-2020", ErrorKind::OneOf))),
            },
        ];
        for t in &tests {
            let result = uint_parse(t.input);
            assert_eq!(result, t.output);
        }
    }

    #[test]
    fn test_range_value_parse() {
        struct Test {
            input: &'static str,
            output: IResult<&'static str, RangeVal<i64>>,
        }
        let tests = [
            Test {
                input: "100",
                output: Ok(("100", RangeVal::<i64>::Val(100))),
            },
            Test {
                input: "-0",
                output: Ok(("-0", RangeVal::<i64>::Val(0))),
            },
            Test {
                input: "-1",
                output: Ok(("-1", RangeVal::<i64>::Val(-1))),
            },
            Test {
                input: "max",
                output: Ok(("max", RangeVal::<i64>::Max)),
            },
        ];
        for t in &tests {
            let result = range_value_parse::<i64>(t.input);
            assert_eq!(result, t.output);
        }
    }

    #[test]
    fn test_range_uint_single_parse() {
        struct Test {
            input: &'static str,
            output: IResult<&'static str, RangeUint>,
        }
        let tests = [
            Test {
                input: "128",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Val(128u64),
                        end: RangeVal::None,
                    },
                )),
            },
            Test {
                input: "max",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Max,
                        end: RangeVal::None,
                    },
                )),
            },
            Test {
                input: "0",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Val(0u64),
                        end: RangeVal::None,
                    },
                )),
            },
            Test {
                input: "-0",
                output: Err(Error(nom::error::Error::new("-0", ErrorKind::OneOf))),
            },
            Test {
                input: "-100",
                output: Err(Error(nom::error::Error::new("-100", ErrorKind::OneOf))),
            },
            Test {
                input: "abc",
                output: Err(Error(nom::error::Error::new("abc", ErrorKind::OneOf))),
            },
        ];
        for t in &tests {
            let result = range_uint_single_parse(t.input);
            assert_eq!(result, t.output);
        }
    }

    #[test]
    fn test_range_uint_pair_parse() {
        struct Test {
            input: &'static str,
            output: IResult<&'static str, RangeUint>,
        }
        let tests = [
            Test {
                input: "0..1",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Val(0u64),
                        end: RangeVal::Val(1u64),
                    },
                )),
            },
            Test {
                input: "1..100",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Val(1u64),
                        end: RangeVal::Val(100u64),
                    },
                )),
            },
            Test {
                input: "100..1",
                output: Ok((
                    "",
                    RangeUint {
                        start: RangeVal::Val(100u64),
                        end: RangeVal::Val(1u64),
                    },
                )),
            },
            Test {
                input: "-0..1",
                output: Err(Error(nom::error::Error::new("-0..1", ErrorKind::OneOf))),
            },
            Test {
                input: "-1..1",
                output: Err(Error(nom::error::Error::new("-1..1", ErrorKind::OneOf))),
            },
            Test {
                input: "100..-1",
                output: Err(Error(nom::error::Error::new("-1", ErrorKind::OneOf))),
            },
        ];
        for t in &tests {
            let result = range_uint_pair_parse(t.input);
            assert_eq!(result, t.output);
        }
    }

    #[test]
    fn test_range_uint_parse() {
        let literal = "1 .. 20";
        let result = range_uint_parse(literal);
        let expect = vec![RangeUint {
            start: RangeVal::Val(1u64),
            end: RangeVal::Val(20u64),
        }];
        assert_eq!(result, Ok(("", expect)));

        let literal = "0..20";
        let result = range_uint_parse(literal);
        let expect = vec![RangeUint {
            start: RangeVal::Val(0u64),
            end: RangeVal::Val(20u64),
        }];
        assert_eq!(result, Ok(("", expect)));

        let literal = "-1.. 20";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "min..20";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "max..20";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "68..max";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "min..max";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "min";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "1..20 | 22..24";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "1..20 | 22..24 | 35..100";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "0 | 1..10 | max";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "0 | 30..65535";
        let result = range_uint_parse(literal);
        println!("{:?}", result);

        let literal = "1..14 | 36 | 40 | 44| 48 | 52 | 56 | 60 | 64 | 100 | 104 | 108 | 112 | 116 | 120 | 124 | 128 | 132 | 136 | 140 | 144 | 149 | 153 | 157 | 161 | 165";
        let result = range_uint_parse(literal);
        println!("{:?}", result);
    }

    #[test]
    fn parse_minus_zero() {
        // minus zero is valid for integer value.
        let literal = "-0";
        let n = literal.parse::<i64>().unwrap();
        assert_eq!(n, 0);
    }
}
