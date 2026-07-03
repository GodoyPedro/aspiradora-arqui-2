use axum::http::header::{HeaderValue, CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};

const DEMO_HTML: &str = include_str!("assets/demo.html");
const DEMO_CSS: &str = include_str!("assets/demo.css");
const DEMO_JS: &str = include_str!("assets/demo.js");

pub async fn demo_page() -> Html<&'static str> {
    Html(DEMO_HTML)
}

pub async fn demo_css() -> Response {
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        DEMO_CSS,
    )
        .into_response()
}

pub async fn demo_js() -> Response {
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static("application/javascript; charset=utf-8"),
        )],
        DEMO_JS,
    )
        .into_response()
}
