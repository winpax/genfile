use std::{ops::Mul, str::FromStr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid multiplier. Valid multipliers are b, kb, mb, gb, tb")]
    InvalidMultiplier,
    #[error("Failed to parse size: {0}")]
    InvalidSize(#[from] std::num::ParseFloatError),
    #[error("Found two periods in the float input")]
    DoublePeriod,
    #[error("Missing numeric value")]
    MissingValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplier {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
    Terabyte,
}

impl Multiplier {
    #[inline(always)]
    pub fn to_bytes(self) -> f64 {
        match self {
            Multiplier::Byte => 1.0,
            Multiplier::Kilobyte => &Self::Byte.to_bytes() * 1024.0,
            Multiplier::Megabyte => &Self::Kilobyte.to_bytes() * 1024.0,
            Multiplier::Gigabyte => &Self::Megabyte.to_bytes() * 1024.0,
            Multiplier::Terabyte => Self::Gigabyte.to_bytes() * 1024.0,
        }
    }
}

// impl Mul for Multiplier {
//     type Output = Self;

//     #[inline(always)]
//     fn mul(self, rhs: Self) -> Self::Output {
//         match (self, rhs) {
//             (Multiplier::Byte, Multiplier::Byte) => Multiplier::Byte,
//             (Multiplier::Byte, Multiplier::Kilobyte) => Multiplier::Kilobyte,
//             (Multiplier::Byte, Multiplier::Megabyte) => Multiplier::Megabyte,
//             (Multiplier::Byte, Multiplier::Gigabyte) => Multiplier::Gigabyte,
//             (Multiplier::Byte, Multiplier::Terabyte) => Multiplier::Terabyte,
//             (Multiplier::Kilobyte, Multiplier::Byte) => Multiplier::Kilobyte,
//             (Multiplier::Kilobyte, Multiplier::Kilobyte) => Multiplier::Kilobyte,
//             (Multiplier::Kilobyte, Multiplier::Megabyte) => Multiplier::Megabyte,
//             (Multiplier::Kilobyte, Multiplier::Gigabyte) => Multiplier::Gigabyte,
//             (Multiplier::Kilobyte, Multiplier::Terabyte) => Multiplier::Terabyte,
//             (Multiplier::Megabyte, Multiplier::Byte) => Multiplier::Megabyte,
//             (Multiplier::Megabyte, Multiplier::Kilobyte) => Multiplier::Megabyte,
//             (Multiplier::Megabyte, Multiplier::Megabyte) => Multiplier::Megabyte,
//             (Multiplier::Megabyte, Multiplier::Gigabyte) => Multiplier::Gigabyte,
//             (Multiplier::Megabyte, Multiplier::Terabyte) => Multiplier::Terabyte,
//             (Multiplier::Gigabyte, Multiplier::Byte) => Multiplier::Gigabyte,
//             (Multiplier::Gigabyte, Multiplier::Kilobyte) => Multiplier::Gigabyte,
//             (Multiplier::Gigabyte, Multiplier::Megabyte) => Multiplier::Gigabyte,
//             (Multiplier::Gigabyte, Multiplier::Gigabyte) => Multiplier::Gigabyte,
//             (Multiplier::Gigabyte, Multiplier::Terabyte) => Multiplier::Terabyte,
//             (Multiplier::Terabyte, Multiplier::Byte) => Multiplier::Terabyte,
//             (Multiplier::Terabyte, Multiplier::Kilobyte) => Multiplier::Terabyte,
//             (Multiplier::Terabyte, Multiplier::Megabyte) => Multiplier::Terabyte,
//             (Multiplier::Terabyte, Multiplier::Gigabyte) => Multiplier::Terabyte,
//             (Multiplier::Terabyte, Multiplier::Terabyte) => Multiplier::Terabyte,
//         }
//     }
// }

impl Mul<f64> for Multiplier {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.to_bytes() * rhs
    }
}

impl FromStr for Multiplier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "b" | "byte" | "bytes" => Ok(Multiplier::Byte),
            "k" | "kb" | "kilobyte" | "kilobytes" => Ok(Multiplier::Kilobyte),
            "m" | "mb" | "megabyte" | "megabytes" => Ok(Multiplier::Megabyte),
            "g" | "gb" | "gigabyte" | "gigabytes" => Ok(Multiplier::Gigabyte),
            "t" | "tb" | "terabyte" | "terabytes" => Ok(Multiplier::Terabyte),
            _ => Err(Error::InvalidMultiplier),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    value: f64,
    multiplier: Multiplier,
}

impl Size {
    pub fn to_bytes(self) -> u64 {
        (self.value * self.multiplier.to_bytes()) as u64
    }
}

impl FromStr for Size {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::MissingValue);
        }

        if let Some((value, multiplier)) = s.split_once(' ') {
            let value = value.parse()?;

            let multiplier = multiplier.parse().map_err(|_| Error::InvalidMultiplier)?;

            Ok(Size { value, multiplier })
        } else {
            let chars = s.chars().collect::<Vec<_>>();

            let mut has_period = false;
            let mut float_index = chars.len();

            for (i, c) in chars.iter().enumerate() {
                if c.is_ascii_digit() {
                    continue;
                }

                if *c == '.' {
                    if has_period {
                        return Err(Error::DoublePeriod);
                    }

                    has_period = true;
                    continue;
                }

                float_index = i;
                break;
            }

            let number = chars[0..float_index].iter().collect::<String>().parse()?;

            let multiplier = chars[float_index..].iter().collect::<String>().parse()?;

            Ok(Self {
                value: number,
                multiplier,
            })
        }
    }
}
