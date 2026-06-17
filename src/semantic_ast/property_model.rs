//! Typed priority, property, and duration helpers for semantic projections.

/// Normalized headline priority semantics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Priority {
    pub cookie: Option<PriorityCookie>,
    pub effective: PriorityValue,
}

impl Priority {
    /// Creates priority semantics from an optional source cookie value.
    pub fn from_cookie(raw: Option<String>) -> Self {
        let cookie = raw.and_then(PriorityCookie::parse);
        let effective = cookie
            .as_ref()
            .map(|cookie| cookie.value.clone())
            .unwrap_or_default();
        Self { cookie, effective }
    }

    /// Returns true when no explicit priority cookie was present.
    pub fn is_default(&self) -> bool {
        self.cookie.is_none()
    }

    /// Returns the raw source value inside the priority cookie.
    pub fn raw_cookie(&self) -> Option<&str> {
        self.cookie.as_ref().map(|cookie| cookie.raw.as_str())
    }

    /// Returns the normalized effective priority text.
    pub fn effective_text(&self) -> String {
        self.effective.to_normalized_string()
    }

    /// Returns the `org-get-priority` style score using Org's default A/B/C profile.
    pub fn org_priority_score(&self) -> Option<i32> {
        PriorityProfile::org_default().score_for_value(&self.effective)
    }

    /// Returns whether the effective value sits inside Org's default A/B/C profile.
    pub fn range_status(&self) -> PriorityRangeStatus {
        PriorityProfile::org_default().range_status_for_value(&self.effective)
    }

    /// Returns the `org-get-priority` style score using a caller-provided profile.
    pub fn score_with_profile(&self, profile: &PriorityProfile) -> Option<i32> {
        profile.score_for_value(&self.effective)
    }

    /// Returns whether the effective value sits inside a caller-provided profile.
    pub fn range_status_with_profile(&self, profile: &PriorityProfile) -> PriorityRangeStatus {
        profile.range_status_for_value(&self.effective)
    }
}

/// Explicit priority cookie projected from `[#...]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriorityCookie {
    pub raw: String,
    pub value: PriorityValue,
    pub normalized: String,
}

impl PriorityCookie {
    /// Parses the value inside a priority cookie.
    pub fn parse(raw: String) -> Option<Self> {
        let value = PriorityValue::parse(raw.as_str())?;
        let normalized = value.to_normalized_string();
        Some(Self {
            raw,
            value,
            normalized,
        })
    }
}

/// Priority value after parser-v2 normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PriorityValue {
    /// Alphabetic priority such as the default Org `A`, `B`, and `C` set.
    Letter(char),
    /// Numeric priority. Org's numeric priority custom variables use `0..=64`.
    Numeric(u8),
    /// Custom extension value outside official Org priority grammar.
    ///
    /// Native parsing and lint no longer produce this variant, but it remains
    /// in the public model so current consumers keep a stable enum shape.
    Custom(String),
}

impl PriorityValue {
    /// Parses a priority cookie value.
    pub fn parse(raw: &str) -> Option<Self> {
        let value = raw.trim();
        if value.is_empty() {
            return None;
        }
        if let Some(numeric) = parse_priority_numeric(value) {
            return Some(Self::Numeric(numeric));
        }

        let mut chars = value.chars();
        let first = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        if first.is_ascii_uppercase() {
            Some(Self::Letter(first))
        } else {
            None
        }
    }

    /// Returns normalized text used for agenda matching and display metadata.
    pub fn to_normalized_string(&self) -> String {
        match self {
            Self::Letter(value) => value.to_string(),
            Self::Numeric(value) => value.to_string(),
            Self::Custom(value) => value.clone(),
        }
    }

    fn org_numeric_value(&self) -> Option<i32> {
        match self {
            Self::Letter(value) => Some(*value as i32),
            Self::Numeric(value) => Some(i32::from(*value)),
            Self::Custom(_) => None,
        }
    }
}

impl Default for PriorityValue {
    fn default() -> Self {
        Self::Letter('B')
    }
}

fn parse_priority_numeric(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    match *bytes {
        [digit @ b'0'..=b'9'] => Some(digit - b'0'),
        [first @ b'1'..=b'5', second @ b'0'..=b'9'] => Some((first - b'0') * 10 + second - b'0'),
        [b'6', second @ b'0'..=b'4'] => Some(60 + second - b'0'),
        _ => None,
    }
}

/// Priority bounds used by Org to validate and score a priority cookie.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriorityProfile {
    highest: PriorityValue,
    lowest: PriorityValue,
    default: PriorityValue,
}

impl PriorityProfile {
    /// Creates a priority profile when all bounds belong to one valid priority family.
    pub fn new(
        highest: PriorityValue,
        lowest: PriorityValue,
        default: PriorityValue,
    ) -> Option<Self> {
        let family = priority_family(&highest)?;
        if priority_family(&lowest) != Some(family) || priority_family(&default) != Some(family) {
            return None;
        }
        let highest_value = highest.org_numeric_value()?;
        let lowest_value = lowest.org_numeric_value()?;
        let default_value = default.org_numeric_value()?;
        (highest_value <= default_value && default_value <= lowest_value).then_some(Self {
            highest,
            lowest,
            default,
        })
    }

    /// Returns Org's default `A`/`B`/`C` priority profile.
    pub fn org_default() -> Self {
        Self {
            highest: PriorityValue::Letter('A'),
            lowest: PriorityValue::Letter('C'),
            default: PriorityValue::Letter('B'),
        }
    }

    /// Returns the highest priority value in this profile.
    pub fn highest(&self) -> &PriorityValue {
        &self.highest
    }

    /// Returns the lowest priority value in this profile.
    pub fn lowest(&self) -> &PriorityValue {
        &self.lowest
    }

    /// Returns the implicit priority used when a headline has no explicit cookie.
    pub fn default_priority(&self) -> &PriorityValue {
        &self.default
    }

    /// Computes the same score shape as Org's `org-get-priority`.
    ///
    /// The score increases by 1000 for each priority step above the profile's
    /// lowest value.  Values outside the profile can still be scored, matching
    /// Org's runtime behavior; callers can use `range_status_for_value` to
    /// distinguish those cases.
    pub fn score_for_value(&self, value: &PriorityValue) -> Option<i32> {
        Some(1000 * (self.lowest.org_numeric_value()? - value.org_numeric_value()?))
    }

    /// Returns whether a priority value is inside this profile's configured bounds.
    pub fn range_status_for_value(&self, value: &PriorityValue) -> PriorityRangeStatus {
        let Some(value_family) = priority_family(value) else {
            return PriorityRangeStatus::Unsupported;
        };
        let Some(profile_family) = priority_family(&self.highest) else {
            return PriorityRangeStatus::Unsupported;
        };
        if value_family != profile_family {
            return PriorityRangeStatus::OutOfRange;
        }
        let Some(value) = value.org_numeric_value() else {
            return PriorityRangeStatus::Unsupported;
        };
        let Some(highest) = self.highest.org_numeric_value() else {
            return PriorityRangeStatus::Unsupported;
        };
        let Some(lowest) = self.lowest.org_numeric_value() else {
            return PriorityRangeStatus::Unsupported;
        };
        if highest <= value && value <= lowest {
            PriorityRangeStatus::InRange
        } else {
            PriorityRangeStatus::OutOfRange
        }
    }
}

impl Default for PriorityProfile {
    fn default() -> Self {
        Self::org_default()
    }
}

/// Profile membership for a parsed priority value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriorityRangeStatus {
    InRange,
    OutOfRange,
    Unsupported,
}

impl PriorityRangeStatus {
    /// Stable label for compact agent and JSON projections.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InRange => "inRange",
            Self::OutOfRange => "outOfRange",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PriorityFamily {
    Letter,
    Numeric,
}

fn priority_family(value: &PriorityValue) -> Option<PriorityFamily> {
    match value {
        PriorityValue::Letter(_) => Some(PriorityFamily::Letter),
        PriorityValue::Numeric(_) => Some(PriorityFamily::Numeric),
        PriorityValue::Custom(_) => None,
    }
}

/// Org duration value normalized to seconds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgDuration {
    pub raw: String,
    pub total_seconds: u64,
}

impl OrgDuration {
    /// Parses common Org duration forms such as `1:30`, `1:02:03`, `2h`, and
    /// `1d3h5min`.
    pub fn parse(raw: impl Into<String>) -> Option<Self> {
        let raw = raw.into();
        let minutes = duration_minutes(raw.trim())?;
        if !minutes.is_finite() || minutes.is_sign_negative() {
            return None;
        }
        Some(Self {
            raw,
            total_seconds: (minutes * 60.0).round() as u64,
        })
    }

    /// Returns the duration as minutes, preserving sub-minute HMS values.
    pub fn total_minutes(&self) -> f64 {
        self.total_seconds as f64 / 60.0
    }
}

fn duration_minutes(value: &str) -> Option<f64> {
    if value.is_empty() {
        return Some(0.0);
    }
    if let Some(minutes) = hms_minutes(value) {
        return Some(minutes);
    }
    if let Ok(minutes) = value.parse::<f64>() {
        return Some(minutes);
    }

    let mut cursor = 0;
    let mut minutes = 0.0;
    let mut parsed_unit = false;
    while cursor < value.len() {
        cursor = skip_ascii_whitespace(value, cursor);
        if cursor >= value.len() {
            break;
        }
        if parsed_unit && let Some((hms, next)) = hms_minutes_at(value, cursor) {
            let tail = skip_ascii_whitespace(value, next);
            if tail == value.len() {
                return Some(minutes + hms);
            }
        }

        let (number, after_number) = number_at(value, cursor)?;
        let after_spaces = skip_ascii_whitespace(value, after_number);
        let (unit, after_unit) = unit_at(value, after_spaces)?;
        minutes += number * unit_minutes(unit);
        cursor = after_unit;
        parsed_unit = true;
    }

    parsed_unit.then_some(minutes)
}

fn hms_minutes(value: &str) -> Option<f64> {
    let (minutes, next) = hms_minutes_at(value, 0)?;
    (skip_ascii_whitespace(value, next) == value.len()).then_some(minutes)
}

fn hms_minutes_at(value: &str, start: usize) -> Option<(f64, usize)> {
    let (hours, mut cursor) = unsigned_integer_at(value, start)?;
    if value.as_bytes().get(cursor) != Some(&b':') {
        return None;
    }
    cursor += 1;
    let (minutes, next) = two_digit_component_at(value, cursor)?;
    cursor = next;
    let mut seconds = 0;
    if value.as_bytes().get(cursor) == Some(&b':') {
        cursor += 1;
        let (parsed_seconds, next) = two_digit_component_at(value, cursor)?;
        seconds = parsed_seconds;
        cursor = next;
    }
    Some((
        hours as f64 * 60.0 + minutes as f64 + seconds as f64 / 60.0,
        cursor,
    ))
}

fn number_at(value: &str, start: usize) -> Option<(f64, usize)> {
    let bytes = value.as_bytes();
    let mut cursor = start;
    let mut seen_digit = false;
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
        seen_digit = true;
    }
    if cursor < bytes.len() && bytes[cursor] == b'.' {
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
            seen_digit = true;
        }
    }
    seen_digit
        .then(|| {
            value[start..cursor]
                .parse::<f64>()
                .ok()
                .map(|n| (n, cursor))
        })
        .flatten()
}

fn unsigned_integer_at(value: &str, start: usize) -> Option<(u64, usize)> {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    (cursor > start)
        .then(|| {
            value[start..cursor]
                .parse::<u64>()
                .ok()
                .map(|n| (n, cursor))
        })
        .flatten()
}

fn two_digit_component_at(value: &str, start: usize) -> Option<(u64, usize)> {
    let bytes = value.as_bytes();
    let tens = *bytes.get(start)?;
    let ones = *bytes.get(start + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some((u64::from((tens - b'0') * 10 + (ones - b'0')), start + 2))
}

fn unit_at(value: &str, start: usize) -> Option<(&'static str, usize)> {
    ["min", "h", "d", "w", "m", "y"]
        .into_iter()
        .find_map(|unit| {
            value[start..]
                .starts_with(unit)
                .then_some((unit, start + unit.len()))
        })
}

fn unit_minutes(unit: &str) -> f64 {
    match unit {
        "min" => 1.0,
        "h" => 60.0,
        "d" => 60.0 * 24.0,
        "w" => 60.0 * 24.0 * 7.0,
        "m" => 60.0 * 24.0 * 30.0,
        "y" => 60.0 * 24.0 * 365.25,
        _ => 0.0,
    }
}

fn skip_ascii_whitespace(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}
