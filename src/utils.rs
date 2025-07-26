use axum_extra::headers::UserAgent;

pub fn is_term(user_agent: &UserAgent) -> bool {
    if user_agent.to_string().starts_with("curl") {
        return true;
    }

    if user_agent.to_string().starts_with("Wget") {
        return true;
    }

    tracing::debug!("user agent: {}", user_agent);

    false
}
