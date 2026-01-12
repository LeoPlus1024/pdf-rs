use std::ops::Range;
use std::str::FromStr;
use crate::error::PDFError;

/// Represents a date and time value used in PDF documents.
///
/// This struct stores time information with millisecond precision,
/// following the PDF specification for date/time representation.
pub struct Date {
    /// Time zone offset from UTC in hours.
    pub(crate) time_zero: i8,
    /// Milliseconds component of the time.
    pub(crate) millisecond: u64,
}



impl Date {
    /// Creates a new Date instance with the specified date and time components.
    ///
    /// # Arguments
    ///
    /// * `year` - The year (e.g., 2024)
    /// * `month` - The month (1-12)
    /// * `day` - The day of the month (1-31)
    /// * `hour` - The hour (0-23)
    /// * `minute` - The minute (0-59)
    /// * `second` - The second (0-59)
    /// * `time_zero` - Time zone offset from UTC in hours (-12 to +12)
    /// * `utm` - shall be the absolute value of the offset from UT in minutes (00–59)
    ///
    /// # Returns
    ///
    /// A new Date instance with the calculated Unix timestamp in milliseconds
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, time_zero: i8, utm: u8) -> Self {
        let millisecond = Self::calculate_unix_timestamp_millis(year, month, day, hour, minute, second, time_zero, utm);

        Date {
            time_zero,
            millisecond,
        }
    }

    /// Calculates the Unix timestamp in milliseconds from 1970-01-01 00:00:00 UTC.
    ///
    /// This function computes the number of milliseconds elapsed since the Unix epoch
    /// (January 1, 1970, 00:00:00 UTC) for the given date and time, taking into account
    /// the time zone offset.
    ///
    /// # Arguments
    ///
    /// * `year` - The year (e.g., 2024)
    /// * `month` - The month (1-12)
    /// * `day` - The day of the month (1-31)
    /// * `hour` - The hour (0-23)
    /// * `minute` - The minute (0-59)
    /// * `second` - The second (0-59)
    /// * `time_zero` - Time zone offset from UTC in hours (-12 to +12)
    /// * `utm` - Milliseconds component (0-999)
    ///
    /// # Returns
    ///
    /// The Unix timestamp in milliseconds
    fn calculate_unix_timestamp_millis(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, time_zero: i8, utm: u8) -> u64 {
        // Days in each month for non-leap years
       static DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        // Calculate total days from 1970 to the given year
        let mut total_days: u64 = 0;

        // Add days for full years from 1970 to year-1
        for y in 1970..year {
            total_days += if Self::is_leap_year(y) { 366 } else { 365 };
        }

        // Add days for full months in the current year
        for m in 1..month {
            let month_idx = (m - 1) as usize;
            total_days += DAYS_IN_MONTH[month_idx];
            // Add leap day for February in leap years
            if m == 2 && Self::is_leap_year(year) {
                total_days += 1;
            }
        }

        // Add days in the current month
        total_days += (day - 1) as u64;

        // Convert days to seconds
        let mut total_seconds = total_days * 86400;

        // Add hours, minutes, and seconds
        total_seconds += hour as u64 * 3600;
        total_seconds += minute as u64 * 60;
        total_seconds += second as u64;

        // Adjust for time zone offset (convert hours to seconds)
        let tz_offset_seconds = (time_zero as i64) * 3600;
        total_seconds = (total_seconds as i64 - tz_offset_seconds) as u64;

        // Convert to milliseconds and add the milliseconds component
        total_seconds * 1000 + (utm as u64) * 60 * 1000
    }

    /// Determines if a given year is a leap year.
    ///
    /// A year is a leap year if:
    /// - It is divisible by 4, and
    /// - It is not divisible by 100, unless it is also divisible by 400
    ///
    /// # Arguments
    ///
    /// * `year` - The year to check
    ///
    /// # Returns
    ///
    /// true if the year is a leap year, false otherwise
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

fn parse_part(text: &str, range: Range<usize>) -> u8 {
    text.get(range)
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0)
}

impl FromStr for Date {
    type Err = PDFError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let length = text.len();
        if !text.starts_with("D:") || length < 6 {
            return Err(PDFError::IllegalDateFormat(text.to_string()));
        }
        let year = text[2..6].parse::<i32>().unwrap_or(0);
        let month = parse_part(text, 6..8);
        let day = parse_part(text, 8..10);
        let hour = parse_part(text, 10..12);
        let minute = parse_part(text, 12..14);
        let second = parse_part(text, 14..16);
        let (tz, utm) = if length >= 17 {
            let tmp = &text[16..17];
            let mut index = 17;
            let time_zero = if tmp == "Z" {
                0
            } else {
                let plus_sign = tmp == "+";
                let minus_sign = tmp == "-";
                if !plus_sign || minus_sign || length < 19 {
                    return Err(PDFError::IllegalDateFormat(text.to_string()));
                }
                let tz = parse_part(text, 17..19) as i8;
                index = 19;
                if minus_sign {
                    -tz
                } else {
                    tz
                }
            };
            if length > index && index + 3 != length {
                return Err(PDFError::IllegalDateFormat(text.to_string()));
            }
            let utm = parse_part(text, index + 1..length);
            (time_zero, utm)
        } else {
            (0, 0)
        };
        Ok(Self::new(year, month, day, hour, minute, second, tz, utm))
    }

}