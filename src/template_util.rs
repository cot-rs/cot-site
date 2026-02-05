use chrono::Datelike;

pub(crate) fn current_year() -> i32 {
    chrono::Utc::now().year()
}
