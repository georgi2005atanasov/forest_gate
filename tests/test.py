# detect_anomalies.py
# Isolation Forest for login_attempts anomaly detection

import os
import math
import ipaddress
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from sqlalchemy import create_engine, text
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import RobustScaler
from sklearn.pipeline import Pipeline
from joblib import dump

# -----------------------------
# 1) DB connection
# -----------------------------
DB_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://postgres:1234Kaima56@localhost:5432/auth"
)
engine = create_engine(DB_URL, pool_pre_ping=True)

# -----------------------------
# 2) Load data
# -----------------------------
ALL_SQL = """
SELECT
  id, user_id, success, ip_address::text AS ip_address, country, city, asn,
  latitude::float AS latitude, longitude::float AS longitude, created_at
FROM login_attempts
ORDER BY created_at ASC;
"""

VALID_SQL = """
SELECT
  id, user_id, success, ip_address::text AS ip_address, country, city, asn,
  latitude::float AS latitude, longitude::float AS longitude, created_at
FROM login_attempts
WHERE asn NOT IN (
  'AS9009 M247', 'AS16276 OVH', 'AS14061 DigitalOcean',
  'AS138915 Cloud9', 'AS60068 Datacamp',
  'AS202425 1984', 'AS50613 Arktur',
  'AS13335 Cloudflare', 'AS45102 Alibaba',
  'AS200130 Tor-Exit', 'AS56630 Tor-Exit'
)
ORDER BY created_at ASC;
"""

print("Reading data from Postgres...")
df_all = pd.read_sql(text(ALL_SQL), engine, parse_dates=["created_at"])
df_valid = pd.read_sql(text(VALID_SQL), engine, parse_dates=["created_at"])

if df_all.empty:
    raise SystemExit("Table login_attempts is empty. Add data and try again.")

if df_valid.empty:
    # If valid set is empty, train on all (not ideal, but allows demo)
    print("Warning: valid set is empty. Training on ALL data for demo.")
    df_valid = df_all.copy()

# -----------------------------
# 3) Feature engineering helpers
# -----------------------------
def ip_to_int(ip_str: str) -> float:
    """Map IPv4/IPv6 to a numeric value. Return NaN if invalid."""
    if ip_str is None or ip_str == "":
        return np.nan
    try:
        ip_obj = ipaddress.ip_address(ip_str)
        # Map to large int, then log-scale for stability
        val = int(ip_obj)
        # Avoid log(0)
        return math.log1p(val)
    except Exception:
        return np.nan

def freq_encode(series: pd.Series) -> pd.Series:
    """Frequency encode a categorical column (safe for many categories)."""
    freq = series.value_counts(dropna=True)
    # Normalize to 0..1 for scale stability
    freq = freq / (freq.max() if len(freq) else 1.0)
    return series.map(freq).fillna(0.0)

def enrich(df: pd.DataFrame) -> pd.DataFrame:
    d = df.copy()

    # Basic clean
    # Fill geo gaps with median (so model can still learn)
    for col in ["latitude", "longitude"]:
        if col not in d:
            d[col] = np.nan
        d[col] = d[col].astype(float)
        d[col] = d[col].fillna(d[col].median())

    # Time features
    d["hour"] = d["created_at"].dt.hour.astype(float)
    d["dow"]  = d["created_at"].dt.dayofweek.astype(float)  # 0=Mon
    d["month"] = d["created_at"].dt.month.astype(float)

    # Success as 0/1
    d["success_i"] = d["success"].astype(int)

    # IP numeric
    d["ip_num"] = d["ip_address"].apply(ip_to_int).fillna(0.0)

    # Frequency encodings (lightweight for demo)
    d["country_f"] = freq_encode(d["country"])
    d["city_f"]    = freq_encode(d["city"])
    d["asn_f"]     = freq_encode(d["asn"])

    # Final feature matrix (NumPy friendly)
    feature_cols = [
        "latitude", "longitude",
        "hour", "dow", "month",
        "success_i", "ip_num",
        "country_f", "city_f", "asn_f"
    ]
    d["_features"] = list(d[feature_cols].to_numpy(dtype=float))
    return d

print("Building features...")
df_all_e   = enrich(df_all)
df_valid_e = enrich(df_valid)

X_all   = np.vstack(df_all_e["_features"].to_numpy())
X_train = np.vstack(df_valid_e["_features"].to_numpy())

# -----------------------------
# 4) Train Isolation Forest
# -----------------------------
# RobustScaler helps with outliers in numeric ranges
model = Pipeline(steps=[
    ("scaler", RobustScaler()),
    ("iso", IsolationForest(
        n_estimators=300,
        max_samples='auto',
        contamination=0.02,  # let the model decide proportion of anomalies
        random_state=42,
        n_jobs=-1
    ))
])

print("Training Isolation Forest on valid data...")
model.fit(X_train)

# -----------------------------
# 5) Score all data
# -----------------------------
# IsolationForest.score_samples: higher = more normal
# We invert so that higher = more anomalous (easier to read)
raw_scores = model.named_steps["iso"].score_samples(
    model.named_steps["scaler"].transform(X_all)
)
anomaly_scores = -raw_scores

preds = model.named_steps["iso"].predict(
    model.named_steps["scaler"].transform(X_all)
)  # 1 = normal, -1 = anomaly

df_out = df_all_e.copy()
df_out["anomaly_score"] = anomaly_scores
df_out["is_anomaly"] = (preds == -1)

# -----------------------------
# 6) Show some results
# -----------------------------
print("\nTop 20 most suspicious login attempts:")
cols_to_show = [
    "id", "user_id", "created_at", "ip_address", "country", "city", "asn",
    "latitude", "longitude", "success", "anomaly_score", "is_anomaly"
]
print(df_out.sort_values("anomaly_score", ascending=False)[cols_to_show].head(20).to_string(index=False))

# Save CSV so you can inspect in VS Code
out_path = "login_anomalies_scored.csv"
df_out[cols_to_show].to_csv(out_path, index=False)
print(f"\nSaved: {out_path}")

# Save model for reuse
dump(model, "isolation_forest_login_attempts.joblib")
print("Saved: isolation_forest_login_attempts.joblib")

# -----------------------------
# 7) Simple visuals (NumPy + matplotlib)
#    Each plot uses NumPy arrays.
# -----------------------------

# (A) Histogram of anomaly scores
scores_np = anomaly_scores.astype(float)
is_anomaly_np = df_out["is_anomaly"].to_numpy()

plt.figure(figsize=(12, 6))

# Normal logins
plt.scatter(
    np.where(~is_anomaly_np)[0],
    scores_np[~is_anomaly_np],
    s=8, c="blue", alpha=0.5, label="Normal"
)

# Anomalous logins
plt.scatter(
    np.where(is_anomaly_np)[0],
    scores_np[is_anomaly_np],
    s=12, c="red", alpha=0.8, label="Anomaly"
)

plt.title("Anomaly scores per login attempt")
plt.xlabel("Login attempt index (time order)")
plt.ylabel("Anomaly score (higher = more anomalous)")
plt.legend()
plt.tight_layout()
plt.show()

# (C) Time vs anomaly score
# order by created_at (already ordered, but we ensure)
time_order = np.argsort(df_out["created_at"].to_numpy())
scores_t = scores_np[time_order]
plt.figure()
plt.plot(scores_t)
plt.title("Anomaly score over time")
plt.xlabel("Event index (time-sorted)")
plt.ylabel("Anomaly score")
plt.tight_layout()
plt.show()
