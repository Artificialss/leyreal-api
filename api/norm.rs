//! DEMO of GET /api/v1/norms/:id
//! Real endpoint: leyreal-system/api/norm.rs (private).
use leyreal_api_showcase::{
    auth::authenticate_demo, norms::demo_detail, params::query_params, respond::json_response,
};
use vercel_runtime::{Error, Request, Response, ResponseBody, run, service_fn};

pub async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    let params = query_params(&req);
    let Some(id) = params.get("id").filter(|s| !s.trim().is_empty()) else {
        return json_response(400, &serde_json::json!({ "error": "Missing law id." }));
    };
    match authenticate_demo(&req) {
        Ok(key) => json_response(200, &demo_detail(&key, id)),
        Err(message) => json_response(401, &serde_json::json!({ "error": message })),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
