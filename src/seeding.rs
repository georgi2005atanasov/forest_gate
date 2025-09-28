// src/bin/seed_logins.rs (or src/seed_logins.rs if you prefer calling from main)
use chrono::{DateTime, Duration, Utc};
use rand::{
    distributions::{Alphanumeric, DistString},
    seq::SliceRandom,
    Rng,
};
use sqlx::QueryBuilder;
use sqlx::{PgPool, Postgres, Row, Transaction}; // Row is important for .get()

/// Public entry you can call from your main
pub async fn run(pool: &PgPool) -> Result<(), anyhow::Error> {
    // idempotency guard: if you want, skip seeding when users exist
    let existing: (i64,) = sqlx::query_as("SELECT COALESCE(COUNT(*),0) FROM users")
        .fetch_one(pool)
        .await?;
    if existing.0 >= 1500 {
        tracing::info!("Seed skipped: users >= 1500 already present.");
        return Ok(());
    }

    // 1) create users
    let mut tx = pool.begin().await?;
    let seeded_user_ids = insert_users(&mut tx, 1500).await?;
    tx.commit().await?;

    // 2) create login attempts (5,000 total, ~10% anomalies)
    let mut tx = pool.begin().await?;
    insert_login_attempts(&mut tx, &seeded_user_ids, 5000, 0.10).await?;
    tx.commit().await?;

    tracing::info!("Seed complete: 1500 users, 5000 login_attempts (10% anomalies).");
    Ok(())
}

/* ---------------------- User seeding ---------------------- */

#[derive(Clone)]
struct CityProfile {
    country: &'static str,
    city: &'static str,
    // Center point; we’ll add a small jitter.
    lat: f64,
    lon: f64,
    // A few common ASNs for the region
    asns: &'static [&'static str],
    // IPv4 /16 bases typical for local ISPs (fake but plausible subnets)
    ipv4_bases: &'static [(u8, u8)],
}

// A small set of realistic clusters; many users will belong to one "home" cluster.
fn profiles() -> Vec<CityProfile> {
    vec![
        CityProfile {
            country: "Bulgaria",
            city: "Sofia",
            lat: 42.6977,
            lon: 23.3219,
            asns: &["AS8866 Vivacom", "AS13124 A1 Bulgaria", "AS9070 Telenor BG"],
            ipv4_bases: &[(95, 87), (185, 80), (212, 5)],
        },
        CityProfile {
            country: "Bulgaria",
            city: "Plovdiv",
            lat: 42.1354,
            lon: 24.7453,
            asns: &["AS13124 A1 Bulgaria", "AS8866 Vivacom"],
            ipv4_bases: &[(95, 43), (46, 10)],
        },
        CityProfile {
            country: "United States",
            city: "New York",
            lat: 40.7128,
            lon: -74.0060,
            asns: &["AS7018 AT&T", "AS7922 Comcast", "AS21928 T-Mobile"],
            ipv4_bases: &[(73, 114), (67, 84), (96, 224)],
        },
        CityProfile {
            country: "Germany",
            city: "Berlin",
            lat: 52.5200,
            lon: 13.4050,
            asns: &[
                "AS3320 Deutsche Telekom",
                "AS3209 Vodafone DE",
                "AS6805 Telefonica DE",
            ],
            ipv4_bases: &[(91, 0), (93, 216), (2, 204)],
        },
        CityProfile {
            country: "United Kingdom",
            city: "London",
            lat: 51.5074,
            lon: -0.1278,
            asns: &["AS5089 Virgin Media", "AS2856 BT", "AS5607 Sky UK"],
            ipv4_bases: &[(82, 19), (86, 0), (90, 214)],
        },
    ]
}

// Ranges with datacenter/Tor-like ASNs & unusual geos for anomalies
fn anomaly_profiles() -> Vec<CityProfile> {
    vec![
        CityProfile {
            country: "Netherlands",
            city: "Amsterdam",
            lat: 52.3676,
            lon: 4.9041,
            asns: &["AS9009 M247", "AS16276 OVH", "AS14061 DigitalOcean"],
            ipv4_bases: &[(178, 239), (51, 38), (159, 65)],
        },
        CityProfile {
            country: "Seychelles",
            city: "Victoria",
            lat: -4.6167,
            lon: 55.4500,
            asns: &["AS138915 Cloud9", "AS60068 Datacamp"],
            ipv4_bases: &[(196, 196), (185, 229)],
        },
        CityProfile {
            country: "Iceland",
            city: "Reykjavik",
            lat: 64.1466,
            lon: -21.9426,
            asns: &["AS202425 1984", "AS50613 Arktur"],
            ipv4_bases: &[(93, 95), (82, 221)],
        },
        CityProfile {
            country: "Hong Kong",
            city: "Hong Kong",
            lat: 22.3193,
            lon: 114.1694,
            asns: &["AS13335 Cloudflare", "AS45102 Alibaba"],
            ipv4_bases: &[(172, 69), (47, 52)],
        },
        CityProfile {
            country: "Unknown",
            city: "Tor Exit",
            lat: 0.0,
            lon: 0.0,
            asns: &["AS200130 Tor-Exit", "AS56630 Tor-Exit"],
            ipv4_bases: &[(185, 220), (185, 220)], // fake / plausible
        },
    ]
}

async fn insert_users(
    tx: &mut Transaction<'_, Postgres>,
    count: usize,
) -> Result<Vec<(i64, usize)>, anyhow::Error> {
    let first_names = &[
        "Ivan", "Georgi", "Dimitar", "Teodora", "Maria", "John", "Emily", "Oliver", "Sophia",
        "Liam", "Emma", "Noah", "Mia", "Lucas", "Amelia", "Ben", "Lena", "Mark", "Sara", "Nikolay",
    ];
    let last_names = &[
        "Petrov", "Ivanov", "Dimitrov", "Kamenov", "Georgiev", "Smith", "Johnson", "Brown",
        "Taylor", "Miller", "Anderson", "Wilson", "Thompson", "Moore", "Clark", "Walker", "Young",
        "King", "Wright", "Scott",
    ];
    let domains = &[
        "gmail.com",
        "outlook.com",
        "yahoo.com",
        "abv.bg",
        "mail.bg",
        "proton.me",
    ];

    let profs = profiles();
    let mut rng = rand::thread_rng();

    let mut result: Vec<(i64, usize)> = Vec::with_capacity(count);
    const BATCH: usize = 300;

    for chunk_start in (0..count).step_by(BATCH) {
        let chunk_end = (chunk_start + BATCH).min(count);

        // Prepare rows in memory first
        struct RowData {
            username: String,
            email: String,
            phone: Option<String>,
            password_hash: String,
            salt: Vec<u8>,
            login_method: String,
            home_idx: usize,
        }
        let mut rows_local: Vec<RowData> = Vec::with_capacity(chunk_end - chunk_start);

        for _ in chunk_start..chunk_end {
            let f = first_names.choose(&mut rng).unwrap();
            let l = last_names.choose(&mut rng).unwrap();
            let num: u16 = rng.gen_range(1..=9999);
            let username = format!("{}.{}.{}", f.to_lowercase(), l.to_lowercase(), num);
            let email = format!(
                "{}@{}",
                username.replace('.', ""),
                domains.choose(&mut rng).unwrap()
            );
            let phone = if rng.gen_bool(0.55) {
                Some(match rng.gen_range(0..3) {
                    0 => format!(
                        "+3598{}{}{}{}{}",
                        rng.gen_range(7..10),
                        rng.gen_range(0..10),
                        rng.gen_range(0..10),
                        rng.gen_range(0..10),
                        rng.gen_range(0..10)
                    ),
                    1 => format!("+49711{}", rng.gen_range(100000..999999)),
                    _ => format!("+44{}", rng.gen_range(7000000000u64..7999999999u64)),
                })
            } else {
                None
            };

            let salt: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
            let rand_str = Alphanumeric.sample_string(&mut rng, 32);
            let password_hash = format!("$argon2id$v=19$m=65536,t=3,p=1${}", rand_str);
            let login_method = *["password", "oauth_google", "oauth_facebook", "magic_link"]
                .choose(&mut rng)
                .unwrap();

            let home_idx = rng.gen_range(0..profs.len());

            rows_local.push(RowData {
                username,
                email,
                phone,
                password_hash,
                salt,
                login_method: login_method.to_string(),
                home_idx,
            });
        }

        // Use QueryBuilder to bind all values
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO users (username, email, phone_number, password_hash, salt, login_method) ",
        );
        qb.push_values(&rows_local, |mut b, r| {
            b.push_bind(&r.username)
                .push_bind(&r.email)
                .push_bind(&r.phone)
                .push_bind(&r.password_hash)
                .push_bind(&r.salt)
                .push_bind(&r.login_method);
        });
        qb.push(" RETURNING id");
        let rows = qb.build().fetch_all(tx.as_mut()).await?;
        // align with rows_local for home_idx
        for (row, r) in rows.into_iter().zip(rows_local.into_iter()) {
            let id: i64 = row.get(0); // single-column RETURNING id
            result.push((id, r.home_idx));
        }
    }

    Ok(result)
}

// thread-local storage for home profiles aligned with users[] order
use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr};
thread_local! {
    static HOME_PROFILE: RefCell<Option<Vec<usize>>> = RefCell::new(None);
}

/* ---------------------- Login attempts seeding ---------------------- */

async fn insert_login_attempts(
    tx: &mut Transaction<'_, Postgres>,
    users_with_home: &[(i64, usize)], // (user_id, home_profile_idx)
    total: usize,
    anomaly_ratio: f32,
) -> Result<(), anyhow::Error> {
    let normal_profiles = profiles();
    let bad_profiles = anomaly_profiles();

    let anomalies = (total as f32 * anomaly_ratio).round() as usize;
    let normals = total - anomalies;

    let mut rng = rand::thread_rng();
    let now = Utc::now();

    #[derive(Clone)]
    struct AttemptRow {
        user_id: Option<i64>,
        success: bool,
        ip: IpAddr,
        country: String,
        city: String,
        asn: String,
        latitude: f64,
        longitude: f64,
        created_at: DateTime<Utc>,
    }

    let mut rows: Vec<AttemptRow> = Vec::with_capacity(total);

    // normals
    for _ in 0..normals {
        let (user_id, home_idx) = users_with_home[weighted_index(users_with_home.len(), &mut rng)];
        let home = &normal_profiles[home_idx];

        let (ip, asn) = random_ip_and_asn(home, &mut rng);
        let (lat, lon) = jitter_geo(home.lat, home.lon, 0.02, &mut rng);
        let (ts, success) = realistic_time_and_result(&now, &mut rng, false);

        rows.push(AttemptRow {
            user_id: Some(user_id),
            success,
            ip,
            country: home.country.to_string(),
            city: home.city.to_string(),
            asn: asn.to_string(),
            latitude: lat,
            longitude: lon,
            created_at: ts,
        });
    }

    // anomalies
    for _ in 0..anomalies {
        let (user_id, _) = users_with_home[rng.gen_range(0..users_with_home.len())];
        let bad = bad_profiles.choose(&mut rng).unwrap();

        let (ip, asn) = random_ip_and_asn(bad, &mut rng);
        let (lat, lon) = jitter_geo(bad.lat, bad.lon, 0.2, &mut rng);
        let (ts, success) = realistic_time_and_result(&now, &mut rng, true);

        rows.push(AttemptRow {
            user_id: Some(user_id),
            success,
            ip,
            country: bad.country.to_string(),
            city: bad.city.to_string(),
            asn: asn.to_string(),
            latitude: lat,
            longitude: lon,
            created_at: ts,
        });
    }

    rows.shuffle(&mut rng);

    // batch insert with QueryBuilder
    const BATCH: usize = 500;
    for chunk in rows.chunks(BATCH) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO login_attempts \
     (user_id, success, ip_address, country, city, asn, latitude, longitude, created_at) ",
        );
        qb.push_values(chunk, |mut b, r| {
            b.push_bind(r.user_id)
                .push_bind(r.success)
                .push_bind(&r.ip)
                .push_bind(&r.country)
                .push_bind(&r.city)
                .push_bind(&r.asn)
                .push_bind(r.latitude)
                .push_bind(r.longitude)
                .push_bind(r.created_at);
        });
        qb.build().execute(tx.as_mut()).await?;
    }

    Ok(())
}

struct LoginAttemptRow {
    user_id: Option<i64>,
    success: bool,
    ip: String,
    country: String,
    city: String,
    asn: String,
    latitude: f64,
    longitude: f64,
    created_at: DateTime<Utc>,
}

/* ---------------------- Helpers ---------------------- */

fn random_ip_and_asn(p: &CityProfile, rng: &mut impl Rng) -> (IpAddr, &'static str) {
    let (a, b) = *p.ipv4_bases.choose(rng).unwrap();
    let c: u8 = rng.gen();
    let d: u8 = rng.gen();
    (
        IpAddr::V4(Ipv4Addr::new(a, b, c, d)),
        *p.asns.choose(rng).unwrap(),
    )
}

fn jitter_geo(lat: f64, lon: f64, r: f64, rng: &mut impl Rng) -> (f64, f64) {
    // small random offset (degrees). r ~ 0.02 ~ ~2km; for anomalies r ~ 0.2
    let dlat: f64 = rng.gen_range(-r..r);
    let dlon: f64 = rng.gen_range(-r..r);
    (round6(lat + dlat), round6(lon + dlon))
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

fn realistic_time_and_result(
    now: &DateTime<Utc>,
    rng: &mut impl Rng,
    anomaly: bool,
) -> (DateTime<Utc>, bool) {
    // pick a time in last 60 days
    let days_back: i64 = rng.gen_range(0..60);
    // normals: more logins between 07:00–22:00; anomalies often at 00:00–05:00
    let hour = if anomaly {
        // 60% in 00–05
        if rng.gen_bool(0.6) {
            rng.gen_range(0..6)
        } else {
            rng.gen_range(6..24)
        }
    } else {
        // 70% in 08–20
        if rng.gen_bool(0.7) {
            rng.gen_range(8..21)
        } else {
            rng.gen_range(0..24)
        }
    };
    let minute: i64 = rng.gen_range(0..60);
    let second: i64 = rng.gen_range(0..60);

    let ts = *now - Duration::days(days_back) - Duration::hours(rng.gen_range(0..3))
        // we craft an approximate hour/min/sec by subtracting from day start
        + Duration::hours(hour as i64)
        + Duration::minutes(minute)
        + Duration::seconds(second);

    // success rate
    let success = if anomaly {
        rng.gen_bool(0.15) // many anomaly attempts fail
    } else {
        rng.gen_bool(0.75) // normals mostly succeed
    };

    (ts, success)
}

/// Weighted user pick so a few users are much more active (Zipf-like feel)
fn weighted_index(n: usize, rng: &mut impl Rng) -> usize {
    // draw from quadratic bias: small indices more likely
    let x: f64 = rng.gen::<f64>();
    let idx = (x * x * (n as f64)) as usize;
    idx.min(n - 1)
}

/* ---------------------- Minimal test main (optional) ---------------------- */
// If you want to run as a standalone binary, uncomment this and set DATABASE_URL in .env.
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenvy::dotenv().ok();
//     let database_url = std::env::var("DATABASE_URL")?;
//     let pool = PgPoolOptions::new()
//         .max_connections(1)
//         .acquire_timeout(std::time::Duration::from_secs(30))
//         .connect(&database_url)
//         .await?;
//     run(&pool).await?;
//     Ok(())
// }
