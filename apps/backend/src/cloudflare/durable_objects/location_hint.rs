use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableObjectLocationHint {
    Wnam,
    Enam,
    Weur,
    Eeur,
    Apac,
    Oc,
}

impl DurableObjectLocationHint {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wnam => "wnam",
            Self::Enam => "enam",
            Self::Weur => "weur",
            Self::Eeur => "eeur",
            Self::Apac => "apac",
            Self::Oc => "oc",
        }
    }
}

impl fmt::Display for DurableObjectLocationHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct ParseDurableObjectLocationHintError(pub String);

impl fmt::Display for ParseDurableObjectLocationHintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported durable object location hint: {}", self.0)
    }
}

impl std::error::Error for ParseDurableObjectLocationHintError {}

impl FromStr for DurableObjectLocationHint {
    type Err = ParseDurableObjectLocationHintError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "wnam" => Ok(Self::Wnam),
            "enam" => Ok(Self::Enam),
            "weur" => Ok(Self::Weur),
            "eeur" => Ok(Self::Eeur),
            "apac" => Ok(Self::Apac),
            "oc" => Ok(Self::Oc),
            other => Err(ParseDurableObjectLocationHintError(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableObjectJurisdiction(String);

impl DurableObjectJurisdiction {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn new(value: &str) -> Result<Self, ParseDurableObjectJurisdictionError> {
        value.parse()
    }
}

impl fmt::Display for DurableObjectJurisdiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ParseDurableObjectJurisdictionError(pub String);

impl fmt::Display for ParseDurableObjectJurisdictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported durable object jurisdiction: {}", self.0)
    }
}

impl std::error::Error for ParseDurableObjectJurisdictionError {}

impl FromStr for DurableObjectJurisdiction {
    type Err = ParseDurableObjectJurisdictionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Err(ParseDurableObjectJurisdictionError(String::new()));
        }

        if trimmed.len() > 32 {
            return Err(ParseDurableObjectJurisdictionError(trimmed));
        }

        if !trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ParseDurableObjectJurisdictionError(trimmed));
        }

        Ok(Self(trimmed))
    }
}
