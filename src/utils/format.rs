use std::time::Duration;

/// Format a duration as MM:SS or HH:MM:SS
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Create a progress bar string
/// Example: [▓▓▓▓▓░░░░░] 2:30 / 5:00
pub fn create_progress_bar(
    current: Duration,
    total: Option<Duration>,
    bar_length: usize,
) -> String {
    let current_secs = current.as_secs_f64();
    let total_secs = match total {
        Some(duration) => duration.as_secs_f64(),
        _ => current.as_secs_f64(),
    };

    let progress = if total_secs > 0.0 {
        (current_secs / total_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let filled_length = (progress * bar_length as f64).round() as usize;
    let empty_length = bar_length.saturating_sub(filled_length);

    let filled = "▓".repeat(filled_length);
    let empty = "░".repeat(empty_length);

    let current_str = format_duration(current);
    let total_str = match total {
        Some(duration) => format_duration(duration),
        _ => format_duration(current),
    };

    format!("[{}{}] {} / {}", filled, empty, current_str, total_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        let duration = Duration::from_secs(45);
        assert_eq!(format_duration(duration), "00:45");
    }

    #[test]
    fn test_format_duration_minutes() {
        let duration = Duration::from_secs(150);
        assert_eq!(format_duration(duration), "02:30");
    }

    #[test]
    fn test_format_duration_hours() {
        let duration = Duration::from_secs(3665);
        assert_eq!(format_duration(duration), "01:01:05");
    }

    #[test]
    fn test_progress_bar_empty() {
        let current = Duration::from_secs(0);
        let total = Duration::from_secs(300);
        let bar = create_progress_bar(current, Some(total), 10);
        assert!(bar.contains("░░░░░░░░░░"));
        assert!(bar.contains("00:00 / 05:00"));
    }

    #[test]
    fn test_progress_bar_half() {
        let current = Duration::from_secs(150);
        let total = Duration::from_secs(300);
        let bar = create_progress_bar(current, Some(total), 10);
        assert!(bar.contains("▓▓▓▓▓░░░░░"));
        assert!(bar.contains("02:30 / 05:00"));
    }

    #[test]
    fn test_progress_bar_full() {
        let current = Duration::from_secs(300);
        let total = Duration::from_secs(300);
        let bar = create_progress_bar(current, Some(total), 10);
        assert!(bar.contains("▓▓▓▓▓▓▓▓▓▓"));
        assert!(bar.contains("05:00 / 05:00"));
    }

    #[test]
    fn test_format_duration_zero() {
        let duration = Duration::from_secs(0);
        assert_eq!(format_duration(duration), "00:00");
    }

    #[test]
    fn test_format_duration_max_values() {
        let duration = Duration::from_secs(359999); // 99:59:59
        assert_eq!(format_duration(duration), "99:59:59");
    }

    #[test]
    fn test_progress_bar_no_total() {
        let current = Duration::from_secs(150);
        let bar = create_progress_bar(current, None, 10);
        assert!(bar.contains("02:30 / 02:30"));
    }

    #[test]
    fn test_progress_bar_over_100_percent() {
        // Test that progress is clamped at 100% when current exceeds total
        let current = Duration::from_secs(400);
        let total = Duration::from_secs(300);
        let bar = create_progress_bar(current, Some(total), 10);
        assert!(bar.contains("▓▓▓▓▓▓▓▓▓▓"));
        assert!(bar.contains("06:40 / 05:00"));
    }

    #[test]
    fn test_progress_bar_different_lengths() {
        let current = Duration::from_secs(150);
        let total = Duration::from_secs(300);

        let bar_5 = create_progress_bar(current, Some(total), 5);
        assert!(bar_5.contains("▓▓░░░") || bar_5.contains("▓▓▓░░"));

        let bar_20 = create_progress_bar(current, Some(total), 20);
        assert!(bar_20.contains("02:30 / 05:00"));
    }

    #[test]
    fn test_progress_bar_zero_length() {
        let current = Duration::from_secs(0);
        let total = Duration::from_secs(300);
        let bar = create_progress_bar(current, Some(total), 0);
        assert!(bar.contains("[] 00:00 / 05:00"));
    }
}
