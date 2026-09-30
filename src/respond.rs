use serde_json::Value;
use vercel_runtime::{Error, Response, ResponseBody};

pub fn json_response(status: u16, value: &Value) -> Result<Response<ResponseBody>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(ResponseBody::from(value.to_string()))?)
}
