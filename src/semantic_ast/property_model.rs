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
    /// Single-character custom priority outside ASCII letters and digits.
    Custom(String),
}

impl PriorityValue {
    /// Parses a priority cookie value.
    pub fn parse(raw: &str) -> Option<Self> {
        let value = raw.trim();
        if value.is_empty() {
            return None;
        }
        if value.chars().all(|ch| ch.is_ascii_digit()) {
            let numeric = value.parse::<u8>().ok()?;
            return (numeric <= 64).then_some(Self::Numeric(numeric));
        }

        let mut chars = value.chars();
        let first = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        if first.is_ascii_alphabetic() {
            Some(Self::Letter(first.to_ascii_uppercase()))
        } else {
            Some(Self::Custom(first.to_string()))
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
}

impl Default for PriorityValue {
    fn default() -> Self {
        Self::Letter('B')
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
        if parsed_unit {
            if let Some((hms, next)) = hms_minutes_at(value, cursor) {
                let tail = skip_ascii_whitespace(value, next);
                if tail == value.len() {
                    return Some(minutes + hms);
                }
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
