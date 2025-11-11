use worker::{Headers, Method, Request, Response, Result};

const ALLOWED_METHODS: &str = "GET,POST,PUT,PATCH,DELETE,OPTIONS";
const ALLOWED_HEADERS: &str = "Content-Type, Authorization";

pub fn with_cors(mut response: Response) -> Result<Response> {
    apply_cors_headers(response.headers_mut())?;
    Ok(response)
}

pub fn preflight_response(_: &Request) -> Result<Response> {
    let mut response = Response::empty()?;
    let headers = response.headers_mut();
    apply_cors_headers(headers)?;
    headers.set("Access-Control-Max-Age", "86400")?;
    headers.set("Content-Length", "0")?;
    Ok(response)
}

pub fn is_preflight(request: &Request) -> bool {
    request.method() == Method::Options
}

fn apply_cors_headers(headers: &mut Headers) -> Result<()> {
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", ALLOWED_METHODS)?;
    headers.set("Access-Control-Allow-Headers", ALLOWED_HEADERS)?;
    headers.append("Vary", "Origin")?;
    headers.append("Vary", "Access-Control-Request-Method")?;
    headers.append("Vary", "Access-Control-Request-Headers")?;
    Ok(())
}
