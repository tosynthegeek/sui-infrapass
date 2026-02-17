use std::str::FromStr;

use crate::utils::constants::TEST_TOKEN_PACKAGE_ID;
use anyhow::{Ok, Result};
use sui_types::TypeTag;

#[derive(Debug, Clone)]
pub enum CoinType {
    SUI,
    WAL,
    USDC,
    USDT,
}

impl CoinType {
    pub fn to_type_tag(&self) -> Result<TypeTag> {
        let type_str = match self {
            CoinType::SUI => "0x2::sui::SUI".to_string(),

            CoinType::WAL => {
                format!("{TEST_TOKEN_PACKAGE_ID}::wal::WAL")
            }

            CoinType::USDC => {
                format!("{TEST_TOKEN_PACKAGE_ID}::usdc::USDC")
            }

            CoinType::USDT => {
                format!("{TEST_TOKEN_PACKAGE_ID}::usdt::USDT")
            }
        };

        TypeTag::from_str(&type_str)
            .map_err(|e| anyhow::anyhow!("Invalid type tag for {:?}: {}", self, e))
    }

    pub fn package_id(&self) -> &str {
        match self {
            CoinType::SUI => "0x2",
            CoinType::WAL => TEST_TOKEN_PACKAGE_ID,
            CoinType::USDC => TEST_TOKEN_PACKAGE_ID,
            CoinType::USDT => TEST_TOKEN_PACKAGE_ID,
        }
    }

    pub fn from_u8(coin_type: u8) -> Result<Self> {
        match coin_type {
            0 => Ok(CoinType::SUI),
            1 => Ok(CoinType::WAL),
            2 => Ok(CoinType::USDC),
            3 => Ok(CoinType::USDT),
            _ => Err(anyhow::anyhow!(
                "Unknown coin type: {}. Supported: SUI, WAL, USDC, USDT",
                coin_type
            )),
        }
    }

    pub fn to_u8(&self) -> Result<u8> {
        match self {
            Self::SUI => Ok(0),
            Self::WAL => Ok(1),
            Self::USDC => Ok(2),
            Self::USDT => Ok(3),
        }
    }

    pub fn u8_to_typetag(coin_type: u8) -> Result<TypeTag> {
        let c_type = CoinType::from_u8(coin_type)?;
        c_type.to_type_tag()
    }

    pub fn name(&self) -> &str {
        match self {
            CoinType::SUI => "SUI",
            CoinType::WAL => "WAL",
            CoinType::USDC => "USDC",
            CoinType::USDT => "USDT",
        }
    }

    pub fn symbol(&self) -> &str {
        self.name()
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "SUI" => Ok(CoinType::SUI),
            "WAL" => Ok(CoinType::WAL),
            "USDC" => Ok(CoinType::USDC),
            "USDT" => Ok(CoinType::USDT),
            _ => Err(anyhow::anyhow!(
                "Unknown coin type: {}. Supported: SUI, WAL, USDC, USDT",
                s
            )),
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            CoinType::SUI => 9,  // SUI has 9 decimals (1 SUI = 1,000,000,000 MIST)
            CoinType::WAL => 9,  // WAL has 9 decimals (same as SUI)
            CoinType::USDC => 6, // USDC has 6 decimals (standard for stablecoins)
            CoinType::USDT => 6, // USDT has 6 decimals (standard for stablecoins)
        }
    }

    /// Convert human-readable amount to smallest unit
    /// Example: 10.5 SUI -> 10,500,000,000 MIST
    pub fn to_smallest_unit(&self, amount: f64) -> u64 {
        let decimals = self.decimals();
        (amount * 10_f64.powi(decimals as i32)) as u64
    }

    /// Convert smallest unit to human-readable amount
    /// Example: 10,500,000,000 MIST -> 10.5 SUI
    pub fn from_smallest_unit(&self, amount: u64) -> f64 {
        let decimals = self.decimals();
        amount as f64 / 10_f64.powi(decimals as i32)
    }

    /// Format amount with proper decimals
    pub fn format_amount(&self, amount: u64) -> String {
        format!("{} {}", self.from_smallest_unit(amount), self.symbol())
    }

    /// Get all supported coin types
    pub fn all() -> Vec<CoinType> {
        vec![CoinType::SUI, CoinType::WAL, CoinType::USDC, CoinType::USDT]
    }
}

impl std::fmt::Display for CoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
