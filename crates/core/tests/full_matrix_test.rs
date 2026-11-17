//! Full matrix, fuzz, and timing tests for tkey-core.
//!
//! Covers:
//! - 200+ syllable and sentence test cases
//! - Fuzz test: randomized arbitrary UTF-8 input must never panic
//! - Timing test: per-keystroke latency < 1ms

use std::time::Instant;
use tkey_core::{Config, Engine, apply_pinyin_tone, segment_pinyin};

fn feed_and_commit(input: &str) -> String {
    let mut e = Engine::new(Config::default());
    for ch in input.chars() {
        e.feed(ch);
    }
    e.commit().commit
}

#[test]
fn test_200_syllables_and_tones() {
    let cases = [
        // a-
        ("a1", "ā"),
        ("a2", "á"),
        ("a3", "ǎ"),
        ("a4", "à"),
        ("a5", "a"),
        ("ai1", "āi"),
        ("ai2", "ái"),
        ("ai3", "ǎi"),
        ("ai4", "ài"),
        ("an1", "ān"),
        ("an2", "án"),
        ("an3", "ǎn"),
        ("an4", "àn"),
        ("ang1", "āng"),
        ("ang2", "áng"),
        ("ang3", "ǎng"),
        ("ang4", "àng"),
        ("ao1", "āo"),
        ("ao2", "áo"),
        ("ao3", "ǎo"),
        ("ao4", "ào"),
        // b-
        ("ba1", "bā"),
        ("ba2", "bá"),
        ("ba3", "bǎ"),
        ("ba4", "bà"),
        ("bai1", "bāi"),
        ("bai2", "bái"),
        ("bai3", "bǎi"),
        ("bai4", "bài"),
        ("ban1", "bān"),
        ("ban2", "bán"),
        ("ban3", "bǎn"),
        ("ban4", "bàn"),
        ("bang1", "bāng"),
        ("bang2", "báng"),
        ("bang3", "bǎng"),
        ("bang4", "bàng"),
        ("bao1", "bāo"),
        ("bao2", "báo"),
        ("bao3", "bǎo"),
        ("bao4", "bào"),
        ("bei1", "bēi"),
        ("bei2", "béi"),
        ("bei3", "běi"),
        ("bei4", "bèi"),
        ("ben1", "bēn"),
        ("ben3", "běn"),
        ("ben4", "bèn"),
        ("beng1", "bēng"),
        ("beng2", "béng"),
        ("beng3", "běng"),
        ("beng4", "bèng"),
        ("bi1", "bī"),
        ("bi2", "bí"),
        ("bi3", "bǐ"),
        ("bi4", "bì"),
        ("bian1", "biān"),
        ("bian3", "biǎn"),
        ("bian4", "biàn"),
        ("biao1", "biāo"),
        ("biao3", "biǎo"),
        ("biao4", "biào"),
        ("bie1", "biē"),
        ("bie2", "bié"),
        ("bie3", "biě"),
        ("bie4", "biè"),
        ("bin1", "bīn"),
        ("bin3", "bǐn"),
        ("bin4", "bìn"),
        ("bing1", "bīng"),
        ("bing3", "bǐng"),
        ("bing4", "bìng"),
        ("bo1", "bō"),
        ("bo2", "bó"),
        ("bo3", "bǒ"),
        ("bo4", "bò"),
        ("bu1", "bū"),
        ("bu2", "bú"),
        ("bu3", "bǔ"),
        ("bu4", "bù"),
        // c-
        ("ca1", "cā"),
        ("ca3", "cǎ"),
        ("ca4", "cà"),
        ("cai1", "cāi"),
        ("cai2", "cái"),
        ("cai3", "cǎi"),
        ("cai4", "cài"),
        ("can1", "cān"),
        ("can2", "cán"),
        ("can3", "cǎn"),
        ("can4", "càn"),
        ("cang1", "cāng"),
        ("cang2", "cáng"),
        ("cang3", "cǎng"),
        ("cang4", "càng"),
        ("cao1", "cāo"),
        ("cao2", "cáo"),
        ("cao3", "cǎo"),
        ("cao4", "cào"),
        ("ce4", "cè"),
        ("cen1", "cēn"),
        ("ceng2", "céng"),
        ("ceng4", "cèng"),
        ("cha1", "chā"),
        ("cha2", "chá"),
        ("cha3", "chǎ"),
        ("cha4", "chà"),
        ("chai1", "chāi"),
        ("chai2", "chái"),
        ("chai3", "chǎi"),
        ("chai4", "chài"),
        ("chan1", "chān"),
        ("chan2", "chán"),
        ("chan3", "chǎn"),
        ("chan4", "chàn"),
        ("chang1", "chāng"),
        ("chang2", "cháng"),
        ("chang3", "chǎng"),
        ("chang4", "chàng"),
        ("chao1", "chāo"),
        ("chao2", "cháo"),
        ("chao3", "chǎo"),
        ("chao4", "chào"),
        ("che1", "chē"),
        ("che3", "chě"),
        ("che4", "chè"),
        ("chen1", "chēn"),
        ("chen2", "chén"),
        ("chen3", "chěn"),
        ("chen4", "chèn"),
        ("cheng1", "chēng"),
        ("cheng2", "chéng"),
        ("cheng3", "chěng"),
        ("cheng4", "chèng"),
        ("chi1", "chī"),
        ("chi2", "chí"),
        ("chi3", "chǐ"),
        ("chi4", "chì"),
        ("chong1", "chōng"),
        ("chong2", "chóng"),
        ("chong3", "chǒng"),
        ("chong4", "chòng"),
        ("chou1", "chōu"),
        ("chou2", "chóu"),
        ("chou3", "chǒu"),
        ("chou4", "chòu"),
        ("chu1", "chū"),
        ("chu2", "chú"),
        ("chu3", "chǔ"),
        ("chu4", "chù"),
        ("chuai1", "chuāi"),
        ("chuai4", "chuài"),
        ("chuan1", "chuān"),
        ("chuan2", "chuán"),
        ("chuan3", "chuǎn"),
        ("chuan4", "chuàn"),
        ("chuang1", "chuāng"),
        ("chuang2", "chuáng"),
        ("chuang3", "chuǎng"),
        ("chuang4", "chuàng"),
        ("chui1", "chuī"),
        ("chui2", "chuí"),
        ("chun1", "chūn"),
        ("chun2", "chún"),
        ("chun3", "chǔn"),
        ("chuo1", "chuō"),
        ("chuo4", "chuò"),
        // d-
        ("da1", "dā"),
        ("da2", "dá"),
        ("da3", "dǎ"),
        ("da4", "dà"),
        ("dai1", "dāi"),
        ("dai3", "dǎi"),
        ("dai4", "dài"),
        ("dan1", "dān"),
        ("dan3", "dǎn"),
        ("dan4", "dàn"),
        ("dang1", "dāng"),
        ("dang3", "dǎng"),
        ("dang4", "dàng"),
        ("dao1", "dāo"),
        ("dao2", "dáo"),
        ("dao3", "dǎo"),
        ("dao4", "dào"),
        ("de1", "dē"),
        ("de2", "dé"),
        ("de5", "de"),
        ("deng1", "dēng"),
        ("deng3", "děng"),
        ("deng4", "dèng"),
        ("di1", "dī"),
        ("di2", "dí"),
        ("di3", "dǐ"),
        ("di4", "dì"),
        ("dian1", "diān"),
        ("dian3", "diǎn"),
        ("dian4", "diàn"),
        ("diao1", "diāo"),
        ("diao3", "diǎo"),
        ("diao4", "diào"),
        ("die1", "diē"),
        ("die2", "dié"),
        ("die4", "diè"),
        ("ding1", "dīng"),
        ("ding3", "dǐng"),
        ("ding4", "dìng"),
        ("diu1", "diū"),
        ("dong1", "dōng"),
        ("dong3", "dǒng"),
        ("dong4", "dòng"),
        ("dou1", "dōu"),
        ("dou3", "dǒu"),
        ("dou4", "dòu"),
        ("du1", "dū"),
        ("du2", "dú"),
        ("du3", "dǔ"),
        ("du4", "dù"),
        ("duan1", "duān"),
        ("duan3", "duǎn"),
        ("duan4", "duàn"),
        ("dui1", "duī"),
        ("dui4", "duì"),
        ("dun1", "dūn"),
        ("dun3", "dǔn"),
        ("dun4", "dùn"),
        ("duo1", "duō"),
        ("duo2", "duó"),
        ("duo3", "duǒ"),
        ("duo4", "duò"),
        // g-
        ("ga1", "gā"),
        ("ga2", "gá"),
        ("ga3", "gǎ"),
        ("ga4", "gà"),
        ("gai1", "gāi"),
        ("gai3", "gǎi"),
        ("gai4", "gài"),
        ("gan1", "gān"),
        ("gan3", "gǎn"),
        ("gan4", "gàn"),
        ("gang1", "gāng"),
        ("gang3", "gǎng"),
        ("gang4", "gàng"),
        ("gao1", "gāo"),
        ("gao3", "gǎo"),
        ("gao4", "gào"),
        ("ge1", "gē"),
        ("ge2", "gé"),
        ("ge3", "gě"),
        ("ge4", "gè"),
        ("gen1", "gēn"),
        ("gen3", "gěn"),
        ("gen4", "gèn"),
        ("geng1", "gēng"),
        ("geng3", "gěng"),
        ("geng4", "gèng"),
        ("gong1", "gōng"),
        ("gong3", "gǒng"),
        ("gong4", "gòng"),
        ("gou1", "gōu"),
        ("gou3", "gǒu"),
        ("gou4", "gòu"),
        ("gu1", "gū"),
        ("gu2", "gú"),
        ("gu3", "gǔ"),
        ("gu4", "gù"),
        ("gua1", "guā"),
        ("gua3", "guǎ"),
        ("gua4", "guà"),
        ("guai1", "guāi"),
        ("guai3", "guǎi"),
        ("guai4", "guài"),
        ("guan1", "guān"),
        ("guan3", "guǎn"),
        ("guan4", "guàn"),
        ("guang1", "guāng"),
        ("guang3", "guǎng"),
        ("guang4", "guàng"),
        ("gui1", "guī"),
        ("gui3", "guǐ"),
        ("gui4", "guì"),
        ("gun1", "gūn"),
        ("gun3", "gǔn"),
        ("gun4", "gùn"),
        ("guo1", "guō"),
        ("guo2", "guó"),
        ("guo3", "guǒ"),
        ("guo4", "guò"),
        // h-
        ("ha1", "hā"),
        ("ha2", "há"),
        ("ha3", "hǎ"),
        ("hai1", "hāi"),
        ("hai2", "hái"),
        ("hai3", "hǎi"),
        ("hai4", "hài"),
        ("han1", "hān"),
        ("han2", "hán"),
        ("han3", "hǎn"),
        ("han4", "hàn"),
        ("hang1", "hāng"),
        ("hang2", "háng"),
        ("hao1", "hāo"),
        ("hao2", "háo"),
        ("hao3", "hǎo"),
        ("hao4", "hào"),
        ("he1", "hē"),
        ("he2", "hé"),
        ("he4", "hè"),
        ("hei1", "hēi"),
        ("hen2", "hén"),
        ("hen3", "hěn"),
        ("hen4", "hèn"),
        ("heng1", "hēng"),
        ("heng2", "héng"),
        ("hong1", "hōng"),
        ("hong2", "hóng"),
        ("hong3", "hǒng"),
        ("hong4", "hòng"),
        ("hou2", "hóu"),
        ("hou3", "hǒu"),
        ("hou4", "hòu"),
        ("hu1", "hū"),
        ("hu2", "hú"),
        ("hu3", "hǔ"),
        ("hu4", "hù"),
        ("hua1", "huā"),
        ("hua2", "huá"),
        ("hua4", "huà"),
        ("huai2", "huái"),
        ("huai4", "huài"),
        ("huan1", "huān"),
        ("huan2", "huán"),
        ("huan3", "huǎn"),
        ("huan4", "huàn"),
        ("huang1", "huāng"),
        ("huang2", "huáng"),
        ("huang3", "huǎng"),
        ("huang4", "huàng"),
        ("hui1", "huī"),
        ("hui2", "huí"),
        ("hui3", "huǐ"),
        ("hui4", "huì"),
        ("hun1", "hūn"),
        ("hun2", "hún"),
        ("hun4", "hùn"),
        ("huo1", "huō"),
        ("huo2", "huó"),
        ("huo3", "huǒ"),
        ("huo4", "huò"),
        // j-
        ("ji1", "jī"),
        ("ji2", "jí"),
        ("ji3", "jǐ"),
        ("ji4", "jì"),
        ("jia1", "jiā"),
        ("jia2", "jiá"),
        ("jia3", "jiǎ"),
        ("jia4", "jià"),
        ("jian1", "jiān"),
        ("jian3", "jiǎn"),
        ("jian4", "jiàn"),
        ("jiang1", "jiāng"),
        ("jiang3", "jiǎng"),
        ("jiang4", "jiàng"),
        ("jiao1", "jiāo"),
        ("jiao2", "jiáo"),
        ("jiao3", "jiǎo"),
        ("jiao4", "jiào"),
        ("jie1", "jiē"),
        ("jie2", "jié"),
        ("jie3", "jiě"),
        ("jie4", "jiè"),
        ("jin1", "jīn"),
        ("jin3", "jǐn"),
        ("jin4", "jìn"),
        ("jing1", "jīng"),
        ("jing3", "jǐng"),
        ("jing4", "jìng"),
        ("jiong1", "jiōng"),
        ("jiong3", "jiǒng"),
        ("jiu1", "jiū"),
        ("jiu3", "jiǔ"),
        ("jiu4", "jiù"),
        ("ju1", "jū"),
        ("ju2", "jú"),
        ("ju3", "jǔ"),
        ("ju4", "jù"),
        ("juan1", "juān"),
        ("juan3", "juǎn"),
        ("juan4", "juàn"),
        ("jue1", "juē"),
        ("jue2", "jué"),
        ("jue4", "juè"),
        ("jun1", "jūn"),
        ("jun4", "jùn"),
        // l-
        ("lv1", "lǖ"),
        ("lv2", "lǘ"),
        ("lv3", "lǚ"),
        ("lv4", "lǜ"),
        ("lve4", "lüè"),
        ("lue4", "lüè"),
        ("liu1", "liū"),
        ("liu2", "liú"),
        ("liu3", "liǔ"),
        ("liu4", "liù"),
        // n-
        ("nv3", "nǚ"),
        ("nu:3", "nǚ"),
        // w-
        ("wo3", "wǒ"),
        ("wen4", "wèn"),
        ("wang3", "wǎng"),
        // x-
        ("xue2", "xué"),
        ("xuan3", "xuǎn"),
        ("xian1", "xiān"),
        // y-
        ("yi1", "yī"),
        ("yuan2", "yuán"),
        ("yue4", "yuè"),
        // z-
        ("zhong1", "zhōng"),
        ("zheng4", "zhèng"),
    ];

    for (input, expected) in cases {
        let got = feed_and_commit(input);
        assert_eq!(got, expected, "Failed for input {input}");
    }
}

#[test]
fn test_sentences() {
    let sentences = [
        ("ni3hao3", "nǐhǎo"),
        ("wo3ai4zhong1guo2", "wǒàizhōngguó"),
        ("ni3hao3ma5", "nǐhǎoma"),
        ("xian1sheng1", "xiānshēng"),
        ("zhong1hua2ren2min2gong4he2guo2", "zhōnghuárénmíngònghéguó"),
        ("bei3jing1", "běijīng"),
        ("shang4hai3", "shànghǎi"),
        ("guang3zhou1", "guǎngzhōu"),
        ("shen1zhen4", "shēnzhèn"),
        ("si4chuan1", "sìchuān"),
    ];

    for (input, expected) in sentences {
        let got = feed_and_commit(input);
        assert_eq!(got, expected, "Sentence failed for {input}");
    }
}

#[test]
fn test_fuzz_randomized_input_never_panics() {
    // Generate pseudorandom inputs mixing ASCII, punctuation, unicode, digits, control chars
    let mut state: u64 = 0x123456789abcdef0;
    let mut next_u32 = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (state >> 32) as u32
    };

    let sample_chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789': !?,\n\t\x00\x08üöä你好🌍🔥".chars().collect();

    let mut engine = Engine::new(Config::default());

    for _ in 0..10_000 {
        let op = next_u32() % 4;
        match op {
            0 => {
                let idx = (next_u32() as usize) % sample_chars.len();
                let ch = sample_chars[idx];
                let out = engine.feed(ch);
                // Ensure output is valid UTF-8 (String guarantees this, but check properties)
                assert!(out.commit.chars().count() <= out.commit.len());
                assert!(out.preedit.chars().count() <= out.preedit.len());
            }
            1 => {
                let out = engine.backspace();
                assert!(out.commit.chars().count() <= out.commit.len());
            }
            2 => {
                let out = engine.commit();
                assert!(out.preedit.is_empty());
            }
            _ => {
                engine.reset();
            }
        }
    }
}

#[test]
fn test_timing_sub_millisecond_keystrokes() {
    let mut engine = Engine::new(Config::default());
    let stream = "ni3hao3zhong1guo2ren2shen1zhen4bei3jing1shang4hai3".repeat(100);

    let start = Instant::now();
    let count = stream.chars().count();
    for ch in stream.chars() {
        let _ = engine.feed(ch);
    }
    let elapsed = start.elapsed();

    let per_keystroke_micros = elapsed.as_micros() as f64 / count as f64;
    println!(
        "Benchmark: {count} keystrokes processed in {:?}. Average per keystroke: {:.2} µs",
        elapsed, per_keystroke_micros
    );

    // Keystroke must be well under 1,000 µs (1 ms); typically < 10 µs
    assert!(
        per_keystroke_micros < 1000.0,
        "Keystroke too slow: {per_keystroke_micros} µs"
    );
}

#[test]
fn test_segment_pinyin_public_api() {
    assert_eq!(segment_pinyin("nihao"), vec!["ni", "hao"]);
    assert_eq!(segment_pinyin("zhongguo"), vec!["zhong", "guo"]);
    assert_eq!(segment_pinyin("xi'an"), vec!["xi", "an"]);
}

#[test]
fn test_apply_pinyin_tone_public_api() {
    assert_eq!(apply_pinyin_tone("ni", 3), "nǐ");
    assert_eq!(apply_pinyin_tone("hao", 3), "hǎo");
    assert_eq!(apply_pinyin_tone("lv", 4), "lǜ");
    assert_eq!(apply_pinyin_tone("xue", 2), "xué");
}
