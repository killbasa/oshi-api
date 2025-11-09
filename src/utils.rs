use axum::http::HeaderValue;

pub fn is_term(user_agent: Option<&HeaderValue>) -> bool {
    let user_agent = match user_agent {
        Some(ua) => match ua.to_str() {
            Ok(s) => s,
            Err(_) => {
                tracing::debug!("invalid user agent header");
                return false;
            }
        },
        None => {
            tracing::debug!("no user agent provided");
            return false;
        }
    };

    if user_agent.contains("curl") {
        return true;
    }

    if user_agent.starts_with("Wget") {
        return true;
    }

    tracing::debug!("user agent: {}", user_agent);

    false
}
