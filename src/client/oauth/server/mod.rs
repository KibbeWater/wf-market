/*!
Expose a tiny_http helper server to catch OAuth code callbacks
*/

use std::collections::HashMap;
use tiny_http::{Response, Server};
use url::Url;

pub fn start_listener_server(port: u16) -> Result<String, Box<dyn std::error::Error + Send>> {
    let server = Server::http(format!("0.0.0.0:{}", port))
        .map_err(|e| e as Box<dyn std::error::Error + Send>)?;

    for request in server.incoming_requests() {
        let url = Url::parse(&format!("http://localhost{}", request.url()))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

        let response = Response::from_string(
            "You can now close this tab"
        );
        let _ = request.respond(response);

        // Extract query parameters
        let query_params: HashMap<String, String> = url.query_pairs()
            .into_owned()
            .collect();

        if let Some(code) = query_params.get("code") {
            return Ok(code.clone());
        }
    }

    // This should never be reached in normal operation
    // but needed for completeness
    Ok(String::new())
}