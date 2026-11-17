//! Static Pinyin syllable table.
//!
//! All ~411 valid toneless syllables sourced from the standard Hanyu Pinyin chart
//! (21 initials × valid finals, plus standalone finals, syllabic nasals, ê, er).
//!
//! The array is sorted; use `binary_search` for O(log n) lookup with zero allocation.
//! `v` and `u:` are normalised to `ü` *before* lookup (see `tone::normalize_input`).

/// Full set of valid toneless Pinyin syllables (lowercase, `ü` for ü-vowel).
/// Sorted lexicographically.
pub(crate) static SYLLABLE_SET: &[&str] = &[
    "a", "ai", "an", "ang", "ao", "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi",
    "bian", "biao", "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "ce",
    "cen", "ceng", "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi", "chong",
    "chou", "chu", "chua", "chuai", "chuan", "chuang", "chui", "chun", "chuo", "ci", "cong", "cou",
    "cu", "cuan", "cui", "cun", "cuo", "da", "dai", "dan", "dang", "dao", "de", "dei", "den",
    "deng", "di", "dia", "dian", "diao", "die", "ding", "diu", "dong", "dou", "du", "duan", "dui",
    "dun", "duo", "e", "ei", "en", "eng", "er", "fa", "fan", "fang", "fei", "fen", "feng", "fo",
    "fou", "fu", "ga", "gai", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong", "gou",
    "gu", "gua", "guai", "guan", "guang", "gui", "gun", "guo", "ha", "hai", "han", "hang", "hao",
    "he", "hei", "hen", "heng", "hm", "hng", "hong", "hou", "hu", "hua", "huai", "huan", "huang",
    "hui", "hun", "huo", "ji", "jia", "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong",
    "jiu", "ju", "juan", "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "kei", "ken",
    "keng", "kong", "kou", "ku", "kua", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai",
    "lan", "lang", "lao", "le", "lei", "leng", "li", "lia", "lian", "liang", "liao", "lie", "lin",
    "ling", "liu", "lo", "long", "lou", "lu", "luan", "lun", "luo", "lü", "lüe", "m", "ma", "mai",
    "man", "mang", "mao", "mei", "men", "meng", "mi", "mian", "miao", "mie", "min", "ming", "miu",
    "mo", "mou", "mu", "n", "na", "nai", "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ng",
    "ni", "nian", "niang", "niao", "nie", "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nuo",
    "nü", "nüe", "o", "ou", "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian",
    "piao", "pie", "pin", "ping", "po", "pou", "pu", "qi", "qia", "qian", "qiang", "qiao", "qie",
    "qin", "qing", "qiong", "qiu", "qu", "quan", "que", "qun", "ran", "rang", "rao", "re", "ren",
    "reng", "ri", "rong", "rou", "ru", "rua", "ruan", "rui", "run", "ruo", "sa", "sai", "san",
    "sang", "sao", "se", "sen", "seng", "sha", "shai", "shan", "shang", "shao", "she", "shei",
    "shen", "sheng", "shi", "shou", "shu", "shua", "shuai", "shuan", "shuang", "shui", "shun",
    "shuo", "si", "song", "sou", "su", "suan", "sui", "sun", "suo", "ta", "tai", "tan", "tang",
    "tao", "te", "teng", "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu", "tuan", "tui",
    "tun", "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", "xi", "xia",
    "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan", "xue", "xun",
    "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yong", "you", "yu", "yuan", "yue",
    "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zha", "zhai", "zhan",
    "zhang", "zhao", "zhe", "zhei", "zhen", "zheng", "zhi", "zhong", "zhou", "zhu", "zhua",
    "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", "zi", "zong", "zou", "zu", "zuan", "zui",
    "zun", "zuo", "ê",
];

/// Normalizes a raw string for table lookup:
/// - lowercases
/// - maps `v` to `ü`
/// - maps `lue`/`nue` to `lüe`/`nüe`
pub(crate) fn normalize_for_table(s: &str) -> String {
    let mut norm: String = s
        .chars()
        .map(|c| {
            let lc = c.to_lowercase().next().unwrap_or(c);
            if lc == 'v' { 'ü' } else { lc }
        })
        .collect();
    if (norm.starts_with("lu") || norm.starts_with("nu"))
        && norm.len() >= 3
        && norm[2..].starts_with('e')
    {
        let mut chars: Vec<char> = norm.chars().collect();
        chars[1] = 'ü';
        norm = chars.into_iter().collect();
    }
    norm
}

/// Returns `true` if `s` is a complete valid Pinyin syllable.
pub(crate) fn is_valid_syllable(s: &str) -> bool {
    let norm = normalize_for_table(s);
    SYLLABLE_SET.binary_search(&norm.as_str()).is_ok()
}

/// Returns `true` if `s` could be the start of a valid Pinyin syllable.
pub(crate) fn is_valid_prefix(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    let norm = normalize_for_table(s);
    let idx = SYLLABLE_SET.partition_point(|&entry| entry < norm.as_str());
    SYLLABLE_SET
        .get(idx)
        .is_some_and(|entry| entry.starts_with(norm.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syllable_set_is_sorted() {
        let mut sorted = SYLLABLE_SET.to_vec();
        sorted.sort_unstable();
        assert_eq!(
            SYLLABLE_SET,
            sorted.as_slice(),
            "SYLLABLE_SET must be sorted for binary_search to work"
        );
    }

    #[test]
    fn syllable_set_no_duplicates() {
        let mut prev = "";
        for &s in SYLLABLE_SET {
            assert_ne!(s, prev, "Duplicate syllable: {s}");
            prev = s;
        }
    }

    #[test]
    fn known_syllables_present() {
        let required = [
            "ni", "hao", "zhong", "guo", "xi", "an", "liu", "gui", "hou", "lü", "lüe", "nü", "er",
            "hua", "ma", "yi", "wu", "yu", "ying", "yue", "yuan", "zhi", "chi", "shi", "ri", "zi",
            "ci", "si", "xue", "jun", "quan", "xuan",
        ];
        for s in required {
            assert!(is_valid_syllable(s), "Missing required syllable: {s}");
        }
    }

    #[test]
    fn invalid_syllables_absent() {
        let invalid = ["hello", "vscode", "win", "mp", "abc", "xyz"];
        for s in invalid {
            assert!(!is_valid_syllable(s), "Unexpected syllable: {s}");
        }
    }

    #[test]
    fn prefix_detection() {
        assert!(is_valid_prefix("n"));
        assert!(is_valid_prefix("zh"));
        assert!(is_valid_prefix("ni"));
        assert!(is_valid_prefix("zho"));
        assert!(!is_valid_prefix("hello"));
        assert!(!is_valid_prefix("mp"));
        assert!(is_valid_prefix(""));
    }

    #[test]
    fn jqxy_syllables_use_u_not_ü() {
        // After j/q/x/y the ü is written as u in the table
        assert!(is_valid_syllable("ju"));
        assert!(is_valid_syllable("qu"));
        assert!(is_valid_syllable("xu"));
        assert!(is_valid_syllable("yu"));
        assert!(is_valid_syllable("xue"));
        assert!(is_valid_syllable("yuan"));
        // lü/nü use ü
        assert!(is_valid_syllable("lü"));
        assert!(is_valid_syllable("nü"));
    }

    #[test]
    fn erhua_base_present() {
        assert!(is_valid_syllable("hua")); // huar = hua + r (erhua)
        assert!(is_valid_syllable("er"));
    }
}
