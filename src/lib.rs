//! # nengo
//!
//! 和暦文字列をパースするクレートです。
//!
//! 明治・大正・昭和・平成・令和に対応しており、漢数字・全角数字・半角数字の混在も扱えます。
//!
//! ## 使い方
//!
//! ```rust
//! use nengo::parse_wareki_date;
//!
//! let date = parse_wareki_date("令和六年五月二十四日").unwrap();
//! assert_eq!(date.to_string(), "2024-05-24");
//! ```

use once_cell::sync::Lazy;
use regex::Regex;

// Internal

/// 全角ASCII数字を半角に変換する関数（'０'→'0' など）
fn to_halfwidth(c: char) -> char {
    if ('０'..='９').contains(&c) {
        char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
    } else {
        c
    }
}

/// 和暦を年号にキャストする内部関数
fn parse_wareki_inner(raw: &str) -> Option<String> {
    let caps = DATE_RE.captures(raw)?;
    let offset: u32 = match &caps[1] {
        "令和" => 2018,
        "平成" => 1988,
        "昭和" => 1925,
        "大正" => 1911,
        "明治" => 1867,
        _ => return None,
    };
    let y = offset + kanji_to_num(&caps[2])?;
    let m = kanji_to_num(&caps[3])?;
    let d = kanji_to_num(&caps[4])?;
    Some(format!("{:04}{:02}{:02}", y, m, d))
}

/// 明治〜令和までの正規表現
static DATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(明治|大正|昭和|平成|令和)([0-9０-９一二三四五六七八九十百元]+?)年([0-9０-９一二三四五六七八九十百元]+?)月([0-9０-９一二三四五六七八九十百元]+?)日$",
    )
    .unwrap()
});

/// 漢数字・アラビア数字（半角・全角）を `u32` に変換します。
///
/// `元` は `1` として扱います。対応範囲は 1〜999 です。
///
/// # Examples
///
/// ```rust
/// use nengo::kanji_to_num;
///
/// assert_eq!(kanji_to_num("元"), Some(1));
/// assert_eq!(kanji_to_num("二十四"), Some(24));
/// assert_eq!(kanji_to_num("２４"), Some(24));
/// assert_eq!(kanji_to_num("abc"), None);
/// ```
pub fn kanji_to_num(kanji: &str) -> Option<u32> {
    if kanji == "元" {
        return Some(1);
    }
    // 全角数字を半角に正規化
    let normalized: String = kanji.chars().map(to_halfwidth).collect();
    // アラビア数字のみならそのままパース
    if let Ok(n) = normalized.parse::<u32>() {
        return Some(n);
    }
    // 漢数字パース
    let mut total = 0u32;
    let mut temp = 0u32;
    for c in normalized.chars() {
        match c {
            '百' => {
                total += if temp == 0 { 100 } else { temp * 100 };
                temp = 0;
            }
            '十' => {
                total += if temp == 0 { 10 } else { temp * 10 };
                temp = 0;
            }
            '一' => temp = 1,
            '二' => temp = 2,
            '三' => temp = 3,
            '四' => temp = 4,
            '五' => temp = 5,
            '六' => temp = 6,
            '七' => temp = 7,
            '八' => temp = 8,
            '九' => temp = 9,
            '0'..='9' => temp = c.to_digit(10).unwrap(),
            _ => return None,
        }
    }
    Some(total + temp)
}

/// 和暦文字列を [`chrono::NaiveDate`] に変換します。
/// パース失敗・不正な日付（2月30日など）の場合は `None` を返します。
///
/// 対応元号：明治・大正・昭和・平成・令和
///
/// 年月日は以下の形式が混在していても解釈できます：
/// - 漢数字（`二十四`）
/// - 半角アラビア数字（`24`）
/// - 全角アラビア数字（`２４`）
/// - 元年表記（`元`）
///
/// # Examples
///
/// ```rust
/// use nengo::parse_wareki_date;
///
/// // 漢数字
/// assert!(parse_wareki_date("令和六年五月二十四日").is_some());
///
/// // 元年
/// assert!(parse_wareki_date("令和元年五月一日").is_some());
///
/// // 全角数字
/// assert!(parse_wareki_date("令和６年５月２４日").is_some());
///
/// // 不正な日付はNone
/// assert!(parse_wareki_date("令和六年二月三十日").is_none());
///
/// // パース失敗もNone
/// assert!(parse_wareki_date("20240524").is_none());
/// ```
#[cfg(feature = "chrono")]
pub fn parse_wareki_date(raw: &str) -> Option<chrono::NaiveDate> {
    let s = parse_wareki_inner(raw)?;
    chrono::NaiveDate::parse_from_str(&s, "%Y%m%d").ok()
}

/// 和暦文字列を `YYYYMMDD` 形式の [`String`] に変換します。
/// パース失敗時は `None` を返します。
///
/// # Examples
///
/// ```rust
/// #[allow(deprecated)]
/// use nengo::parse_wareki;
///
/// assert_eq!(parse_wareki("令和六年五月二十四日"), Some("20240524".into()));
/// assert_eq!(parse_wareki("令和元年五月一日"), Some("20190501".into()));
/// ```
///
/// # Deprecation
///
/// [`parse_wareki_date`] を使ってください。不正な日付（2月30日など）を弾けます。
#[deprecated(since = "0.2.0", note = "Use `parse_wareki_date` instead")]
pub fn parse_wareki(raw: &str) -> Option<String> {
    parse_wareki_inner(raw)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_kanji_to_num() {
        assert_eq!(kanji_to_num("元"), Some(1));
        assert_eq!(kanji_to_num("一"), Some(1));
        assert_eq!(kanji_to_num("十"), Some(10));
        assert_eq!(kanji_to_num("十一"), Some(11));
        assert_eq!(kanji_to_num("二十"), Some(20));
        assert_eq!(kanji_to_num("二十四"), Some(24));
        assert_eq!(kanji_to_num("六十四"), Some(64));
        assert_eq!(kanji_to_num("百"), Some(100));
        assert_eq!(kanji_to_num("百二十三"), Some(123));
        assert_eq!(kanji_to_num("二百"), Some(200));
        // アラビア数字（半角）
        assert_eq!(kanji_to_num("6"), Some(6));
        assert_eq!(kanji_to_num("24"), Some(24));
        // アラビア数字（全角）
        assert_eq!(kanji_to_num("６"), Some(6));
        assert_eq!(kanji_to_num("２４"), Some(24));
        // 不正
        assert_eq!(kanji_to_num("abc"), None);
    }

    #[test]
    fn test_reiwa() {
        assert_eq!(
            parse_wareki("令和六年五月二十四日"),
            Some("20240524".into())
        );
        assert_eq!(parse_wareki("令和元年五月一日"), Some("20190501".into()));
    }

    #[test]
    fn test_heisei() {
        assert_eq!(parse_wareki("平成元年一月八日"), Some("19890108".into()));
        assert_eq!(
            parse_wareki("平成三十一年四月三十日"),
            Some("20190430".into())
        );
    }

    #[test]
    fn test_showa() {
        assert_eq!(
            parse_wareki("昭和六十四年一月七日"),
            Some("19890107".into())
        );
        assert_eq!(
            parse_wareki("昭和元年十二月二十五日"),
            Some("19261225".into())
        );
    }

    #[test]
    fn test_taisho() {
        assert_eq!(parse_wareki("大正元年七月三十日"), Some("19120730".into()));
        assert_eq!(
            parse_wareki("大正十五年十二月二十五日"),
            Some("19261225".into())
        );
    }

    #[test]
    fn test_meiji() {
        assert_eq!(parse_wareki("明治五年十一月九日"), Some("18721109".into()));
        assert_eq!(
            parse_wareki("明治四十五年七月三十日"),
            Some("19120730".into())
        );
    }

    #[test]
    fn test_mixed_width() {
        // 全角数字のみ
        assert_eq!(parse_wareki("令和６年５月２４日"), Some("20240524".into()));
        // 半角・全角・漢字の混合
        assert_eq!(parse_wareki("令和８年九月２7日"), Some("20260927".into()));
        assert_eq!(parse_wareki("令和８年９月２７日"), Some("20260927".into()));
        assert_eq!(
            parse_wareki("令和６年１２月３１日"),
            Some("20241231".into())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse_wareki("20240524"), None);
        assert_eq!(parse_wareki(""), None);
        assert_eq!(parse_wareki("令和六年五月"), None);
    }
}
