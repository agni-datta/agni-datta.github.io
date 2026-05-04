//! Privacy-preserving route classification and Cloudflare D1 aggregation.

/// The only page categories that analytics may persist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteBucket {
    Home,
    Publications,
    Notes,
    Resources,
    Miscellany,
    Privacy,
    NotFound,
}

impl RouteBucket {
    /// Classifies HTML paths and excludes non-page requests from analytics.
    #[must_use]
    pub fn classify(path: &str) -> Option<Self> {
        let normalized = path.trim_end_matches('/');
        match normalized {
            "" => Some(Self::Home),
            "/publications" => Some(Self::Publications),
            "/notes" => Some(Self::Notes),
            "/resources" => Some(Self::Resources),
            "/miscellany" => Some(Self::Miscellany),
            "/privacy" => Some(Self::Privacy),
            "/404.html" => Some(Self::NotFound),
            _ if is_ignored_path(normalized) => None,
            _ => Some(Self::NotFound),
        }
    }

    /// Returns the stable value stored in D1.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Publications => "publications",
            Self::Notes => "notes",
            Self::Resources => "resources",
            Self::Miscellany => "miscellany",
            Self::Privacy => "privacy",
            Self::NotFound => "not-found",
        }
    }
}

fn is_ignored_path(path: &str) -> bool {
    path.starts_with("/assets/")
        || path == "/robots.txt"
        || path == "/sitemap.xml"
        || path == "/favicon.ico"
        || path.to_ascii_lowercase().ends_with(".pdf")
}

/// Coarse location codes normalized to a bounded storage representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoCodes {
    /// Uppercase country code or the literal `unknown`.
    pub country_code: String,
    /// Uppercase region code or the literal `unknown`.
    pub region_code: String,
}

impl GeoCodes {
    /// Normalizes missing or malformed edge-provided values to `unknown`.
    #[must_use]
    pub fn normalize(country_code: Option<&str>, region_code: Option<&str>) -> Self {
        Self {
            country_code: normalize_code(country_code, 3),
            region_code: normalize_code(region_code, 16),
        }
    }
}

fn normalize_code(value: Option<&str>, maximum_length: usize) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= maximum_length)
        .filter(|value| {
            value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Returns whether the response qualifies as one HTML page view.
#[must_use]
pub fn is_countable_page(method: &str, status: u16, content_type: Option<&str>) -> bool {
    method == "GET"
        && (200..300).contains(&status)
        && content_type.is_some_and(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/html"))
        })
}

/// Parameterized atomic D1 statement used for every aggregate increment.
pub const UPSERT_SQL: &str = "INSERT INTO location_page_views (day, route, country_code, region_code, views, updated_at) VALUES (?1, ?2, ?3, ?4, 1, datetime('now')) ON CONFLICT(day, route, country_code, region_code) DO UPDATE SET views = views + 1, updated_at = datetime('now')";

/// Scheduled deletion statement that fixes retention to 90 days.
pub const RETENTION_SQL: &str =
    "DELETE FROM location_page_views WHERE day < date('now', '-90 days')";

/// Returns whether a row age belongs strictly beyond the 90-day retention window.
#[must_use]
pub const fn exceeds_retention(age_in_days: u16) -> bool {
    age_in_days > 90
}

#[cfg(target_arch = "wasm32")]
mod edge {
    use super::{GeoCodes, RETENTION_SQL, RouteBucket, UPSERT_SQL, is_countable_page};
    use worker::{
        Context, Env, Fetch, Method, Request, Response, Result, ScheduleContext, ScheduledEvent,
    };
    use worker::{console_error, event};

    #[event(fetch)]
    pub async fn fetch(request: Request, env: Env, context: Context) -> Result<Response> {
        let method = request.method();
        let path = request.path();
        let route = RouteBucket::classify(&path);
        let edge = request.cf();
        let country = edge.and_then(worker::Cf::country);
        let region = edge.and_then(worker::Cf::region_code);
        let geo = GeoCodes::normalize(country.as_deref(), region.as_deref());

        let response = Fetch::Request(request).send().await?;
        let content_type = response.headers().get("content-type").ok().flatten();
        if method == Method::Get
            && is_countable_page("GET", response.status_code(), content_type.as_deref())
            && let Some(route) = route
        {
            let day = js_sys::Date::new_0()
                .to_iso_string()
                .as_string()
                .unwrap_or_default()
                .chars()
                .take(10)
                .collect::<String>();
            let route = route.as_str().to_owned();
            if let Ok(database) = env.d1("ANALYTICS_DB") {
                context.wait_until(async move {
                    match database.prepare(UPSERT_SQL).bind(&[
                        day.into(),
                        route.into(),
                        geo.country_code.into(),
                        geo.region_code.into(),
                    ]) {
                        Ok(statement) => {
                            if let Err(error) = statement.run().await {
                                console_error!("aggregate increment failed: {error}");
                            }
                        }
                        Err(error) => console_error!("aggregate statement failed: {error}"),
                    }
                });
            }
        }
        Ok(response)
    }

    #[event(scheduled)]
    pub async fn scheduled(_event: ScheduledEvent, env: Env, _context: ScheduleContext) {
        if let Ok(database) = env.d1("ANALYTICS_DB")
            && let Err(error) = database.prepare(RETENTION_SQL).run().await
        {
            console_error!("aggregate retention failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GeoCodes, RETENTION_SQL, RouteBucket, UPSERT_SQL, exceeds_retention, is_countable_page,
    };

    #[test]
    fn route_buckets_are_bounded_and_assets_are_ignored() {
        assert_eq!(RouteBucket::classify("/"), Some(RouteBucket::Home));
        assert_eq!(RouteBucket::classify("/notes/"), Some(RouteBucket::Notes));
        assert_eq!(
            RouteBucket::classify("/new-page/"),
            Some(RouteBucket::NotFound)
        );
        assert_eq!(RouteBucket::classify("/assets/site.css"), None);
        assert_eq!(RouteBucket::classify("/paper.pdf"), None);
        assert_eq!(RouteBucket::classify("/robots.txt"), None);
    }

    #[test]
    fn geolocation_normalizes_missing_and_malformed_values() {
        assert_eq!(
            GeoCodes::normalize(Some("in"), Some(" wb ")),
            GeoCodes {
                country_code: "IN".to_owned(),
                region_code: "WB".to_owned(),
            }
        );
        assert_eq!(GeoCodes::normalize(None, Some("")).country_code, "unknown");
        assert_eq!(
            GeoCodes::normalize(Some("too-long"), Some("bad/value")).region_code,
            "unknown"
        );
    }

    #[test]
    fn only_successful_html_gets_are_counted() {
        assert!(is_countable_page(
            "GET",
            200,
            Some("text/html; charset=utf-8")
        ));
        assert!(!is_countable_page("POST", 200, Some("text/html")));
        assert!(!is_countable_page("GET", 404, Some("text/html")));
        assert!(!is_countable_page("GET", 200, Some("application/pdf")));
    }

    #[test]
    fn d1_mutations_are_parameterized_and_atomic() {
        for marker in ["?1", "?2", "?3", "?4", "ON CONFLICT", "views = views + 1"] {
            assert!(UPSERT_SQL.contains(marker));
        }
        assert!(RETENTION_SQL.contains("'-90 days'"));
    }

    #[test]
    fn retention_keeps_the_boundary_day_only() {
        assert!(!exceeds_retention(90));
        assert!(exceeds_retention(91));
    }
}
