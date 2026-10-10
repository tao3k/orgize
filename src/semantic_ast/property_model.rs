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
        let row = super::org_values::optional("priority", raw)?;
        let [kind, value]: [String; 2] = row.try_into().expect("native priority arity");
        match kind.as_str() {
            "numeric" => Some(Self::Numeric(
                value.parse().expect("native priority numeric"),
            )),
            "letter" => {
                let mut chars = value.chars();
                let letter = chars.next().expect("native priority letter");
                assert!(chars.next().is_none(), "native priority letter arity");
                Some(Self::Letter(letter))
            }
            _ => panic!("native priority kind"),
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
        let row = super::org_values::optional("duration", &raw)?;
        let [seconds]: [String; 1] = row.try_into().expect("native duration arity");
        Some(Self {
            raw,
            total_seconds: seconds.parse().expect("native duration seconds"),
        })
    }

    /// Returns the duration as minutes, preserving sub-minute HMS values.
    pub fn total_minutes(&self) -> f64 {
        self.total_seconds as f64 / 60.0
    }
}
