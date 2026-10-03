//! DEMO response shapes -- fixed example data, no database, no live corpus.
//! The real API's src/norms.rs (leyreal-system, private) runs these same
//! five queries against Neon Postgres and returns per-key opaque ids.

use crate::auth::DemoKeyContext;
use crate::opaque_id;
use serde::Serialize;
use serde_json::{Value, json};

fn example_row(key: &DemoKeyContext, id: i64) -> Value {
    json!({
        "id": opaque_id::encode(&key.raw_key, &key.opaque_id_salt, id),
        "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
        "number": "0000",
        "norm_type": "Leyes",
        "status": "vigente",
        "issued_date": "2000-01-01"
    })
}

pub fn demo_paged_result(key: &DemoKeyContext) -> Value {
    json!({
        "results": [example_row(key, 1), example_row(key, 2)],
        "total": 2,
        "page": 1,
        "page_count": 1,
        "note": "Example data. The real endpoint queries ~99,922 real norms and caps pagination at 5 results x 10 pages."
    })
}

pub fn demo_by_number(key: &DemoKeyContext) -> Value {
    json!({ "results": [example_row(key, 1)] })
}

#[derive(Serialize)]
struct DemoArticle {
    number: String,
    heading: Option<String>,
    text: Option<String>,
}

pub fn demo_detail(key: &DemoKeyContext, opaque_id_param: &str) -> Value {
    let decoded = opaque_id::decode(&key.raw_key, &key.opaque_id_salt, opaque_id_param);
    json!({
        "id": opaque_id_param,
        "decoded_successfully": decoded.is_ok(),
        "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
        "number": "0000",
        "norm_type": "Leyes",
        "status": "vigente",
        "issued_date": "2000-01-01",
        "ente_emisor": "Asamblea Legislativa",
        "full_text_included": true,
        "articles": [DemoArticle {
            number: "1".into(),
            heading: None,
            text: Some("Texto de ejemplo. El artículo real se sirve solo si la clave es tier=paid.".into()),
        }],
        "note": "Example data. Try changing the opaque id: it will still decode (the Feistel network is a real bijection over the 64-bit id space), but there's no real row behind it."
    })
}

pub fn demo_status() -> Value {
    json!({
        "results": [{
            "title": "Ejemplo ilustrativo -- no es un dato real del corpus",
            "number": "0000",
            "norm_type": "Leyes",
            "status": "vigente",
            "issued_date": "2000-01-01"
        }],
        "note": "This endpoint is free and needs no API key -- see README."
    })
}
