# 🌲 Forest Gate — Creative Rust Authentication Starter (Redis, Pub/Sub, LLM Summaries)

**Forest Gate** is a small Rust project made to **enhance creativity when building an auth service**.  
It is not a strict template. It is a **creative repo** that shows ideas and lets you try things your own way.

One special decision here: **instead of inserting user interactions into a database, we write them to simple `.md` files** in `/interactions`. I do it **because I want it that way**, and because **everyone is free to do what they want**. This repo is a **great starting point for an auth service using Redis**. Of course, it is **not complete**, but it **has the potential** to be completed and extended.

This repo also includes a **PDF file** that shows an **example of a database**. There are **many ways to set up an auth service**. This project shows one path.

---

## ✨ What this repo is
- A prototype auth service in **Rust**.
- A simple and **opinionated** approach to session events.
- A **playground** for ideas like Redis Pub/Sub and LLM summaries.

## 🧱 What this repo is not
- Not production-ready.
- Not security-audited.
- Not feature-complete.

---

## 🔗 Integrations (with links)
This repo integrates a few external services:

- **MaxMind (GeoIP/ASN)** — enrich IPs with country/city/ASN.  
  👉 https://www.maxmind.com
- **SendGrid (Email)** — send alerts and notifications.  
  👉 https://sendgrid.com
- **OpenRouter (LLMs)** — generate short behavior summaries from session events.  
  👉 https://openrouter.ai

---

## 🔑 Key ideas
- **Redis for events:** user actions are stored per session while the user is active.
- **Inactivity flush:** when a timer expires, a background task reads the events and **writes a Markdown summary** to `/interactions/{interaction_id}.md`.
- **LLM summaries:** the service uses **OpenRouter** to create a **short, fluent summary** of what the user did during the session.
- **Freedom by design:** write to Markdown now; swap to a database later if you prefer.

---

## 🚀 Quick start (local)

### 1) Start services
```bash
docker compose up
```

### 2) Enable Redis key event notifications
Enter the Redis container and run `redis-cli`:
```bash
# in another terminal
docker ps                         # find the redis container name
docker exec -it <redis-container> sh
redis-cli
CONFIG SET notify-keyspace-events Ex
```

This setting enables the **Pub/Sub mechanism** for key **expiration events**.  
It is needed so the flusher can **detect inactivity**, **read events from Redis**, and **write a summary** to `.md` files in the **`/interactions`** folder.

### 3) (Optional) Run app + database together
If you want to run the **app and the DB** together, use the script:
```bash
./scripts/app.bash
```
This script starts the needed Docker Compose files. I **separated** the compose files because I had **connection troubleshooting** to the DB (the SQL library threw exceptions). This makes debugging easier.

---

## 🔐 Keys and scripts

In the **`/scripts`** folder, you will find instructions for generating **EC keys** for **token signing**.  
Example (for reference only; use your scripts and security policies):
```bash
# Generate a private key (P-256)
openssl ecparam -name prime256v1 -genkey -noout -out ec_private.pem

# Derive the public key
openssl ec -in ec_private.pem -pubout -out ec_public.pem
```

Set your app to read these keys from paths you prefer (see env vars below).

---

## ⚙️ Environment variables

You will need some environment variables to run the app:

```
# Email (SendGrid)
SENDGRID_API_KEY=Your_SendGrid_ApiKey
FROM_EMAIL=your_email@example.com
FROM_NAME="Your Name"
REPLY_TO_EMAIL=reply_to_email@example.com
NOTIFY_EMAIL=notify_email@example.com

# Visitor HMAC
VISITOR_HMAC_KEY=32 bit HMAC key

# Auth token signing (EC keys)
AUTH_EC_PRIVATE_PEM_PATH=/path/to/ec_private.pem
AUTH_EC_PUBLIC_PEM_PATH=/path/to/ec_public.pem
AUTH_ISSUER=issuer_name
AUTH_AUDIENCE=issuer_audience

# LLM summaries (OpenRouter)
OPENROUTER_API_KEY=Your_Open_Router_ApiKey
OPENROUTER_MODEL=Some_Open_Router_Model
OPENROUTER_APP_NAME=Your_App_Name

# (Optional) MaxMind local DB path if you use GeoLite2 locally
MAXMIND_DB_PATH=/path/to/GeoLite2-City.mmdb
```

---

## 🧠 How the summaries work (short)

1. While a session is active, user events are pushed into **Redis**.
2. A **timer key** with TTL keeps track of **inactivity** (for example, 60 seconds).
3. When the timer key **expires**, Redis publishes an **expired** event (Pub/Sub).
4. The background worker (flusher) listens for this event, **reads the list of events**, and **deletes** it.
5. It asks **OpenRouter** for a **short, B1–B2 level** summary.
6. It **creates or appends** a file at **`/interactions/{interaction_id}.md`**, with a timestamp, the summary, and the raw event list.

---

## 🗺️ Philosophy

There are many ways to build auth. This project shows a **simple, creative path**:
- keep things **readable**,
- try ideas fast,
- replace parts later when you need more power.

PRs and experiments are welcome. Have fun and stay safe. ✌️
