use axum::http::HeaderValue;

pub static ROOT_OG_HTML: &str = r##"
<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<meta property="og:url" content="https://oshi.killbasa.com" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="Oshi API" />
	<meta property="og:description" content="Public API for VTubers that I watch" />
	<meta name="theme-color" content="#739fff" />
	<title>Oshi API</title>
</head>
</html>
"##;

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

pub fn is_preview_bot(user_agent: Option<&HeaderValue>) -> bool {
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

    if user_agent.eq("Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)") {
        return true;
    }

    tracing::debug!("user agent: {}", user_agent);

    false
}
