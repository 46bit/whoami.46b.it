use worker::*;

mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::log_request(&req);
    utils::set_panic_hook();

    Router::new()
        .get("/", |req, _| {
            let mut html = String::new();
            if let Ok(Some(ip)) = req.headers().get("CF-Connecting-IP") {
                html.push_str(format!("<h1>You are <code>{}</code></h1>", ip).as_str());
            }
            if let Ok(request_cf_info) = (*req.inner().cf()).into_serde::<serde_json::Value>() {
                if let Ok(json) = serde_json::to_string_pretty(&request_cf_info) {
                    html.push_str(
                        "<p>Information about the request provided by Cloudflare's edge:</p>",
                    );
                    html.push_str(format!("<pre>{}</pre>", json).as_str());
                }
            }
            Response::from_html(html)
        })
        .get("/version", |_, _| {
            let name = env!("CARGO_PKG_NAME");
            let version = env!("CARGO_PKG_VERSION");
            Response::ok(format!("{} {}", name, version))
        })
        .run(req, env)
        .await
}
