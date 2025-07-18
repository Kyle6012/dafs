use axum::{
    Router,
    response::{Html, IntoResponse, Response},
    http::{StatusCode, Uri},
};
use tower_http::services::ServeDir;
use axum::routing::get_service;
use std::path::PathBuf;
use std::fs;
use std::net::SocketAddr;

pub async fn run_web_server() -> anyhow::Result<()> {
    // Try multiple possible web assets paths
    let web_assets_paths = [
        "target/web-assets",                    // Development build
        "/usr/share/dafs/web-assets",          // Installed package
        "web-assets",                          // Relative path
    ];
    let mut web_assets_path = None;
    for path in &web_assets_paths {
        if std::path::Path::new(path).exists() {
            web_assets_path = Some(path.to_string());
            break;
        }
    }
    let web_assets_path = web_assets_path.unwrap_or_else(|| {
        println!("⚠️  Warning: Web assets not found, using fallback page");
        "".to_string()
    });
    let serve_dir = get_service(ServeDir::new(format!("{}/assets", web_assets_path)))
        .handle_error(|error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {error}"),
            )
        });
    let app = Router::new()
        .route_service("/assets/*path", serve_dir)
        .fallback(handle_spa);
    let addr: SocketAddr = "127.0.0.1:3093".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🌐 Web dashboard server listening on http://{}", addr);
    let std_listener = listener.into_std()?;
    axum::Server::from_tcp(std_listener)?.serve(app.into_make_service()).await?;
    Ok(())
}

async fn handle_spa(uri: Uri) -> Response {
    let path = uri.path();
    // Try multiple possible web assets paths
    let web_assets_paths = [
        PathBuf::from("target/web-assets"),           // Development build
        PathBuf::from("/usr/share/dafs/web-assets"), // Installed package
        PathBuf::from("web-assets"),                 // Relative path
    ];
    let mut requested_path = None;
    for web_assets_path in &web_assets_paths {
        let path_to_try = web_assets_path.join(path.trim_start_matches('/'));
        if path_to_try.exists() && path_to_try.is_file() {
            requested_path = Some(path_to_try);
            break;
        }
    }
    if let Some(requested_path) = requested_path {
        match fs::read_to_string(&requested_path) {
            Ok(content) => {
                let content_type = get_content_type(&requested_path);
                (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, content_type)], content).into_response()
            }
            Err(_) => serve_index_html(),
        }
    } else {
        serve_index_html()
    }
}

fn serve_index_html() -> Response {
    // Try multiple possible index.html paths
    let index_paths = [
        PathBuf::from("target/web-assets/index.html"),           // Development build
        PathBuf::from("/usr/share/dafs/web-assets/index.html"), // Installed package
        PathBuf::from("web-assets/index.html"),                 // Relative path
    ];
    let mut index_content = None;
    for index_path in &index_paths {
        if let Ok(content) = fs::read_to_string(index_path) {
            index_content = Some(content);
            break;
        }
    }
    if let Some(content) = index_content {
        Html(content).into_response()
    } else {
        // Fallback HTML if index.html doesn't exist
        let fallback_html = r#"<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>DAFS</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 40px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            text-align: center;
            max-width: 600px;
        }
        h1 {
            font-size: 3rem;
            margin-bottom: 1rem;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        p {
            font-size: 1.2rem;
            margin-bottom: 2rem;
            opacity: 0.9;
        }
        .status {
            background: rgba(255,255,255,0.1);
            padding: 20px;
            border-radius: 10px;
            backdrop-filter: blur(10px);
            margin: 20px 0;
        }
        .endpoints {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 20px;
        }
        .endpoint {
            background: rgba(255,255,255,0.1);
            padding: 15px;
            border-radius: 8px;
            backdrop-filter: blur(5px);
        }
        .endpoint h3 {
            margin: 0 0 10px 0;
            font-size: 1rem;
        }
        .endpoint a {
            color: #fff;
            text-decoration: none;
            font-weight: bold;
        }
        .endpoint a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class=\"container\">
        <h1>🚀 DAFS</h1>
        <p>Decentralized AI File System</p>
        <div class=\"status\">
            <h2>System Status</h2>
            <p>✅ Backend is running</p>
            <p>⚠️ Web dashboard assets not found</p>
            <p>Please build the web dashboard first:</p>
            <code>npm install && npm run build</code>
        </div>
        <div class=\"endpoints\">
            <div class=\"endpoint\">
                <h3>HTTP API</h3>
                <a href="http://localhost:6543\" target="_blank">http://localhost:6543</a>
            </div>
            <div class=\"endpoint\">
                <h3>gRPC API</h3>
                <span>grpc://localhost:50051</span>
            </div>
            <div class=\"endpoint\">
                <h3>P2P Network</h3>
                <span>Port 2093</span>
            </div>
        </div>
        <div style=\"margin-top: 40px; opacity: 0.7;\">
            <p>For API documentation, visit: <a href="http://localhost:6543/docs\" target="_blank">http://localhost:6543/docs</a></p>
        </div>
    </div>
</body>
</html>"#;
        Html(fallback_html.to_string()).into_response()
    }
}

fn get_content_type(path: &PathBuf) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "text/plain",
    }
} 