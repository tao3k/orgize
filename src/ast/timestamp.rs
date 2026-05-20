//! Typed syntax wrapper helpers for Org timestamps.

use super::{SyntaxTimestamp, filter_token};
use crate::syntax::SyntaxKind;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Unit used by timestamp repeater and delay cookies.
pub enum SyntaxTimeUnit {
    /// Hour unit.
    Hour,
    /// Day unit.
    Day,
    /// Week unit.
    Week,
    /// Month unit.
    Month,
    /// Year unit.
    Year,
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Repeater mode for an Org timestamp.
pub enum RepeaterType {
    /// Cumulate repeater.
    Cumulate,
    /// Catch-up repeater.
    CatchUp,
    /// Restart repeater.
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Warning delay mode for an Org timestamp.
pub enum DelayType {
    /// Warn for all matching occurrences.
    All,
    /// Warn only for the first occurrence.
    First,
}

impl SyntaxTimestamp {
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let ts = Org::parse("<2003-09-16 Tue 09:39-10:39>").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_active());
    /// let ts = Org::parse("<2003-09-16 Tue 09:39>--<2003-09-16 Tue 10:39>").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_active());
    /// let ts = Org::parse("<2003-09-16 Tue 09:39>").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_ACTIVE
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_inactive());
    /// ```
    pub fn is_inactive(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_INACTIVE
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let ts = Org::parse("<%%(org-calendar-holiday)>").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_diary());
    /// ```
    pub fn is_diary(&self) -> bool {
        self.syntax.kind() == SyntaxKind::TIMESTAMP_DIARY
    }

    /// Returns `true` if this timestamp has a range
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_range());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_range());
    /// let ts = Org::parse("[2003-09-16]--[2003-09-16]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.is_range());
    /// let ts = Org::parse("[2003-09-16 Tue 09:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(!ts.is_range());
    /// ```
    pub fn is_range(&self) -> bool {
        self.syntax
            .children_with_tokens()
            .filter_map(filter_token(SyntaxKind::MINUS))
            .count()
            > 2
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::{SyntaxTimestamp, RepeaterType}};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), Some(RepeaterType::Cumulate));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), Some(RepeaterType::Restart));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_type(), None);
    /// ```
    pub fn repeater_type(&self) -> Option<RepeaterType> {
        self.nth_repeater(0).map(|i| i.0)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), Some(1));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), Some(10));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_value(), None);
    /// ```
    pub fn repeater_value(&self) -> Option<u32> {
        self.nth_repeater(0).map(|i| i.1)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::{SyntaxTimestamp, SyntaxTimeUnit}};
    ///
    /// let t = Org::parse("[2000-01-01 +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), Some(SyntaxTimeUnit::Week));
    /// let t = Org::parse("[2000-01-01 .+10d +1w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), Some(SyntaxTimeUnit::Day));
    /// let t = Org::parse("[2000-01-01 --1y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.repeater_unit(), None);
    /// ```
    pub fn repeater_unit(&self) -> Option<SyntaxTimeUnit> {
        self.nth_repeater(0).map(|i| i.2)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::{SyntaxTimestamp, DelayType}};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::All));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::All));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_type(), Some(DelayType::First));
    /// ```
    pub fn warning_type(&self) -> Option<DelayType> {
        self.nth_delay(0).map(|i| i.0)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(3));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(5));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_value(), Some(10));
    /// ```
    pub fn warning_value(&self) -> Option<u32> {
        self.nth_delay(0).map(|i| i.1)
    }

    /// ```rust
    /// use orgize::{Org, syntax_ast::{SyntaxTimestamp, SyntaxTimeUnit}};
    ///
    /// let t = Org::parse("[2000-01-01 -3y]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(SyntaxTimeUnit::Year));
    /// let t = Org::parse("[2000-01-01]--[2000-01-02 -5w]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(SyntaxTimeUnit::Week));
    /// let t = Org::parse("[2000-01-01 01:00-02:00 --10m]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(t.warning_unit(), Some(SyntaxTimeUnit::Month));
    /// ```
    pub fn warning_unit(&self) -> Option<SyntaxTimeUnit> {
        self.nth_delay(0).map(|i| i.2)
    }

    fn nth_repeater(&self, nth: usize) -> Option<(RepeaterType, u32, SyntaxTimeUnit)> {
        let mut i = nth + 1;

        let mut iter = self.syntax.children_with_tokens().skip_while(|n| {
            if n.kind() == SyntaxKind::TIMESTAMP_REPEATER_MARK {
                i -= 1;
                i != 0
            } else {
                true
            }
        });

        let mark = iter.next().and_then(|n| match n.as_token()?.text() {
            "++" => Some(RepeaterType::CatchUp),
            "+" => Some(RepeaterType::Cumulate),
            ".+" => Some(RepeaterType::Restart),
            _ => None,
        })?;
        let value = iter
            .next()
            .and_then(|n| n.as_token()?.text().parse::<u32>().ok())?;
        let unit = iter.next().and_then(|n| match n.as_token()?.text() {
            "h" => Some(SyntaxTimeUnit::Hour),
            "d" => Some(SyntaxTimeUnit::Day),
            "w" => Some(SyntaxTimeUnit::Week),
            "m" => Some(SyntaxTimeUnit::Month),
            "y" => Some(SyntaxTimeUnit::Year),
            _ => None,
        })?;

        Some((mark, value, unit))
    }

    fn nth_delay(&self, nth: usize) -> Option<(DelayType, u32, SyntaxTimeUnit)> {
        let mut i = nth + 1;

        let mut iter = self.syntax.children_with_tokens().skip_while(|n| {
            if n.kind() == SyntaxKind::TIMESTAMP_DELAY_MARK {
                i -= 1;
                i != 0
            } else {
                true
            }
        });

        let mark = iter.next().and_then(|n| match n.as_token()?.text() {
            "-" => Some(DelayType::All),
            "--" => Some(DelayType::First),
            _ => None,
        })?;
        let value = iter
            .next()
            .and_then(|n| n.as_token()?.text().parse::<u32>().ok())?;
        let unit = iter.next().and_then(|n| match n.as_token()?.text() {
            "h" => Some(SyntaxTimeUnit::Hour),
            "d" => Some(SyntaxTimeUnit::Day),
            "w" => Some(SyntaxTimeUnit::Week),
            "m" => Some(SyntaxTimeUnit::Month),
            "y" => Some(SyntaxTimeUnit::Year),
            _ => None,
        })?;

        Some((mark, value, unit))
    }

    /// Converts timestamp start to chrono NaiveDateTime
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    /// use chrono::NaiveDateTime;
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(ts.start_to_chrono().unwrap(), "2003-09-16T09:39:00".parse::<NaiveDateTime>().unwrap());
    ///
    /// let ts = Org::parse("[2003-13-00 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert!(ts.start_to_chrono().is_none());
    /// ```
    #[cfg(feature = "chrono")]
    pub fn start_to_chrono(&self) -> Option<chrono::NaiveDateTime> {
        Some(chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                self.year_start()?.parse().ok()?,
                self.month_start()?.parse().ok()?,
                self.day_start()?.parse().ok()?,
            )?,
            chrono::NaiveTime::from_hms_opt(
                self.hour_start()?.parse().ok()?,
                self.minute_start()?.parse().ok()?,
                0,
            )?,
        ))
    }

    /// Converts timestamp end to chrono NaiveDateTime
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    /// use chrono::NaiveDateTime;
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(ts.end_to_chrono().unwrap(), "2003-09-16T10:39:00".parse::<NaiveDateTime>().unwrap());
    /// ```
    #[cfg(feature = "chrono")]
    pub fn end_to_chrono(&self) -> Option<chrono::NaiveDateTime> {
        Some(chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(
                self.year_end()?.parse().ok()?,
                self.month_end()?.parse().ok()?,
                self.day_end()?.parse().ok()?,
            )?,
            chrono::NaiveTime::from_hms_opt(
                self.hour_end()?.parse().ok()?,
                self.minute_end()?.parse().ok()?,
                0,
            )?,
        ))
    }

    /// Returns chrono::TimeDelta between timestamp start and end
    ///
    /// ```rust
    /// use orgize::{Org, syntax_ast::SyntaxTimestamp};
    ///
    /// let ts = Org::parse("[2003-09-16 Tue 09:39-10:39]").first_node::<SyntaxTimestamp>().unwrap();
    /// assert_eq!(ts.time_delta().unwrap().num_hours(), 1);
    /// ```
    #[cfg(feature = "chrono")]
    pub fn time_delta(&self) -> Option<chrono::TimeDelta> {
        Some(self.end_to_chrono()? - self.start_to_chrono()?)
    }
}
