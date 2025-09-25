// maxmind_client.rs

use crate::utils::error::{Error, Result};
use maxminddb::{geoip2, Reader};
use std::{net::IpAddr, path::Path, sync::Arc};
use tracing::{info, error}; // <-- make sure this is imported

// ---------- add this type ----------
#[derive(Debug)]
pub struct GeoIpInfo<'a> {
    pub asn: Option<geoip2::Asn<'a>>,
    pub city: Option<geoip2::City<'a>>,
    pub country: Option<geoip2::Country<'a>>,
}
// -----------------------------------

#[derive(Clone)]
pub struct MaxMindClient {
    asn_reader: Arc<Reader<Vec<u8>>>,
    city_reader: Arc<Reader<Vec<u8>>>,
    country_reader: Arc<Reader<Vec<u8>>>,
}

impl MaxMindClient {
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();

        let asn_reader = Reader::open_readfile(dir.join("GeoLite2-ASN.mmdb"))
            .map_err(|e| Error::Unexpected(format!("failed to open ASN mmdb: {e}")))?;
        let city_reader = Reader::open_readfile(dir.join("GeoLite2-City.mmdb"))
            .map_err(|e| Error::Unexpected(format!("failed to open City mmdb: {e}")))?;
        let country_reader = Reader::open_readfile(dir.join("GeoLite2-Country.mmdb"))
            .map_err(|e| Error::Unexpected(format!("failed to open Country mmdb: {e}")))?;

        Ok(Self {
            asn_reader: Arc::new(asn_reader),
            city_reader: Arc::new(city_reader),
            country_reader: Arc::new(country_reader),
        })
    }

    pub fn from_env_or_default() -> Result<Self> {
        let dir = std::env::var("MAXMIND_DB_DIR")
            .unwrap_or_else(|_| "./src/features/clients/data".to_string());
        Self::from_dir(dir)
    }

    pub fn lookup_all_str<'a>(&'a self, ip_str: &str) -> Result<GeoIpInfo<'a>> {
        let ip: IpAddr = ip_str
            .parse()
            .map_err(|e| Error::Validation(format!("invalid IP address: {e}")))?;
        self.lookup_all(ip)
    }

    pub fn lookup_all<'a>(&'a self, ip: IpAddr) -> Result<GeoIpInfo<'a>> {
        let asn = match (&*self.asn_reader).lookup::<geoip2::Asn>(ip) {
            Ok(v) => {
                info!(%ip, "ASN lookup success: {:?}", v);
                v
            }
            Err(e) => {
                error!(%ip, error = %e, "ASN lookup failed");
                return Err(Error::Unexpected(format!("asn lookup error: {e}")));
            }
        };

        let city = match (&*self.city_reader).lookup::<geoip2::City>(ip) {
            Ok(v) => {
                info!(%ip, "City lookup success");
                v
            }
            Err(e) => {
                error!(%ip, error = %e, "City lookup failed");
                return Err(Error::Unexpected(format!("city lookup error: {e}")));
            }
        };

        let country = match (&*self.country_reader).lookup::<geoip2::Country>(ip) {
            Ok(v) => {
                info!(%ip, "Country lookup success");
                v
            }
            Err(e) => {
                error!(%ip, error = %e, "Country lookup failed");
                return Err(Error::Unexpected(format!("country lookup error: {e}")));
            }
        };

        info!("GeoIP lookup complete: {:?}, {:?}, {:?}", asn, city, country);
        Ok(GeoIpInfo { asn, city, country })
    }
}
