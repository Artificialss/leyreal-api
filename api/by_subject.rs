//! DEMO of GET /api/v1/norms/by-subject?subject=<classification name>&page=<n>
//! Real endpoint: leyreal-system/api/by_subject.rs (private).
use leyreal_api_showcase::{
    auth::authenticate_demo, norms::demo_paged_result, params::query_params, respond::json_response,
};
use vercel_runtime::{Error, Request, Response, ResponseBody, run, service_fn};

pub async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    let params = query_params(&req);
    if params
        .get("subject")
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return json_response(
            400,
            &serde_json::json!({ "error": "Missing required query parameter: subject" }),
        );
    }
    match authenticate_demo(&req) {
        Ok(key) => json_response(200, &demo_paged_result(&key)),
        Err(message) => json_response(401, &serde_json::json!({ "error": message })),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
