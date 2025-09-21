# 🌲 Forest Gate — Rust Authentication with Anomaly Detection, Device Control, LLM Behavior Summaries, and Real-Time Alerts

**Forest Gate** is a **⚡ Rust authentication service** focused on **🔒 security, 👀 visibility, and 🚀 speed**.  
Admins can **🕵️ monitor user behavior**, **📱 manage devices**, **❌ revoke access**, and react to threats fast.  
The system adds **🧩 Isolation Forest anomaly detection**, **🤖 LLM behavior summaries**, **🌍 GeoIP (MaxMind)**, and **📧 SendGrid alerts**.

**Admin Frontend:** https://github.com/georgi2005atanasov/forest_gate_frontend

> ⚠️ Status: **Active development / not finished yet** (APIs and UI may change).

---

## 💡 Why Forest Gate

- **🛡 Catch risky logins** with Isolation Forest risk scoring  
- **🧠 Understand behavior** with LLM summaries built from session events  
- **🔐 Control devices** and **revoke access** in seconds  
- **🌐 Know where users log in from** (ASN, country, city via MaxMind)  
- **📨 Get instant email alerts** for changes and threats (SendGrid)  
- **⚡ Built in Rust** for **high performance** and **low latency**

---

## 🔑 Core Features

- **🔑 Authentication & Sessions**  
  Secure login, tokens, RBAC, and live session tracking.

- **⚡ Real-Time Session Stream (Redis)**  
  Each user session is recorded while the user is active.  
  Events are collected in Redis for fast reads and analysis.

- **🤖 LLM Behavior Summaries**  
  After a session ends, the event stream is processed by an LLM.  
  The system creates a short **summary of the user behavior** for review and detection.

- **📊 Anomaly Detection (Isolation Forest)**  
  Scores logins and sensitive actions with low/medium/high risk.

- **🌍 Geo & Network Intelligence (MaxMind)**  
  Enrich events with **ASN**, **country**, and **city** from IP.

- **📧 Security Alerts (SendGrid)**  
  Emails on configuration changes, high-risk activity, and blocks.

- **🖥 Admin Dashboard**  
  User monitoring, device management, risk filters, and settings.

- **📜 Audit & Compliance Ready**  
  Structured events and change history for security reviews.

---

## 🔍 SEO Highlights

- **Rust authentication service** with **Isolation Forest anomaly detection**  
- **Redis session tracking** + **LLM user behavior summaries**  
- **MaxMind GeoIP & ASN enrichment** for **impossible travel** and ISP changes  
- **SendGrid security alerts** for **risky logins** and **config changes**  
- **Device management**, **access revoke**, and **admin monitoring dashboard**  
- **High performance Rust API** for **fraud prevention** and **account takeover defense**

---

## 🔗 Integrations

- **🗄 Redis** — real-time session events during user activity  
- **🤖 LLM** — post-session behavior summarization  
- **🌍 MaxMind** — ASN, country, city from IP  
- **📧 SendGrid** — security notifications and alert templates  
- **🖥 Frontend (Admin UI)** — https://github.com/georgi2005atanasov/forest_gate_frontend

---

## 🔐 Security Posture

- 🧑‍💻 Principle of least privilege and clear admin roles  
- 🔑 Strong token handling and safe defaults  
- 🕵️ Privacy-aware event storage and retention options  
- 🌐 HTTPS by default in production

---

## 🎯 Use Cases

- Detect **account takeover** and **fraud** during sign-in  
- See **who did what, when, and from where**  
- **Revoke** suspicious sessions and **block** devices  
- Review **LLM summaries** to understand patterns fast  
- Trigger alerts on **risky actions** and **policy changes**

---

## ⚙️ Performance & Reliability

- **⚡ Rust** for throughput and predictable latency  
- 📡 Stateless scaling and clean, typed responses  
- ⚡ Fast event ingestion with **Redis**

---

## 🛠 Roadmap

- 🔐 Step-up auth (2FA / WebAuthn)  
- 📈 Advanced velocity rules and patterns  
- 📊 Export tools for audits and BI  
- 👥 Granular admin roles and permissions

---

## ❓ FAQ

**Is it production-ready?**  
Not yet. It is **under active development** and features can change.

**Can I tune the anomaly threshold?**  
Yes. The Isolation Forest threshold is configurable.

**Do I need a paid MaxMind plan?**  
You can start with GeoLite2 and upgrade later.

**Does it support admin dashboards?**  
Yes—the **Forest Gate Frontend** gives monitoring and control.
