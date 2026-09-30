//! DEMO of GET /api/v1/norms/status?number=<law number> -- free, no key.
//! Real endpoint: leyreal-system/api/status.rs (private).
use leyreal_api_showcase::{norms::demo_status, params::query_params, respond::json_response};
use vercel_runtime::{Error, Request, Response, ResponseBody, run, service_fn};

pub async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    let params = query_params(&req);
    if params
        .get("number")
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return json_response(
            400,
            &serde_json::json!({ "error": "Missing required query parameter: number" }),
        );
    }
    json_response(200, &demo_status())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
