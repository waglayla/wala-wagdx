use crate::imports::*;
use sys_locale::get_locale;

pub fn format_balance_with_precision(num: u64) -> (String, String, String) {
  let suffixes = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "N", "Dc"];
  let mut value = num as f64;
  let mut idx = 0;

  value /= 100000000.0;

  while value >= 1000.0 && idx < suffixes.len() - 1 {
    value /= 1000.0;
    idx += 1;
  }

  let whole_part = value.trunc() as u64;
  let fractional_part = value - whole_part as f64; // Subtract the whole part to get the fractional part

  // Truncate the fractional part to the desired precision
  let precision = if idx == 0 { 3 } else { 2 };
  let scale = 10f64.powi(precision as i32);
  let truncated_fractional = (fractional_part * scale).trunc() / scale;

  let fractional_str = format!("{:.precision$}", truncated_fractional)
    .trim_start_matches('0')
    .to_string();

  let fractional_with_suffix = if idx > 0 {
    format!("{}{}", fractional_str, suffixes[idx])
  } else {
    format!("{}", fractional_str)
  };

  // Format the whole part padded to 3 digits
  let whole_part_padded = format!("{:03}", whole_part);

  (whole_part_padded, whole_part.to_string(), fractional_with_suffix)
}

use num_format::{Locale, ToFormattedString};

pub fn get_sys_lang() -> String {
  get_locale().unwrap_or_else(|| "en".to_string())
}

pub fn format_number(num: u64) -> String {
  let mut value = num as f64;
  let locale = Locale::from_name(get_sys_lang()).unwrap_or(Locale::en);

  let integer_part = value.trunc() as u64;
  let formatted_integer = integer_part.to_formatted_string(&locale);

  format!("{}", formatted_integer)
}

pub fn format_balance(num: u64) -> String {
  let mut value = num as f64;
  value /= 100000000.0;

  let locale = Locale::from_name(get_sys_lang()).unwrap_or(Locale::en);

  let integer_part = value.trunc() as u64;
  let formatted_integer = integer_part.to_formatted_string(&locale);

  let fractional_part = (value.fract() * 100000000.0).round() as u64;
  let formatted_fractional = format!("{}", fractional_part);

  format!("{}.{}", formatted_integer, formatted_fractional)
}

pub fn format_balance_tx(num: u64) -> String {
  let suffixes = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "N", "Dc"];
  let mut value = num as f64;
  let mut idx = 0;

  value /= 100000000.0;

  while value >= 1000.0 && idx < suffixes.len() - 1 {
    value /= 1000.0;
    idx += 1;
  }

  let whole_part = value.trunc() as u64;
  let fractional_part = value.fract();

  let fractional_str = format!("{:08}", (fractional_part * 100000000.0) as u64);
  let fractional_with_suffix = format!(".{}{}", fractional_str, suffixes[idx]);

  let whole_part_formatted = format!("{}", whole_part)
    .chars()
    .rev()
    .collect::<Vec<char>>()
    .chunks(3)
    .map(|chunk| chunk.iter().collect::<String>())
    .collect::<Vec<String>>()
    .join(",")
    .chars()
    .rev()
    .collect::<String>();

  format!("{}{}", whole_part_formatted, fractional_with_suffix)
}

pub fn format_diff(num: u64) -> String {
  let suffixes = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "N", "Dc"];
  let mut value = num as f64;
  let mut idx = 0;

  while value >= 1000.0 && idx < suffixes.len() - 1 {
    value /= 1000.0;
    idx += 1;
  }

  let whole_part = value.trunc() as u64;
  let fractional_part = value.fract();

  let scale = 10f64.powi(2 as i32);
  let truncated_fractional = (fractional_part * scale).trunc() / scale;

  let fractional_str = format!("{:2}", truncated_fractional)
    .trim_start_matches('0')
    .to_string();
  let fractional_with_suffix = format!("{}{}", fractional_str, suffixes[idx]);

  let whole_part_formatted = format!("{}", whole_part)
    .chars()
    .rev()
    .collect::<Vec<char>>()
    .chunks(3)
    .map(|chunk| chunk.iter().collect::<String>())
    .collect::<Vec<String>>()
    .join(",")
    .chars()
    .rev()
    .collect::<String>();

  format!("{}{}", whole_part_formatted, fractional_with_suffix)
}

pub fn format_hashrate(num: u64) -> String {
  let suffixes = ["", "K", "M", "G", "T", "P", "E", "Z"];
  let mut value = num as f64;
  let mut idx = 0;

  while value >= 1000.0 && idx < suffixes.len() - 1 {
    value /= 1000.0;
    idx += 1;
  }

  let whole_part = value.trunc() as u64;
  let fractional_part = value.fract();

  let scale = 10f64.powi(2 as i32);
  let truncated_fractional = (fractional_part * scale).trunc() / scale;

  let fractional_str = format!("{:2}", truncated_fractional)
    .trim_start_matches('0')
    .to_string();
  let fractional_with_suffix = format!("{}{}H/s", fractional_str, suffixes[idx]);

  let whole_part_formatted = format!("{}", whole_part)
    .chars()
    .rev()
    .collect::<Vec<char>>()
    .chunks(3)
    .map(|chunk| chunk.iter().collect::<String>())
    .collect::<Vec<String>>()
    .join(",")
    .chars()
    .rev()
    .collect::<String>();

  format!("{}{}", whole_part_formatted, fractional_with_suffix)
}

pub fn format_balance_split_8(num: u64) -> (String, String) {
  let suffixes = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "N", "Dc"];
  let mut value = num as f64;
  let mut idx = 0;

  value /= 100000000.0;

  while value >= 100_000_000.0 && idx < suffixes.len() - 1 {
    value /= 1000.0;
    idx += 1;
  }

  let whole_part = value.trunc() as u64;
  let fractional_part = value.fract();

  let fractional_str = format!("{:08}", (fractional_part * 100000000.0) as u64);
  let fractional_with_suffix = format!(".{}{}", fractional_str, suffixes[idx]);

  let whole_part_formatted = format!("{}", whole_part)
    .chars()
    .rev()
    .collect::<Vec<char>>()
    .chunks(3)
    .map(|chunk| chunk.iter().collect::<String>())
    .collect::<Vec<String>>()
    .join(",")
    .chars()
    .rev()
    .collect::<String>();

  (whole_part_formatted, fractional_with_suffix)
}

pub fn format_balance_split(num: u64) -> (String, String) {
  let mut value = num as f64;
  value /= 100000000.0;

  let locale = Locale::from_name(get_sys_lang()).unwrap_or(Locale::en);

  let integer_part = value.trunc() as u64;
  let formatted_integer = integer_part.to_formatted_string(&locale);

  let fractional_part = (value.fract() * 100000000.0).round() as u64;
  let formatted_fractional = format!("{:08}", fractional_part);

  (format!("{}", formatted_integer), format!(".{}", formatted_fractional))
}

pub fn format_balance_split_raw(num: u64) -> (String, String) {
  let mut value = num as f64;
  value /= 100000000.0;

  let integer_part = value.trunc() as u64;

  let fractional_part = (value.fract() * 100000000.0).round() as u64;
  let formatted_fractional = format!("{:08}", fractional_part);

  (format!("{}", integer_part), format!(".{}", formatted_fractional))
}

pub fn address_to_compact(input: &str) -> String {
  if let Some((prefix, rest)) = input.split_once(':') {
    let prefix_part = &rest.chars().take(8).collect::<String>();
    let suffix_part = &input.chars().rev().take(6).collect::<String>().chars().rev().collect::<String>();

    return format!("{}:{}...{}", prefix, prefix_part, suffix_part);
  }

  input.to_string()
}

pub fn validate_waglayla_input(input: &str) -> Result<f64> {
  const SCALE_FACTOR: f64 = 100_000_000.0;
  const MAX_INPUT: f64 = u64::MAX as f64 / SCALE_FACTOR;

  let value = input.parse::<f64>().map_err(|_| Error::custom(i18n("Invalid numeric input").to_string()))?;

  if value > MAX_INPUT {
    Err(Error::custom(i18n("Input value is too large").to_string()))
  } else {
    Ok(value)
  }
}