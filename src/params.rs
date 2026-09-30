use std::collections::HashMap;
use vercel_runtime::Request;

pub fn query_params(req: &Request) -> HashMap<String, String> {
    let query = req.uri().query().unwrap_or("");
    url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}
