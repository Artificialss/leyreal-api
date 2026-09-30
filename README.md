<div align="center">
  <img src="public/leyreal-logo.svg" alt="LeyReal logo" width="120" />

  # LeyReal API
</div>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="Rust" src="https://img.shields.io/badge/rust-2024-orange.svg">
  <img alt="Status" src="https://img.shields.io/badge/status-showcase-lightgrey.svg">
  <a href="https://github.com/Artificialss/leyreal-system"><img alt="Real API" src="https://img.shields.io/badge/real%20api-leyreal--system-informational.svg"></a>
</p>

Structure, relations, classification, and full text for **99,858 Costa
Rican legal norms** — laws, executive decrees, municipal regulations,
treaties, and more — served as a REST/JSON API. Built in Rust, deployed
on [Vercel's official Rust runtime](https://vercel.com/docs/functions/runtimes/rust).

This repository is the **public architecture reference** for the API:
the real opaque-id scheme, byte for byte, and the real endpoint
contracts — MIT-licensed. It does not contain the database, the SQL
migrations, the data-ingestion pipeline, or any of the actual law corpus;
every example response below is illustrative, clearly labeled, and fixed
— this repo ships no data and talks to no database. The real
implementation is private ([leyreal-system](https://github.com/Artificialss/leyreal-system));
the corpus itself is scraped and normalized in
[leyreal-costa-rica](https://github.com/Artificialss/leyreal-costa-rica).

## Table of contents

- [Quick start](#quick-start)
- [Authentication](#authentication)
- [Endpoints](#endpoints)
  - [`GET /api/v1/norms/status`](#get-apiv1normsstatus)
  - [`GET /api/v1/norms/search`](#get-apiv1normssearch)
  - [`GET /api/v1/norms/by-number`](#get-apiv1normsby-number)
  - [`GET /api/v1/norms/by-subject`](#get-apiv1normsby-subject)
  - [`GET /api/v1/norms/:id`](#get-apiv1normsid)
- [Norm types covered](#norm-types-covered)
- [The opaque-id scheme](#the-opaque-id-scheme)
- [Architecture](#architecture)
- [Usage & access](#usage--access)
- [What's intentionally not here](#whats-intentionally-not-here)
- [License](#license)
- [About](#about)

## Quick start

```bash
# Free, no key -- checking whether a law is currently in force
curl "https://leyreal-system.vercel.app/api/v1/norms/status?number=7600"
```

```json
{
  "results": [
    {
      "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
      "number": "0000",
      "norm_type": "Leyes",
      "status": "vigente",
      "issued_date": "2000-01-01"
    }
  ],
  "note": "This endpoint is free and needs no API key -- see README."
}
```

Everything else needs a key. Don't have one yet? See
[Usage & access](#usage--access).

## Authentication

Every endpoint except `status` requires `Authorization: Bearer <key>`,
and the key must be a **paid** subscription — there is no
customer-facing free tier. A missing or non-paid key gets a `401` before
any database query runs:

```bash
curl "https://leyreal-system.vercel.app/api/v1/norms/search?q=trabajo"
```

```json
{ "error": "Missing Authorization: Bearer <api-key> header." }
```

```
HTTP/2 401
```

Keys are opaque, random tokens; only their SHA-256 hash is ever stored
server-side — the plaintext is shown once, at issuance, and never
persisted. Each key also carries its own daily/monthly request quota and
a capped relation-graph traversal depth. See
[Usage & access](#usage--access) to request one.

## Endpoints

All responses are JSON. Standard status codes throughout: `200` success,
`400` malformed request, `401` missing/invalid/non-paid key, `404` no
such norm/subject, `429` quota exceeded, `500` server- or database-side
failure.

Three of the five endpoints are paginated at a deliberately small 5
results/page with a hard ceiling of 10 pages — 50 rows max reachable per
distinct query. A scraper that wants the whole corpus has to issue many
distinctly-scoped queries to get there, not just page through one, on top
of whatever rate limiting sits in front of it at the edge.

### `GET /api/v1/norms/status`

Free, no API key. Checks whether a law is currently in force (`vigente`
vs. `derogada`) by its official number. Public by number specifically
because every other endpoint's `id` is opaque *per API key* — there's no
key to derive or decode an opaque id without one, so a truly public
endpoint has to work by plain law number instead. Capped at 5 rows, no
pagination — this path has no quota in front of it.

| Param | Type | Description |
|---|---|---|
| `number` | string | Required. The law's official number. |

```bash
curl "https://leyreal-system.vercel.app/api/v1/norms/status?number=7600"
```

```json
{
  "results": [
    {
      "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
      "number": "0000",
      "norm_type": "Leyes",
      "status": "vigente",
      "issued_date": "2000-01-01"
    }
  ]
}
```

### `GET /api/v1/norms/search`

Fuzzy title search across the corpus.

| Param | Type | Description |
|---|---|---|
| `q` | string | Required. Matches against the norm's title. |
| `page` | int | Default `1`, max `10`. |

```bash
curl -H "Authorization: Bearer YOUR_KEY" \
  "https://leyreal-system.vercel.app/api/v1/norms/search?q=trabajo&page=1"
```

```json
{
  "results": [
    {
      "id": "Jm72shQ5Tr0",
      "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
      "number": "0000",
      "norm_type": "Leyes",
      "status": "vigente",
      "issued_date": "2000-01-01"
    }
  ],
  "total": 2,
  "page": 1,
  "page_count": 1
}
```

### `GET /api/v1/norms/by-number`

Exact lookup by official number. Numbers aren't globally unique — the
real corpus has norms from different issuers legitimately sharing a
number — so this returns every match, not a single record.

| Param | Type | Description |
|---|---|---|
| `number` | string | Required. |

```bash
curl -H "Authorization: Bearer YOUR_KEY" \
  "https://leyreal-system.vercel.app/api/v1/norms/by-number?number=7600"
```

```json
{ "results": [ { "id": "Jm72shQ5Tr0", "title": "...", "number": "7600", "norm_type": "Leyes", "status": "vigente", "issued_date": "1996-05-02" } ] }
```

### `GET /api/v1/norms/by-subject`

Every norm tagged with a given subject classification, newest first.
Subjects are a closed, 48-branch taxonomy (see
[leyreal-costa-rica](https://github.com/Artificialss/leyreal-costa-rica)
for how it was built) — an unrecognized subject is a `404`, not an empty
list, so a typo in a client is never silently indistinguishable from "no
results."

| Param | Type | Description |
|---|---|---|
| `subject` | string | Required. One of the 48 taxonomy names, e.g. `Laboral`, `Ambiental`, `Tributario`. |
| `page` | int | Default `1`, max `10`. |

```bash
curl -H "Authorization: Bearer YOUR_KEY" \
  "https://leyreal-system.vercel.app/api/v1/norms/by-subject?subject=Laboral&page=1"
```

```json
{ "results": [ { "id": "2W28PlwyqKh", "title": "...", "number": "160", "norm_type": "Circulares", "status": "vigente", "issued_date": "2026-09-08" } ], "total": 4528, "page": 1, "page_count": 10 }
```

### `GET /api/v1/norms/:id`

Full detail for one norm, including article text. `:id` is the opaque
id returned by any of the endpoints above. Full article text
(`articles[].text`) is the paid product: it's only present, and
`full_text_included` is only `true`, for a paid key — every key here is
paid (see [Authentication](#authentication)), so in practice this is
always the case today; kept explicit in the response rather than assumed,
in case a lower tier is ever reintroduced.

```bash
curl -H "Authorization: Bearer YOUR_KEY" \
  "https://leyreal-system.vercel.app/api/v1/norms/Jm72shQ5Tr0"
```

```json
{
  "id": "Jm72shQ5Tr0",
  "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
  "number": "0000",
  "norm_type": "Leyes",
  "status": "vigente",
  "issued_date": "2000-01-01",
  "ente_emisor": "Asamblea Legislativa",
  "full_text_included": true,
  "articles": [
    { "number": "1", "heading": null, "text": "Texto de ejemplo." }
  ]
}
```

## Norm types covered

99,858 norms across 17 types, spanning every issuer from the national
Asamblea Legislativa down to individual municipalities:

| Type | Count |
|---|---:|
| Decreto Ejecutivo | 34,644 |
| Ley | 21,338 |
| Reglamento | 15,004 |
| Reglamento municipal | 8,509 |
| Acuerdo | 6,281 |
| Resolución | 5,814 |
| Circular | 4,309 |
| Acuerdo municipal | 1,433 |
| Directriz | 1,279 |
| Tratados Internacionales | 613 |
| Decreto TSE | 385 |
| Aviso | 109 |
| Normas internacionales sin aprobar | 90 |
| Opinión | 26 |
| Constitución Política | 15 |
| Opinión Consultiva | 5 |
| Ordenanza | 4 |
| **Total** | **99,858** |

Every norm also carries article-level structure (895,236 articles total,
each with its full reform history), a 267,710-edge relation graph
(derogations, reforms, regulations between norms), and a 48-branch
subject classification — all queryable via `by-subject` above.

## The opaque-id scheme

Clients never see a raw database id. `src/opaque_id.rs` in this repo is
the real, unmodified implementation:

1. **HKDF-SHA256** derives a per-key cipher key from the raw API key
   (never stored) + a random salt stored on the key's row.
2. An **8-round Feistel network**, keyed by that cipher key, runs as a
   bijection over the 64-bit id space — reversible with the same raw key,
   not reversible without it.
3. The result is **base62-encoded** into the token clients actually see.

No id-mapping table exists or is needed — the transform is computed, not
looked up. A database leak of the stored salt alone reveals nothing
without the client's own raw key. Run `cargo test` in this repo: the real
unit tests (round-tripping, per-key distinctness) ship here unchanged.

## Architecture

- **One binary per endpoint** (`api/search.rs`, `api/by_number.rs`,
  `api/by_subject.rs`, `api/norm.rs`, `api/status.rs`), each its own
  Vercel function, routed by `vercel.json` rewrites to clean
  `/api/v1/norms/...` URLs.
- **API-key auth**, SHA-256 hash lookup against a hashed-credential
  store — never plaintext at rest, never a JWT (a request is checked
  against the database on every call, which is what makes a key
  instantly revocable).
- **Per-key opaque ids**, not a mapping table — see above.
- **Paid-only gate**: the real auth layer rejects any key whose tier
  isn't `paid` before it reaches any endpoint logic. An internal key the
  team uses for its own systems is created the same way, just tier
  `paid` — no special-casing in the code.
- **Canary-record scrape detection**: a handful of norms in the real
  corpus are honeypots, indistinguishable in shape from a real law; any
  key or anonymous request that touches one is logged.
- **Deliberately tight pagination** (5/page, 10-page ceiling) as a
  second, independent control against bulk scraping, alongside rate
  limiting at the edge.

This is a smaller, simpler shape than [cryptoally-api](https://github.com/Artificialss/cryptoally-api)'s
single-router hexagonal-architecture pattern (one axum `Router`, one
Vercel function, `domain`/`application`/`infrastructure`/`http` layers)
— a deliberate choice for this API's smaller endpoint surface, and part
of the team's practice of shipping genuinely different architectures
across products rather than one house style everywhere.

## Usage & access

LeyReal's API is a paid product — there is no customer-facing free tier.
The one exception is `status` above, kept free because it's
public-interest information (whether a law is currently in force), not
part of the paid product.

To request a key, visit **[leyreal-web.vercel.app](https://leyreal-web.vercel.app)**
(custom domain pending). Full,
current usage terms and pricing live there — that page is the source of
truth, not this repository.

## What's intentionally not here

This repo does not include: the database schema or migrations, the
scraping/data-ingestion pipeline, any raw or processed law corpus data,
deployment credentials or environment configuration, quota/canary
enforcement against a real database, or the product's website/UI. Those
live in separate, private repositories
([leyreal-system](https://github.com/Artificialss/leyreal-system),
[leyreal-costa-rica](https://github.com/Artificialss/leyreal-costa-rica),
[leyreal-web](https://github.com/Artificialss/leyreal-web)).

## License

The source code in this repository is released under the
[MIT License](LICENSE). This covers the code only — it does not grant
any rights to LeyReal's law corpus or API service itself; see
[Usage & access](#usage--access) above for those terms.

## About

LeyReal is built by **[Artificialss](https://artificialss.ai)** — an
applied AI research and product lab building software with measurable
social benefit and a transparent ecological footprint, headquartered in
Turrialba, Costa Rica. This repository is the public architecture
reference for the paid API; the product itself — free public law lookup,
an AI legal assistant, and document analysis — lives at
**[leyreal-web.vercel.app](https://leyreal-web.vercel.app)** (custom
domain pending).

Under the hood: a Postgres database (hosted on [Neon](https://neon.tech),
Vercel's Marketplace-native serverless Postgres) holds every norm,
article, relation, and subject tag, loaded from the scraped SINALEVI
corpus by a separate Python pipeline
([leyreal-costa-rica](https://github.com/Artificialss/leyreal-costa-rica)).
The API runs on Vercel's official Rust runtime; the public web UI is a
separate Next.js deployment
([leyreal-web](https://github.com/Artificialss/leyreal-web)), split out
specifically so the UI's AI-integration traffic never competes with the
paid API's load.

Questions or an API key request? Reach out via
**[artificialss.ai](https://artificialss.ai)**.

---

<p align="center">Built by <a href="https://artificialss.ai">Artificialss</a></p>
