use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub(crate) struct RomajiOption {
    pub romaji: String,
    pub priority: u8,
}

impl RomajiOption {
    pub fn new(romaji: &str, priority: u8) -> Self {
        Self {
            romaji: romaji.to_string(),
            priority,
        }
    }

    pub fn from_string(romaji: String, priority: u8) -> Self {
        Self { romaji, priority }
    }
}

pub(crate) static ROMAJI_TABLE: LazyLock<HashMap<&'static str, Vec<RomajiOption>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        m.insert("あ", vec![RomajiOption::new("a", 0)]);
        m.insert("い", vec![RomajiOption::new("i", 0)]);
        m.insert(
            "う",
            vec![
                RomajiOption::new("u", 0),
                RomajiOption::new("wu", 1),
                RomajiOption::new("whu", 2),
            ],
        );
        m.insert("え", vec![RomajiOption::new("e", 0)]);
        m.insert("お", vec![RomajiOption::new("o", 0)]);

        m.insert(
            "か",
            vec![RomajiOption::new("ka", 0), RomajiOption::new("ca", 1)],
        );
        m.insert("き", vec![RomajiOption::new("ki", 0)]);
        m.insert(
            "く",
            vec![
                RomajiOption::new("ku", 0),
                RomajiOption::new("cu", 1),
                RomajiOption::new("qu", 2),
            ],
        );
        m.insert("け", vec![RomajiOption::new("ke", 0)]);
        m.insert(
            "こ",
            vec![RomajiOption::new("ko", 0), RomajiOption::new("co", 1)],
        );

        m.insert("さ", vec![RomajiOption::new("sa", 0)]);
        m.insert(
            "し",
            vec![
                RomajiOption::new("si", 0),
                RomajiOption::new("shi", 1),
                RomajiOption::new("ci", 2),
            ],
        );
        m.insert("す", vec![RomajiOption::new("su", 0)]);
        m.insert(
            "せ",
            vec![RomajiOption::new("se", 0), RomajiOption::new("ce", 1)],
        );
        m.insert("そ", vec![RomajiOption::new("so", 0)]);

        m.insert("た", vec![RomajiOption::new("ta", 0)]);
        m.insert(
            "ち",
            vec![RomajiOption::new("ti", 0), RomajiOption::new("chi", 1)],
        );
        m.insert(
            "つ",
            vec![RomajiOption::new("tu", 0), RomajiOption::new("tsu", 1)],
        );
        m.insert("て", vec![RomajiOption::new("te", 0)]);
        m.insert("と", vec![RomajiOption::new("to", 0)]);

        m.insert("な", vec![RomajiOption::new("na", 0)]);
        m.insert("に", vec![RomajiOption::new("ni", 0)]);
        m.insert("ぬ", vec![RomajiOption::new("nu", 0)]);
        m.insert("ね", vec![RomajiOption::new("ne", 0)]);
        m.insert("の", vec![RomajiOption::new("no", 0)]);

        m.insert("は", vec![RomajiOption::new("ha", 0)]);
        m.insert("ひ", vec![RomajiOption::new("hi", 0)]);
        m.insert(
            "ふ",
            vec![RomajiOption::new("hu", 0), RomajiOption::new("fu", 1)],
        );
        m.insert("へ", vec![RomajiOption::new("he", 0)]);
        m.insert("ほ", vec![RomajiOption::new("ho", 0)]);

        m.insert("ま", vec![RomajiOption::new("ma", 0)]);
        m.insert("み", vec![RomajiOption::new("mi", 0)]);
        m.insert("む", vec![RomajiOption::new("mu", 0)]);
        m.insert("め", vec![RomajiOption::new("me", 0)]);
        m.insert("も", vec![RomajiOption::new("mo", 0)]);

        m.insert("や", vec![RomajiOption::new("ya", 0)]);
        m.insert("ゆ", vec![RomajiOption::new("yu", 0)]);
        m.insert("よ", vec![RomajiOption::new("yo", 0)]);

        m.insert("ら", vec![RomajiOption::new("ra", 0)]);
        m.insert("り", vec![RomajiOption::new("ri", 0)]);
        m.insert("る", vec![RomajiOption::new("ru", 0)]);
        m.insert("れ", vec![RomajiOption::new("re", 0)]);
        m.insert("ろ", vec![RomajiOption::new("ro", 0)]);

        m.insert("わ", vec![RomajiOption::new("wa", 0)]);
        m.insert("ゐ", vec![RomajiOption::new("wyi", 0)]);
        m.insert("ゑ", vec![RomajiOption::new("wye", 0)]);
        m.insert("を", vec![RomajiOption::new("wo", 0)]);
        m.insert(
            "ん",
            vec![
                RomajiOption::new("nn", 1),
                RomajiOption::new("xn", 2),
                RomajiOption::new("n'", 3),
            ],
        ); // "n"での入力は、動的に管理

        m.insert("が", vec![RomajiOption::new("ga", 0)]);
        m.insert("ぎ", vec![RomajiOption::new("gi", 0)]);
        m.insert("ぐ", vec![RomajiOption::new("gu", 0)]);
        m.insert("げ", vec![RomajiOption::new("ge", 0)]);
        m.insert("ご", vec![RomajiOption::new("go", 0)]);

        m.insert("ざ", vec![RomajiOption::new("za", 0)]);
        m.insert(
            "じ",
            vec![RomajiOption::new("zi", 0), RomajiOption::new("ji", 1)],
        );
        m.insert("ず", vec![RomajiOption::new("zu", 0)]);
        m.insert("ぜ", vec![RomajiOption::new("ze", 0)]);
        m.insert("ぞ", vec![RomajiOption::new("zo", 0)]);

        m.insert("だ", vec![RomajiOption::new("da", 0)]);
        m.insert("ぢ", vec![RomajiOption::new("di", 0)]);
        m.insert("づ", vec![RomajiOption::new("du", 0)]);
        m.insert("で", vec![RomajiOption::new("de", 0)]);
        m.insert("ど", vec![RomajiOption::new("do", 0)]);

        m.insert("ば", vec![RomajiOption::new("ba", 0)]);
        m.insert("び", vec![RomajiOption::new("bi", 0)]);
        m.insert("ぶ", vec![RomajiOption::new("bu", 0)]);
        m.insert("べ", vec![RomajiOption::new("be", 0)]);
        m.insert("ぼ", vec![RomajiOption::new("bo", 0)]);

        m.insert("ぱ", vec![RomajiOption::new("pa", 0)]);
        m.insert("ぴ", vec![RomajiOption::new("pi", 0)]);
        m.insert("ぷ", vec![RomajiOption::new("pu", 0)]);
        m.insert("ぺ", vec![RomajiOption::new("pe", 0)]);
        m.insert("ぽ", vec![RomajiOption::new("po", 0)]);

        m.insert("きゃ", vec![RomajiOption::new("kya", 0)]);
        m.insert("きぃ", vec![RomajiOption::new("kyi", 0)]);
        m.insert("きゅ", vec![RomajiOption::new("kyu", 0)]);
        m.insert("きぇ", vec![RomajiOption::new("kye", 0)]);
        m.insert("きょ", vec![RomajiOption::new("kyo", 0)]);

        m.insert("ぎゃ", vec![RomajiOption::new("gya", 0)]);
        m.insert("ぎぃ", vec![RomajiOption::new("gyi", 0)]);
        m.insert("ぎゅ", vec![RomajiOption::new("gyu", 0)]);
        m.insert("ぎぇ", vec![RomajiOption::new("gye", 0)]);
        m.insert("ぎょ", vec![RomajiOption::new("gyo", 0)]);

        m.insert(
            "しゃ",
            vec![RomajiOption::new("sya", 0), RomajiOption::new("sha", 1)],
        );
        m.insert("しぃ", vec![RomajiOption::new("syi", 0)]);
        m.insert(
            "しゅ",
            vec![RomajiOption::new("syu", 0), RomajiOption::new("shu", 1)],
        );
        m.insert(
            "しぇ",
            vec![RomajiOption::new("sye", 0), RomajiOption::new("she", 1)],
        );
        m.insert(
            "しょ",
            vec![RomajiOption::new("syo", 0), RomajiOption::new("sho", 1)],
        );

        m.insert(
            "じゃ",
            vec![
                RomajiOption::new("zya", 0),
                RomajiOption::new("ja", 1),
                RomajiOption::new("jya", 2),
            ],
        );
        m.insert(
            "じぃ",
            vec![RomajiOption::new("zyi", 0), RomajiOption::new("jyi", 1)],
        );
        m.insert(
            "じゅ",
            vec![
                RomajiOption::new("zyu", 0),
                RomajiOption::new("ju", 1),
                RomajiOption::new("jyu", 2),
            ],
        );
        m.insert(
            "じぇ",
            vec![
                RomajiOption::new("zye", 0),
                RomajiOption::new("je", 1),
                RomajiOption::new("jye", 2),
            ],
        );
        m.insert(
            "じょ",
            vec![
                RomajiOption::new("zyo", 0),
                RomajiOption::new("jo", 1),
                RomajiOption::new("jyo", 2),
            ],
        );

        m.insert(
            "ちゃ",
            vec![
                RomajiOption::new("tya", 0),
                RomajiOption::new("cya", 1),
                RomajiOption::new("cha", 2),
            ],
        );
        m.insert(
            "ちぃ",
            vec![RomajiOption::new("tyi", 0), RomajiOption::new("cyi", 1)],
        );
        m.insert(
            "ちゅ",
            vec![
                RomajiOption::new("tyu", 0),
                RomajiOption::new("cyu", 1),
                RomajiOption::new("chu", 2),
            ],
        );
        m.insert(
            "ちぇ",
            vec![
                RomajiOption::new("tye", 0),
                RomajiOption::new("cye", 1),
                RomajiOption::new("che", 2),
            ],
        );
        m.insert(
            "ちょ",
            vec![
                RomajiOption::new("tyo", 0),
                RomajiOption::new("cyo", 1),
                RomajiOption::new("cho", 2),
            ],
        );

        m.insert("ぢゃ", vec![RomajiOption::new("dya", 0)]);
        m.insert("ぢぃ", vec![RomajiOption::new("dyi", 0)]);
        m.insert("ぢゅ", vec![RomajiOption::new("dyu", 0)]);
        m.insert("ぢぇ", vec![RomajiOption::new("dye", 0)]);
        m.insert("ぢょ", vec![RomajiOption::new("dyo", 0)]);

        m.insert("つぁ", vec![RomajiOption::new("tsa", 0)]);
        m.insert("つぃ", vec![RomajiOption::new("tsi", 0)]);
        m.insert("つぇ", vec![RomajiOption::new("tse", 0)]);
        m.insert("つぉ", vec![RomajiOption::new("tso", 0)]);

        m.insert("てゃ", vec![RomajiOption::new("tha", 0)]);
        m.insert(
            "てぃ",
            vec![RomajiOption::new("thi", 0), RomajiOption::new("t'i", 1)],
        );
        m.insert(
            "てゅ",
            vec![RomajiOption::new("thu", 0), RomajiOption::new("t'yu", 1)],
        );
        m.insert("てぇ", vec![RomajiOption::new("the", 0)]);
        m.insert("てょ", vec![RomajiOption::new("tho", 0)]);

        m.insert("でゃ", vec![RomajiOption::new("dha", 0)]);
        m.insert(
            "でぃ",
            vec![RomajiOption::new("dhi", 0), RomajiOption::new("d'i", 1)],
        );
        m.insert(
            "でゅ",
            vec![RomajiOption::new("dhu", 0), RomajiOption::new("d'yu", 1)],
        );
        m.insert("でぇ", vec![RomajiOption::new("dhe", 0)]);
        m.insert("でょ", vec![RomajiOption::new("dho", 0)]);

        m.insert("とぁ", vec![RomajiOption::new("twa", 0)]);
        m.insert("とぃ", vec![RomajiOption::new("twi", 0)]);
        m.insert(
            "とぅ",
            vec![RomajiOption::new("twu", 0), RomajiOption::new("t'u", 1)],
        );
        m.insert("とぇ", vec![RomajiOption::new("twe", 0)]);
        m.insert("とぉ", vec![RomajiOption::new("two", 0)]);

        m.insert("どぁ", vec![RomajiOption::new("dwa", 0)]);
        m.insert("どぃ", vec![RomajiOption::new("dwi", 0)]);
        m.insert(
            "どぅ",
            vec![RomajiOption::new("dwu", 0), RomajiOption::new("d'u", 1)],
        );
        m.insert("どぇ", vec![RomajiOption::new("dwe", 0)]);
        m.insert("どぉ", vec![RomajiOption::new("dwo", 0)]);

        m.insert("にゃ", vec![RomajiOption::new("nya", 0)]);
        m.insert("にぃ", vec![RomajiOption::new("nyi", 0)]);
        m.insert("にゅ", vec![RomajiOption::new("nyu", 0)]);
        m.insert("にぇ", vec![RomajiOption::new("nye", 0)]);
        m.insert("にょ", vec![RomajiOption::new("nyo", 0)]);

        m.insert("ひゃ", vec![RomajiOption::new("hya", 0)]);
        m.insert("ひぃ", vec![RomajiOption::new("hyi", 0)]);
        m.insert("ひゅ", vec![RomajiOption::new("hyu", 0)]);
        m.insert("ひぇ", vec![RomajiOption::new("hye", 0)]);
        m.insert("ひょ", vec![RomajiOption::new("hyo", 0)]);

        m.insert("びゃ", vec![RomajiOption::new("bya", 0)]);
        m.insert("びぃ", vec![RomajiOption::new("byi", 0)]);
        m.insert("びゅ", vec![RomajiOption::new("byu", 0)]);
        m.insert("びぇ", vec![RomajiOption::new("bye", 0)]);
        m.insert("びょ", vec![RomajiOption::new("byo", 0)]);

        m.insert("ぴゃ", vec![RomajiOption::new("pya", 0)]);
        m.insert("ぴぃ", vec![RomajiOption::new("pyi", 0)]);
        m.insert("ぴゅ", vec![RomajiOption::new("pyu", 0)]);
        m.insert("ぴぇ", vec![RomajiOption::new("pye", 0)]);
        m.insert("ぴょ", vec![RomajiOption::new("pyo", 0)]);

        m.insert(
            "ふぁ",
            vec![RomajiOption::new("fa", 0), RomajiOption::new("hwa", 1)],
        );
        m.insert(
            "ふぃ",
            vec![RomajiOption::new("fi", 0), RomajiOption::new("hwi", 1)],
        );
        m.insert(
            "ふぇ",
            vec![RomajiOption::new("fe", 0), RomajiOption::new("hwe", 1)],
        );
        m.insert(
            "ふぉ",
            vec![RomajiOption::new("fo", 0), RomajiOption::new("hwo", 1)],
        );

        m.insert("ふゃ", vec![RomajiOption::new("fya", 0)]);
        m.insert(
            "ふゅ",
            vec![RomajiOption::new("fyu", 0), RomajiOption::new("hwyu", 1)],
        );
        m.insert("ふょ", vec![RomajiOption::new("fyo", 0)]);

        m.insert("みゃ", vec![RomajiOption::new("mya", 0)]);
        m.insert("みぃ", vec![RomajiOption::new("myi", 0)]);
        m.insert("みゅ", vec![RomajiOption::new("myu", 0)]);
        m.insert("みぇ", vec![RomajiOption::new("mye", 0)]);
        m.insert("みょ", vec![RomajiOption::new("myo", 0)]);

        m.insert("りゃ", vec![RomajiOption::new("rya", 0)]);
        m.insert("りぃ", vec![RomajiOption::new("ryi", 0)]);
        m.insert("りゅ", vec![RomajiOption::new("ryu", 0)]);
        m.insert("りぇ", vec![RomajiOption::new("rye", 0)]);
        m.insert("りょ", vec![RomajiOption::new("ryo", 0)]);

        m.insert(
            "ぁ",
            vec![RomajiOption::new("xa", 0), RomajiOption::new("la", 1)],
        );
        m.insert(
            "ぃ",
            vec![
                RomajiOption::new("xi", 0),
                RomajiOption::new("li", 1),
                RomajiOption::new("xyi", 2),
                RomajiOption::new("lyi", 3),
            ],
        );
        m.insert(
            "ぅ",
            vec![RomajiOption::new("xu", 0), RomajiOption::new("lu", 1)],
        );
        m.insert(
            "ぇ",
            vec![
                RomajiOption::new("xe", 0),
                RomajiOption::new("le", 1),
                RomajiOption::new("xye", 2),
                RomajiOption::new("lye", 3),
            ],
        );
        m.insert(
            "ぉ",
            vec![RomajiOption::new("xo", 0), RomajiOption::new("lo", 1)],
        );
        m.insert(
            "ゃ",
            vec![RomajiOption::new("xya", 0), RomajiOption::new("lya", 1)],
        );
        m.insert(
            "ゅ",
            vec![RomajiOption::new("xyu", 0), RomajiOption::new("lyu", 1)],
        );
        m.insert(
            "ょ",
            vec![RomajiOption::new("xyo", 0), RomajiOption::new("lyo", 1)],
        );
        m.insert(
            "ゎ",
            vec![RomajiOption::new("xwa", 0), RomajiOption::new("lwa", 1)],
        );
        m.insert(
            "っ",
            vec![
                RomajiOption::new("xtu", 10),
                RomajiOption::new("ltu", 11),
                RomajiOption::new("xtsu", 12),
                RomajiOption::new("ltsu", 13),
            ],
        ); // 子音重ねは動的に管理

        m.insert("いぇ", vec![RomajiOption::new("ye", 0)]);

        m.insert(
            "くぁ",
            vec![RomajiOption::new("kwa", 0), RomajiOption::new("qa", 1)],
        );
        m.insert(
            "くぃ",
            vec![RomajiOption::new("kwi", 0), RomajiOption::new("qi", 1)],
        );
        m.insert("くぅ", vec![RomajiOption::new("kwu", 0)]);
        m.insert(
            "くぇ",
            vec![RomajiOption::new("kwe", 0), RomajiOption::new("qe", 1)],
        );
        m.insert(
            "くぉ",
            vec![RomajiOption::new("kwo", 0), RomajiOption::new("qo", 1)],
        );

        m.insert("ぐぁ", vec![RomajiOption::new("gwa", 0)]);
        m.insert("ぐぃ", vec![RomajiOption::new("gwi", 0)]);
        m.insert("ぐぅ", vec![RomajiOption::new("gwu", 0)]);
        m.insert("ぐぇ", vec![RomajiOption::new("gwe", 0)]);
        m.insert("ぐぉ", vec![RomajiOption::new("gwo", 0)]);

        m.insert("ゔぁ", vec![RomajiOption::new("va", 0)]);
        m.insert(
            "ゔぃ",
            vec![RomajiOption::new("vi", 0), RomajiOption::new("vyi", 1)],
        );
        m.insert("ゔ", vec![RomajiOption::new("vu", 0)]);
        m.insert(
            "ゔぇ",
            vec![RomajiOption::new("ve", 0), RomajiOption::new("vye", 1)],
        );
        m.insert("ゔぉ", vec![RomajiOption::new("vo", 0)]);

        m.insert("すぁ", vec![RomajiOption::new("swa", 0)]);
        m.insert("すぃ", vec![RomajiOption::new("swi", 0)]);
        m.insert("すぅ", vec![RomajiOption::new("swu", 0)]);
        m.insert("すぇ", vec![RomajiOption::new("swe", 0)]);
        m.insert("すぉ", vec![RomajiOption::new("swo", 0)]);

        m.insert("ずぁ", vec![RomajiOption::new("zwa", 0)]);
        m.insert("ずぃ", vec![RomajiOption::new("zwi", 0)]);
        m.insert("ずぅ", vec![RomajiOption::new("zwu", 0)]);
        m.insert("ずぇ", vec![RomajiOption::new("zwe", 0)]);
        m.insert("ずぉ", vec![RomajiOption::new("zwo", 0)]);

        m.insert("ゔゃ", vec![RomajiOption::new("vya", 0)]);
        m.insert("ゔゅ", vec![RomajiOption::new("vyu", 0)]);
        m.insert("ゔょ", vec![RomajiOption::new("vyo", 0)]);

        m.insert("うぁ", vec![RomajiOption::new("wha", 0)]);
        m.insert(
            "うぃ",
            vec![RomajiOption::new("wi", 0), RomajiOption::new("whi", 1)],
        );
        m.insert(
            "うぇ",
            vec![RomajiOption::new("we", 0), RomajiOption::new("whe", 1)],
        );
        m.insert("うぉ", vec![RomajiOption::new("who", 0)]);

        m
    });

pub(crate) static SYMBOL_TABLE: LazyLock<HashMap<&'static str, Vec<RomajiOption>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        m.insert("ー", vec![RomajiOption::new("-", 0)]);
        m.insert("、", vec![RomajiOption::new(",", 0)]);
        m.insert("。", vec![RomajiOption::new(".", 0)]);
        m.insert("！", vec![RomajiOption::new("!", 0)]);
        m.insert("？", vec![RomajiOption::new("?", 0)]);
        m.insert("：", vec![RomajiOption::new(":", 0)]);
        m.insert("；", vec![RomajiOption::new(";", 0)]);
        m.insert("・", vec![RomajiOption::new("/", 0)]);
        m.insert("〜", vec![RomajiOption::new("~", 0)]);
        m.insert("～", vec![RomajiOption::new("~", 0)]);
        m.insert("（", vec![RomajiOption::new("(", 0)]);
        m.insert("）", vec![RomajiOption::new(")", 0)]);
        m.insert("「", vec![RomajiOption::new("[", 0)]);
        m.insert("」", vec![RomajiOption::new("]", 0)]);
        m.insert("　", vec![RomajiOption::new(" ", 0)]);

        m
    });
