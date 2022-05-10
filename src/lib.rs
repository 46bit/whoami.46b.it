use worker::*;
use chrono::offset::Local;

mod utils;
mod datapoint;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    utils::log_request(&req);
    utils::set_panic_hook();

    let config_bytes = include_bytes!("../config.json");
    let config: utils::Config = serde_json::from_slice(config_bytes).unwrap();

    Router::with_data(config)
        .get_async("/", |req, ctx| async move {
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

            let cf = req.cf();
            let datapoint = datapoint::Datapoint {
                index_id: "whoami.46b.it".to_string(),
                sample_interval: 1,
                timestamp: Local::now().to_rfc3339(),
                strings: vec![
                    match req.headers().get("CF-Connecting-IP") {
                        Ok(Some(ip)) => ip,
                        _ => "Unknown".to_string(),
                    },
                    cf.colo(),
                    cf.city().unwrap_or_else(|| "Unknown".to_string()),
                    cf.country().unwrap_or_else(|| "Unknown".to_string()),
                    cf.continent().unwrap_or_else(|| "Unknown".to_string()),
                    cf.http_protocol(),
                    cf.tls_cipher(),
                    cf.tls_version(),
                ],
                doubles: vec![
                    cf.asn() as f64,
                    cf.coordinates().map(|c| c.0).unwrap_or(0.0) as f64,
                    cf.coordinates().map(|c| c.1).unwrap_or(0.0) as f64,
                ],
            };
            match datapoint.write(&ctx.data.datapoint_endpoint).await {
                Ok(_) => html.push_str("<p>datapoint written successfully</p>"),
                Err(e) => {
                    console_log!("error writing datapoint: {}", e);
                    html.push_str("<p>error filing datapoint</p>");
                }
            };

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
