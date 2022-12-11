
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    GB(usize),
    MB(usize),
    KB(usize),
    B(usize),
    Gb(usize),
    Mb(usize),
    Kb(usize),
    b(usize),
}

impl Unit {
    pub fn from_str(count: usize, unit: &str) -> Option<Self> {
        match unit {
            "GB" => Some(Self::GB(count)),
            "MB" => Some(Self::MB(count)),
            "KB" => Some(Self::KB(count)),
            "B" => Some(Self::B(count)),
            "Gb" => Some(Self::Gb(count)),
            "Mb" => Some(Self::Mb(count)),
            "Kb" => Some(Self::Kb(count)),
            "b" => Some(Self::b(count)),
            _ => None
        }
    }
    
    pub fn to_bytes(&self) -> usize {
        match self {
            Unit::GB(count) => count * 1024 * 1024 * 1024,
            Unit::MB(count) => count * 1024 * 1024,
            Unit::KB(count) => count * 1024,
            Unit::B(count) => count * 1,
            Unit::Gb(count) => count * 1000 * 1000 * 1000 / 8,
            Unit::Mb(count) => count * 1000 * 1000 / 8,
            Unit::Kb(count) => count * 1000 / 8,
            Unit::b(count) => count / 8,
        }
    }
}