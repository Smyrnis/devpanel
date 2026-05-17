pub fn format_duration(total: u64) -> String {
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn format_unix_day(secs: u64) -> String {
    let days = secs / 86_400;
    let (year, day_of_year) = year_and_day(days);
    let (month, day) = month_day(year, day_of_year);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn year_and_day(mut days: u64) -> (i32, u64) {
    let mut year = 1970;
    loop {
        let year_days = if is_leap(year) { 366 } else { 365 };
        if days < year_days {
            return (year, days);
        }
        days -= year_days;
        year += 1;
    }
}

fn month_day(year: i32, mut day_of_year: u64) -> (u64, u64) {
    let mut days_per_month = [31_u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if is_leap(year) {
        days_per_month[1] = 29;
    }
    for (idx, month_days) in days_per_month.iter().enumerate() {
        if day_of_year < *month_days {
            return ((idx + 1) as u64, day_of_year + 1);
        }
        day_of_year -= *month_days;
    }
    (12, 31)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
