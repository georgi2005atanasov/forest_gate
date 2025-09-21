use std::{
    net::IpAddr,
    path::{Path},
};

use maxminddb::{geoip2, Reader};

use crate::utils::error::{Error, Result};

/// Where the .mmdb files live:
/// - GeoLite2-ASN.mmdb
/// - GeoLite2-City.mmdb
/// - GeoLite2-Country.mmdb
///
/// Load order:
/// 1) env var `MAXMIND_DB_DIR` if set
/// 2) fallback to "./src/features/clients/data"
pub struct MaxMindClient {
    asn_reader: Reader<Vec<u8>>,
    city_reader: Reader<Vec<u8>>,
    country_reader: Reader<Vec<u8>>,
}

impl MaxMindClient {
    /// Load all three databases from a directory.
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();

        let asn_path = dir.join("GeoLite2-ASN.mmdb");
        let city_path = dir.join("GeoLite2-City.mmdb");
        let country_path = dir.join("GeoLite2-Country.mmdb");

        let asn_reader = Reader::open_readfile(&asn_path)
            .map_err(|e| Error::Unexpected(format!("failed to open ASN mmdb: {e}")))?;
        let city_reader = Reader::open_readfile(&city_path)
            .map_err(|e| Error::Unexpected(format!("failed to open City mmdb: {e}")))?;
        let country_reader = Reader::open_readfile(&country_path)
            .map_err(|e| Error::Unexpected(format!("failed to open Country mmdb: {e}")))?;

        Ok(Self { asn_reader, city_reader, country_reader })
    }

    /// Load from env var `MAXMIND_DB_DIR` or fallback to "./src/features/clients/data".
    pub fn from_env_or_default() -> Result<Self> {
        let dir = std::env::var("MAXMIND_DB_DIR")
            .unwrap_or_else(|_| "./src/features/clients/data".to_string());
        Self::from_dir(dir)
    }

    /// Lookup an IP (string) and return merged info from ASN + City + Country.
    pub fn lookup_all_str<'a>(&'a self, ip_str: &str) -> Result<GeoIpInfo<'a>> {
        let ip: IpAddr = ip_str
            .parse()
            .map_err(|e| Error::Validation(format!("invalid IP address: {e}")))?;
        self.lookup_all(ip)
    }

    /// Lookup an IP (IpAddr) and return merged info from ASN + City + Country.
    /// Note: returned data borrows from the readers (`self`), so it carries lifetime `'a`.
    pub fn lookup_all<'a>(&'a self, ip: IpAddr) -> Result<GeoIpInfo<'a>> {
        let asn = match self.asn_reader.lookup::<geoip2::Asn>(ip) {
            Ok(v) => v,
            Err(e) => return Err(Error::Unexpected(format!("asn lookup error: {e}"))),
        };

        let city = match self.city_reader.lookup::<geoip2::City>(ip) {
            Ok(v) => v,
            Err(e) => return Err(Error::Unexpected(format!("city lookup error: {e}"))),
        };

        let country = match self.country_reader.lookup::<geoip2::Country>(ip) {
            Ok(v) => v,
            Err(e) => return Err(Error::Unexpected(format!("country lookup error: {e}"))),
        };

        Ok(GeoIpInfo { asn, city, country })
    }
}

/// Unified response holding MaxMind’s own types.
/// These borrow from the readers; you cannot return them without tying the lifetime to `&self`.
#[derive(Debug)]
pub struct GeoIpInfo<'a> {
    pub asn: Option<geoip2::Asn<'a>>,
    pub city: Option<geoip2::City<'a>>,
    pub country: Option<geoip2::Country<'a>>,
}
