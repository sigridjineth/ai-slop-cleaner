use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const BANNED_DENSITY_MAX: f64 = 16.0;
const STRUCTURAL_DENSITY_MAX: f64 = 10.0;
const META_DENSITY_MAX: f64 = 6.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct Components {
    pub BWD: f64,
    pub SPV: f64,
    pub RHY: f64,
    pub META: f64,
    pub MD: f64,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            BWD: 0.0,
            SPV: 0.0,
            RHY: 0.0,
            META: 0.0,
            MD: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub line: Option<usize>,
    pub span: Option<(usize, usize)>,
    pub category: String,
    pub severity: String,
    pub text: String,
    pub context: String,
    pub suggested_fix: String,
    pub detector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Details {
    pub word_count: usize,
    pub sentence_count: usize,
    pub weighted_banned_hits: f64,
    pub weighted_structural_hits: f64,
    pub meta_hits: usize,
    pub markdown_hits: usize,
    pub rhythm_cv: f64,
    pub rhythm_sample_confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct HumanFramework {
    pub H_honest_human_flaws: BTreeMap<String, Value>,
    pub U_unpredictable_structure: BTreeMap<String, Value>,
    pub M_memorable_specifics: BTreeMap<String, Value>,
    pub A_authentic_perspective: BTreeMap<String, Value>,
    pub N_natural_flow: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocPatterns {
    pub avg_sentence_length: f64,
    pub sentence_length_std_dev: f64,
    pub sentence_length_cv: f64,
    pub paragraph_length_cv: f64,
    pub repeated_openers: Vec<String>,
    pub bullet_density: String,
    pub heading_count: usize,
    pub banned_word_count: usize,
    pub human_framework: HumanFramework,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Analysis {
    pub score: u32,
    pub components: Components,
    pub findings: Vec<Finding>,
    pub details: Details,
    pub doc_patterns: DocPatterns,
    pub source: String,
    pub detector: String,
}

#[derive(Debug, Clone)]
struct PreparedDocument {
    clean_text: String,
    prose_lines: Vec<String>,
    sentences: Vec<String>,
    word_count: usize,
    inline_code_spans: usize,
}

#[derive(Debug, Clone)]
pub struct StructuralPattern {
    pub name: &'static str,
    pub severity: &'static str,
    pub weight: f64,
    pub regex: &'static str,
    pub description: &'static str,
}

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|err| panic!("invalid regex {pattern}: {err}"))
}

fn clip01(value: f64) -> f64 {
    if value.is_nan() || value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

pub fn score_from_components(components: &Components) -> u32 {
    let raw = 100.0
        * (0.25 * clip01(components.BWD)
            + 0.25 * clip01(components.SPV)
            + 0.20 * clip01(components.RHY)
            + 0.15 * clip01(components.META)
            + 0.15 * clip01(components.MD));
    raw.round().clamp(0.0, 100.0) as u32
}

pub fn banned_replacements() -> &'static [(&'static str, &'static str)] {
    &[
        ("delve into", "explore"),
        ("delve", "explore"),
        ("elucidate", "explain"),
        ("underscore", "stress"),
        ("harness", "use"),
        ("leverage", "use"),
        ("bolster", "strengthen"),
        ("foster", "encourage"),
        ("showcase", "show"),
        ("streamline", "simplify"),
        ("revolutionize", "transform"),
        ("unveil", "reveal"),
        ("orchestrate", "arrange"),
        ("transcend", "go beyond"),
        ("exemplify", "show"),
        ("augment", "expand"),
        ("surpass", "exceed"),
        ("pinpoint", "identify"),
        ("scrutinize", "examine"),
        ("unravel", "solve"),
        ("embark", "start"),
        ("navigate", "handle"),
        ("elevate", "raise"),
        ("unlock", "open up"),
        ("unleash", "release"),
        ("dive", "look"),
        ("discover", "find"),
        ("craft", "make"),
        ("illuminate", "clarify"),
        ("pivotal", "key"),
        ("meticulous", "careful"),
        ("intricate", "complex"),
        ("transformative", "major"),
        ("groundbreaking", "new"),
        ("unparalleled", "unmatched"),
        ("comprehensive", "thorough"),
        ("robust", "strong"),
        ("crucial", "vital"),
        ("notable", "noteworthy"),
        ("formidable", "impressive"),
        ("nuanced", "subtle"),
        ("multifaceted", "varied"),
        ("paramount", "top"),
        ("instrumental", "helpful"),
        ("foundational", "basic"),
        ("commendable", "admirable"),
        ("cutting-edge", "latest"),
        ("seamless", "smooth"),
        ("vibrant", "lively"),
        ("bustling", "busy"),
        ("holistic", "whole"),
        ("poised", "ready"),
        ("remarkable", "striking"),
        ("realm", "area"),
        ("tapestry", "fabric"),
        ("landscape", "field"),
        ("beacon", "guide"),
        ("hurdles", "obstacles"),
        ("testament", "proof"),
        ("game-changer", "breakthrough"),
        ("journey", "process"),
        ("synergy", "cooperation"),
        ("in today's digital age", ""),
        ("it's important to note", ""),
        ("furthermore", ""),
        ("moreover", ""),
        ("not only this, but also this", ""),
        ("in conclusion", ""),
        ("in closing", ""),
        ("let's dive in", ""),
        ("let's explore", ""),
        ("it's worth noting that", ""),
        ("as we navigate", ""),
        ("in an era of", ""),
        ("it is essential that", "make sure"),
        ("it should be noted", ""),
        ("in order to", "to"),
        ("due to the fact that", "because"),
        ("with regard to", "about"),
        ("in the event that", "if"),
        ("for the purpose of", "to"),
        ("at this point in time", "now"),
        ("in spite of the fact that", "although"),
        ("in the absence of", "without"),
        ("it is evident that", "clearly"),
        ("it is apparent that", "clearly"),
        ("there is a need for", "we need"),
        ("a significant number of", "many"),
        ("a considerable amount of", "much"),
        ("it is interesting to note that", ""),
        ("one could argue that", ""),
        ("it is possible that", ""),
        ("there is a possibility that", ""),
        ("it may be worth considering", ""),
        ("arguably", ""),
        ("potentially", ""),
        ("in some cases", ""),
        ("to a certain extent", ""),
        ("by and large", ""),
        ("pipeline", "flow"),
        ("framework", "system"),
        ("scalable", "able to grow"),
        ("insight", "finding"),
        ("impact", "effect"),
        ("결론적으로", ""),
        ("요약하면", ""),
        ("종합하면", ""),
        ("정리하자면", ""),
        ("시사하는 바가 크다", "의미가 있다"),
        ("주목할 만하다", "눈에 띈다"),
        ("간과할 수 없다", "놓치면 안 된다"),
        ("무시할 수 없다", "작지 않다"),
        ("지평을 연다", "길을 연다"),
        ("방점을 찍는다", "강조한다"),
        ("의미가 적지 않다", "의미가 있다"),
        ("의미심장하다", "의미가 있다"),
        ("혁신적인", "새롭다"),
        ("획기적인", "새롭다"),
        ("전례 없는", "드문"),
        ("압도적", "큰"),
        ("막강한", "강한"),
        ("폭발적", "빠른"),
        ("파격적", "이례적인"),
        ("대대적", "큰 규모의"),
        ("강력한", "강한"),
        ("치열한", "거센"),
        ("뜨거운", "활발한"),
        ("가능성을 열어준다", "가능하게 한다"),
        ("새로운 장을 열다", "새 단계로 넘어가다"),
        ("시대가 도래했다", "시대가 왔다"),
        ("매우", ""),
        ("정말", ""),
        ("진짜로", ""),
        ("대단히", ""),
        ("극히", ""),
    ]
}

pub fn structural_patterns() -> Vec<StructuralPattern> {
    vec![
        StructuralPattern {
            name: "redefinition",
            severity: "medium",
            weight: 2.0,
            regex: r"(?i)(?:\bnot\s+\w+(?:\s+\w+){0,2}\b|rather\s+than\s+\w+(?:\s+\w+){0,2}|instead\s+of\s+\w+(?:\s+\w+){0,2})\s*[,;:]+\s*\b(?:it\s+is|is|are)\b|\bnot\s+\w+(?:\s+\w+){0,2}\s+but\s+\w+",
            description: "A is not X, it is Y / not X but Y redefinition.",
        },
        StructuralPattern {
            name: "closing_summary",
            severity: "low",
            weight: 1.0,
            regex: r"(?i)\b(to summarize|in summary|in conclusion|in closing|in short|put simply|at the end of the day)\b",
            description: "Closing-summary repetition.",
        },
        StructuralPattern {
            name: "progress_announcement",
            severity: "medium",
            weight: 1.0,
            regex: r"(?i)\b(let's\s+(?:take a closer look|unpack(?: this)?|dig deeper|break this down|dive in|explore)|we(?:'ll| will)\s+examine(?: this)?(?: step by step)?|step by step|one by one)\b",
            description: "Progress announcement.",
        },
        StructuralPattern {
            name: "pre_classification",
            severity: "low",
            weight: 1.0,
            regex: r"(?i)\b(?:there are|there is|we can divide|this divides|can be divided)\b[^.!?]{0,80}\b(?:types|branches|categories|groups|kinds)\b",
            description: "Pre-classification framing.",
        },
        StructuralPattern {
            name: "imagine_prompt",
            severity: "low",
            weight: 1.0,
            regex: r"(?i)\b(imagine|picture|visualize)\b[^.!?]{0,80}",
            description: "Imagine/picture/visualize prompt.",
        },
        StructuralPattern {
            name: "broad_field_dump",
            severity: "medium",
            weight: 1.5,
            regex: r"(?is)\b(?:fields|properties|attributes|schema|structure)\b[^.\n]{0,120}:\s*(?:\n\s*(?:[-*+]\s*)?`?[\w-]+`?\s*:[^\n]*){2,}",
            description: "Broad overview followed by a field dump.",
        },
        StructuralPattern {
            name: "colon_enumeration",
            severity: "medium",
            weight: 1.5,
            regex: r"(?i)\b[a-z][\w-]*(?:\s+[a-z][\w-]*){0,6}\s*[:;]\s*`?[\w-]+`?(?:\s*,\s*(?:and\s+)?`?[\w-]+`?){1,}",
            description: "Colon followed by comma-separated items.",
        },
        StructuralPattern {
            name: "a_not_b_redefinition",
            severity: "high",
            weight: 2.5,
            regex: r"(?i)\b(?:is|are|was|were)\s+not\s+[a-z]+(?:\s+[a-z]+){0,3}\s*[,;]\s*(?:it\s+)?(?:is|are|was|were)\b|\bnot\s+[a-z]+(?:\s+[a-z]+){0,3}\s+but\s+(?:is\s+)?[a-z]+",
            description: "A is not X, it is Y redefinition pattern.",
        },
        // 60 Korean im-not-ai/Humanize KR-compatible patterns.
        StructuralPattern {
            name: "ko_A1_about_regarding",
            severity: "high",
            weight: 2.0,
            regex: r"[가-힣A-Za-z0-9]+\s*에\s*대(?:해(?:서)?|하여)",
            description: "Korean '~에 대해/대하여' translationese.",
        },
        StructuralPattern {
            name: "ko_A2_through",
            severity: "high",
            weight: 2.0,
            regex: r"[가-힣A-Za-z0-9]+\s*(?:을|를)\s*통(?:해|하여)",
            description: "Korean '~를 통해/통하여' translationese.",
        },
        StructuralPattern {
            name: "ko_A3_in_terms",
            severity: "high",
            weight: 2.0,
            regex: r"[가-힣A-Za-z0-9]+\s*에\s*있어(?:서)?",
            description: "Korean '~에 있어서' translationese.",
        },
        StructuralPattern {
            name: "ko_A4_point_sense",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:라는|다는)\s+점에서",
            description: "'~라는 점에서' reasoning crutch.",
        },
        StructuralPattern {
            name: "ko_A5_related",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣A-Za-z0-9]+\s*(?:와|과)\s*관련(?:하여|한|된)",
            description: "'~와 관련하여/관련된'.",
        },
        StructuralPattern {
            name: "ko_A6_based_on",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣A-Za-z0-9]+\s*에\s*기반(?:하여|한)|[가-힣A-Za-z0-9]+\s*(?:을|를)\s*바탕으로",
            description: "'~에 기반하여/바탕으로'.",
        },
        StructuralPattern {
            name: "ko_A7_have",
            severity: "high",
            weight: 2.0,
            regex: r"가지고\s+있(?:다|습니다|는)",
            description: "'가지고 있다' possessive translationese.",
        },
        StructuralPattern {
            name: "ko_A8_double_passive",
            severity: "high",
            weight: 2.0,
            regex: r"[가-힣]+(?:되어진|되어진다|지게\s+된|지게\s+된다|보여질\s+수\s+있)",
            description: "double passive.",
        },
        StructuralPattern {
            name: "ko_A9_by_passive",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣A-Za-z0-9]+\s*에\s*의해",
            description: "'~에 의해' by-passive.",
        },
        StructuralPattern {
            name: "ko_A10_can_overuse",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣]+(?:할|될|릴|울|낼|킬|일)\s+수\s+있(?:다|습니다|는|을)",
            description: "Korean can-form '~할 수 있다'.",
        },
        StructuralPattern {
            name: "ko_A11_purpose",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣A-Za-z0-9]+\s*(?:을|를)\s*위해(?:서)?",
            description: "'~을 위해' purpose clause.",
        },
        StructuralPattern {
            name: "ko_A12_automated_passive",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:만들어지(?:다|고|며|는|었다|었습니다)?|이루어지(?:다|고|며|는)?|이루어졌(?:다|습니다)?)",
            description: "'만들어지다/이루어지다'.",
        },
        StructuralPattern {
            name: "ko_A13_noun_stack",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:[가-힣A-Za-z0-9]+\s+){2,}[가-힣A-Za-z0-9]+\s*(?:가속화|고도화|최적화|강화|확대|발전|전환)",
            description: "English-like Korean noun stack.",
        },
        StructuralPattern {
            name: "ko_A14_and_sentence",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)^\s*그리고\b",
            description: "sentence-initial '그리고'.",
        },
        StructuralPattern {
            name: "ko_A15_abstract_subject",
            severity: "medium",
            weight: 1.5,
            regex: r#"[가-힣A-Za-z0-9'"“”\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,50}(?:보여준(?:다|다\.|다는|습니다)|제공(?:한다|합니다)|가져온(?:다|다는)|시사(?:한다|합니다)|흔들고\s+있(?:다|습니다))"#,
            description: "abstract subject plus all-purpose verb.",
        },
        StructuralPattern {
            name: "ko_B1_parenthesized_english",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣]{2,}\s*\([A-Za-z][A-Za-z0-9 ._/-]{1,40}\)",
            description: "Korean term with English parenthetical.",
        },
        StructuralPattern {
            name: "ko_B2_raw_english_term",
            severity: "medium",
            weight: 1.0,
            regex: r"(?i)(?:framework|pipeline|leverage|seamless|robust|scalable|insight|impact|holistic)",
            description: "untranslated English buzzword.",
        },
        StructuralPattern {
            name: "ko_B3_long_english_quote",
            severity: "medium",
            weight: 1.0,
            regex: r#"["“][A-Za-z][^"”\n]{20,}["”]"#,
            description: "long English quote in Korean prose.",
        },
        StructuralPattern {
            name: "ko_B4_known_as",
            severity: "low",
            weight: 0.5,
            regex: r"(?:라고\s+알려진|로\s+일컬어지는)",
            description: "'~라고 알려진/~로 일컬어지는'.",
        },
        StructuralPattern {
            name: "ko_C1_mechanical_enumeration",
            severity: "high",
            weight: 2.0,
            regex: r"(?s)첫째.{0,240}둘째.{0,240}셋째",
            description: "mechanical enumeration.",
        },
        StructuralPattern {
            name: "ko_C2_bullet_block",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)(?:^\s*[-*+]\s+.+\n?){3,}",
            description: "excessive bullet block.",
        },
        StructuralPattern {
            name: "ko_C3_generic_headings",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)^\s{0,3}#{1,6}\s*(?:도입|서론|본론|결론)\b",
            description: "generic headings.",
        },
        StructuralPattern {
            name: "ko_C4_topic_sentence",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)^\s*(?:핵심은|중요한\s+(?:점|것)은|요지는|주목할\s+점은|이\s+문단은)",
            description: "topic sentence formula.",
        },
        StructuralPattern {
            name: "ko_C5_emoji",
            severity: "high",
            weight: 2.0,
            regex: r"[✅🚀💡⚠📊✨🔥👉👍🎯]",
            description: "emoji decoration in prose.",
        },
        StructuralPattern {
            name: "ko_C6_heading_summary",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:이\s+섹션에서는|이\s+장에서는|이번\s+장에서는|이\s+글에서는)\s+[^.!?\n]{0,80}(?:다룬|살펴본|소개한)",
            description: "heading followed by summary.",
        },
        StructuralPattern {
            name: "ko_C7_three_step",
            severity: "medium",
            weight: 1.0,
            regex: r"(?is)먼저.{0,300}반면.{0,300}(?:결국|마지막으로)",
            description: "먼저/반면/결국 formula.",
        },
        StructuralPattern {
            name: "ko_C8_binary_parallel",
            severity: "medium",
            weight: 1.0,
            regex: r#"[가-힣A-Za-z0-9'"“”]+(?:인가|인가요)\s*[,·]?\s*[가-힣A-Za-z0-9'"“”]+(?:인가|인가요)"#,
            description: "A인가, B인가 parallelism.",
        },
        StructuralPattern {
            name: "ko_C9_numeric_index",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:^|\s)(?:1\)|\(1\))[^\n]{0,160}(?:2\)|\(2\))[^\n]{0,160}(?:3\)|\(3\))",
            description: "1) 2) 3) numeric indexing.",
        },
        StructuralPattern {
            name: "ko_C10_colon_heading",
            severity: "high",
            weight: 2.0,
            regex: r"(?m)^\s{0,3}#{1,6}\s*(?:\*\*)?[^:\n]{1,80}(?:\*\*)?\s*:\s+\S.*$",
            description: "colon subtitle heading formula.",
        },
        StructuralPattern {
            name: "ko_D1_conclusion_phrase",
            severity: "high",
            weight: 2.0,
            regex: r"(?:결론적으로|요약하면|종합하면|정리하자면|정리하면|라고\s+할\s+수\s+있다|라고\s+볼\s+수\s+있다|라\s+하겠다|라\s+할\s+것이다|에\s+다름\s+아니다)",
            description: "Korean conclusion/summary formula.",
        },
        StructuralPattern {
            name: "ko_D2_importance_cliche",
            severity: "high",
            weight: 2.0,
            regex: r"(?:시사하는\s+바가\s+크다|주목할\s+만하다|간과할\s+수\s+없다|무시할\s+수\s+없다|지평을\s+연다|방점을\s+찍는다|의미가\s+적지\s+않다|의미심장하다|반드시\s+기억해야\s+한다|매우\s+중요하다)",
            description: "importance cliché.",
        },
        StructuralPattern {
            name: "ko_D3_list_intro",
            severity: "high",
            weight: 2.0,
            regex: r"(?:크게\s+세\s+가지로\s+나눌\s+수\s+있다|다음과\s+같은\s+특징을\s+가진다|다음과\s+같이\s+요약할\s+수\s+있다)",
            description: "list-introduction cliché.",
        },
        StructuralPattern {
            name: "ko_D4_hype",
            severity: "high",
            weight: 2.0,
            regex: r"(?:혁신적인|획기적인|전례\s+없는|압도적|막강한|폭발적|파격적|대대적|강력한|치열한|뜨거운|가능성을\s+열어준다|새로운\s+장을\s+열(?:다|었다)|시대가\s+도래했다)",
            description: "hype vocabulary.",
        },
        StructuralPattern {
            name: "ko_D5_personified_abstract",
            severity: "medium",
            weight: 1.5,
            regex: r#"[가-힣A-Za-z0-9'"“”\s]{1,40}(?:은|는|이|가)\s+[^.!?\n]{0,40}(?:질문을\s+던집니다|질문을\s+던진다|끝나지\s+않습니다|증명합니다|부른다|요구한다)"#,
            description: "personified abstract subject.",
        },
        StructuralPattern {
            name: "ko_D6_closing_formula",
            severity: "medium",
            weight: 1.5,
            regex: r"(?:해야\s+할\s+때(?:입니다|다)|나아갈\s+시점(?:입니다|이다)|할\s+순간(?:입니다|이다)|지금이야말로[^.!?\n]{0,60}(?:때|시점))",
            description: "formulaic closing.",
        },
        StructuralPattern {
            name: "ko_D7_transformation_formula",
            severity: "medium",
            weight: 1.0,
            regex: r#"[가-힣A-Za-z0-9'"“”\s]{1,40}(?:에서|을\s+넘어|를\s+넘어)\s+[가-힣A-Za-z0-9'"“”\s]{1,40}(?:로|으로)"#,
            description: "X에서 Y로 transformation slogan.",
        },
        StructuralPattern {
            name: "ko_E1_sentence_length_uniformity",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:[가-힣A-Za-z0-9 ,]{8,36}[.!?]\s*){5,}",
            description: "uniform Korean sentence lengths; also reflected in RHY.",
        },
        StructuralPattern {
            name: "ko_E2_repeated_sentence_endings",
            severity: "medium",
            weight: 1.0,
            regex: r"(?s)(?:것이다|합니다|된다|있다)[.!?]?\s*(?:[^.!?\n]{0,50}(?:것이다|합니다|된다|있다)[.!?]?\s*){2,}",
            description: "repeated Korean sentence endings; also reflected in RHY.",
        },
        StructuralPattern {
            name: "ko_E3_uniform_paragraph_blocks",
            severity: "low",
            weight: 0.5,
            regex: r"(?m)^[^\n]{20,80}$\n\n^[^\n]{20,80}$\n\n^[^\n]{20,80}$",
            description: "uniform paragraph block lengths; also reflected in RHY.",
        },
        StructuralPattern {
            name: "ko_F1_degree_adverb",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:매우|정말|진짜로|대단히|극히)\s+[가-힣]+",
            description: "excessive degree adverb.",
        },
        StructuralPattern {
            name: "ko_F2_double_modifier",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:중요하고\s+핵심적인|새롭고\s+혁신적인|지속적이고\s+꾸준한|[가-힣]+고\s+[가-힣]+적인\s+(?:역할|접근|노력|변화))",
            description: "synonym double modifier.",
        },
        StructuralPattern {
            name: "ko_F3_role_function",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:역할과\s+기능|의미와\s+가치)",
            description: "role/function doublet.",
        },
        StructuralPattern {
            name: "ko_F4_suffix_abuse",
            severity: "low",
            weight: 0.5,
            regex: r"(?:[가-힣]+적\s+(?:측면|관점)|[가-힣]+성\b|[가-힣]+화\b)",
            description: "abstract suffix overuse.",
        },
        StructuralPattern {
            name: "ko_F5_jeok_chain",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣]+적\s+[가-힣]+",
            description: "~적 N abstract chain.",
        },
        StructuralPattern {
            name: "ko_G1_hedging",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:할\s+수\s+있을\s+것으로\s+보인다|인\s+것으로\s+판단된다|라고\s+여겨진다|인\s+듯하다|것으로\s+보인다)",
            description: "Korean hedge ending.",
        },
        StructuralPattern {
            name: "ko_G2_double_hedge",
            severity: "medium",
            weight: 1.5,
            regex: r"(?:가능성이\s+있을\s+수\s+있다|보여질\s+수\s+있다|할\s+수\s+있을\s+것으로\s+보일\s+수\s+있다)",
            description: "double/triple hedge.",
        },
        StructuralPattern {
            name: "ko_H1_connector",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)^\s*(?:또한|따라서|즉|나아가|아울러|게다가|더욱이)\b",
            description: "sentence-initial connector.",
        },
        StructuralPattern {
            name: "ko_H2_contrast_connector",
            severity: "medium",
            weight: 1.0,
            regex: r"(?m)^\s*(?:하지만|그러나)\b",
            description: "contrast connector.",
        },
        StructuralPattern {
            name: "ko_H3_meta_entry",
            severity: "high",
            weight: 1.5,
            regex: r"(?:이는\s+[^.!?\n]{0,60}의미한다|이\s+점에서|이\s+관점에서\s+보면|이\s+말은)",
            description: "이는/이 점에서 meta-entry.",
        },
        StructuralPattern {
            name: "ko_H4_ie",
            severity: "medium",
            weight: 0.5,
            regex: r"즉",
            description: "redefining connector 즉.",
        },
        StructuralPattern {
            name: "ko_I1_geotida",
            severity: "high",
            weight: 2.0,
            regex: r"[가-힣]+(?:인|한|일|다는)\s+것이다",
            description: "~것이다 formal ending.",
        },
        StructuralPattern {
            name: "ko_I2_dependent_noun",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:라는\s+점(?:에\s+있다|에서)?|다는\s+점(?:에\s+있다|에서)?|주목할\s+점은|나아갈\s+바|할\s+수(?:가)?\s+있|하는\s+데(?:에)?)",
            description: "dependent-noun crutch.",
        },
        StructuralPattern {
            name: "ko_I3_means_ending",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:라는\s+것|다는\s+것이다|다는\s+뜻이다|다는\s+의미다|다는\s+점이다)",
            description: "~다는 뜻/의미/점 ending.",
        },
        StructuralPattern {
            name: "ko_I4_need_to",
            severity: "medium",
            weight: 1.0,
            regex: r"(?:할\s+필요가\s+있다|[가-힣]*해야\s+(?:한다|합니다)|[가-힣]+야\s+(?:한다|합니다))",
            description: "~필요가 있다/해야 한다.",
        },
        StructuralPattern {
            name: "ko_I5_needed",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣]+(?:이|가)\s+필요하다",
            description: "abstract need formula.",
        },
        StructuralPattern {
            name: "ko_I6_ability_noun",
            severity: "medium",
            weight: 1.0,
            regex: r"[가-힣A-Za-z0-9]+\s+능력",
            description: "N 능력 ability noun.",
        },
        StructuralPattern {
            name: "ko_J1_bold",
            severity: "medium",
            weight: 0.5,
            regex: r"(?:\*\*|__)[^*_\n]{1,80}(?:\*\*|__)",
            description: "bold emphasis decoration.",
        },
        StructuralPattern {
            name: "ko_J2_quote_emphasis",
            severity: "medium",
            weight: 0.5,
            regex: r#"["'“‘][가-힣A-Za-z0-9\s]{1,30}["'”’]"#,
            description: "quote emphasis decoration.",
        },
        StructuralPattern {
            name: "ko_J3_em_dash",
            severity: "low",
            weight: 0.5,
            regex: r"—",
            description: "em dash decoration.",
        },
        StructuralPattern {
            name: "ko_J4_parenthetical_aside",
            severity: "medium",
            weight: 1.0,
            regex: r"\([^)]{0,80}(?:의미한다|뜻이다|시사한다|말한다)[^)]{0,80}\)",
            description: "parenthetical explanatory aside.",
        },
    ]
}

fn word_tokens(text: &str) -> Vec<String> {
    re(r"[\p{L}\p{N}_]+(?:['’][\p{L}\p{N}_]+)*")
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in re(r"\n\s*\n+").split(text) {
        let mut start = 0;
        for mat in re(r"[.!?]+").find_iter(paragraph) {
            let part = paragraph[start..mat.start()].trim();
            if !part.is_empty() {
                out.push(part.to_string());
            }
            start = mat.end();
        }
        let tail = paragraph[start..].trim();
        if !tail.is_empty() {
            out.push(tail.to_string());
        }
    }
    out
}

fn prepare_document(text: &str) -> PreparedDocument {
    let mut in_fence = false;
    let mut clean_lines = Vec::new();
    let mut prose_lines = Vec::new();
    let mut inline_code_spans = 0;
    let inline_re = re(r"`[^`\n]*`");
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        inline_code_spans += inline_re.find_iter(line).count();
        let stripped = inline_re.replace_all(line, " ").to_string();
        clean_lines.push(stripped.clone());
        if !stripped.trim().is_empty() {
            prose_lines.push(stripped);
        }
    }
    let clean_text = clean_lines.join("\n");
    let sentences = split_sentences(&clean_text);
    let word_count = word_tokens(&clean_text).len().max(1);
    PreparedDocument {
        clean_text,
        prose_lines,
        sentences,
        word_count,
        inline_code_spans,
    }
}

fn line_for_span(text: &str, start: usize) -> usize {
    text[..start.min(text.len())]
        .chars()
        .filter(|&ch| ch == '\n')
        .count()
        + 1
}

fn line_context(text: &str, line: usize) -> String {
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn entry_regex(entry: &str) -> Regex {
    let escaped = regex::escape(entry);
    if entry.chars().any(|ch| ch.is_ascii_alphanumeric()) {
        re(&format!(
            r"(?i)(?:^|[^\p{{L}}\p{{N}}_-])({escaped})(?:$|[^\p{{L}}\p{{N}}_-])"
        ))
    } else {
        re(&escaped)
    }
}

fn banned_word_density(doc: &PreparedDocument) -> (f64, f64, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut weighted = 0.0;
    let mut entries: Vec<_> = banned_replacements().iter().collect();
    entries.sort_by_key(|(entry, _)| std::cmp::Reverse(entry.len()));
    for (entry, replacement) in entries {
        let regex = entry_regex(entry);
        for mat in regex.find_iter(&doc.clean_text) {
            let matched = mat
                .as_str()
                .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '가');
            if matched.is_empty() {
                continue;
            }
            let is_phrase = entry.contains(' ');
            weighted += if is_phrase { 2.0 } else { 1.0 };
            let line = line_for_span(&doc.clean_text, mat.start());
            findings.push(Finding {
                line: Some(line),
                span: Some((mat.start(), mat.end())),
                category: if is_phrase {
                    "banned_phrase"
                } else {
                    "banned_word"
                }
                .to_string(),
                severity: "high".to_string(),
                text: matched.to_string(),
                context: line_context(&doc.clean_text, line),
                suggested_fix: replacement.to_string(),
                detector: "rust".to_string(),
            });
        }
    }
    let density = 1000.0 * weighted / doc.word_count as f64;
    (clip01(density / BANNED_DENSITY_MAX), weighted, findings)
}

fn structural_subscore(doc: &PreparedDocument) -> (f64, f64, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut weighted = 0.0;
    for pattern in structural_patterns() {
        let regex = re(pattern.regex);
        for mat in regex.find_iter(&doc.clean_text) {
            weighted += pattern.weight;
            let line = line_for_span(&doc.clean_text, mat.start());
            findings.push(Finding {
                line: Some(line),
                span: Some((mat.start(), mat.end())),
                category: "structural_pattern".to_string(),
                severity: pattern.severity.to_string(),
                text: mat.as_str().trim().to_string(),
                context: line_context(&doc.clean_text, line),
                suggested_fix: format!("{}: {}", pattern.name, pattern.description),
                detector: "rust".to_string(),
            });
        }
    }
    let density = 1000.0 * weighted / doc.word_count as f64;
    (clip01(density / STRUCTURAL_DENSITY_MAX), weighted, findings)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn stddev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let avg = mean(values);
    (values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64).sqrt()
}

fn rhythm_subscore(doc: &PreparedDocument) -> (f64, f64, String, Vec<Finding>) {
    let mut findings = Vec::new();
    if doc.sentences.len() < 5 {
        return (0.0, 0.0, "low".to_string(), findings);
    }
    let lengths: Vec<f64> = doc
        .sentences
        .iter()
        .map(|sentence| word_tokens(sentence).len() as f64)
        .filter(|len| *len > 0.0)
        .collect();
    if lengths.len() < 5 {
        return (0.0, 0.0, "low".to_string(), findings);
    }
    let avg = mean(&lengths);
    let cv = stddev(&lengths) / avg.max(1.0);
    let cv_penalty = clip01((0.55 - cv) / 0.35);
    let mut opener_counts: HashMap<String, usize> = HashMap::new();
    let mut ending_counts: HashMap<String, usize> = HashMap::new();
    for sentence in &doc.sentences {
        let tokens = word_tokens(&sentence.to_lowercase());
        if let Some(first) = tokens.first() {
            *opener_counts.entry(first.clone()).or_default() += 1;
        }
        if let Some(last) = tokens.last() {
            *ending_counts.entry(last.clone()).or_default() += 1;
        }
    }
    let opener_repeated = opener_counts
        .values()
        .filter(|&&count| count >= 3)
        .sum::<usize>() as f64
        / doc.sentences.len() as f64;
    let ending_repeated = ending_counts
        .values()
        .filter(|&&count| count >= 2)
        .sum::<usize>() as f64
        / doc.sentences.len() as f64;
    let opener_penalty = clip01((opener_repeated - 0.15) / 0.25);
    let ending_penalty = clip01((ending_repeated - 0.15) / 0.25);
    let rhy = 0.55 * cv_penalty + 0.25 * opener_penalty + 0.20 * ending_penalty;
    if rhy >= 0.25 {
        findings.push(Finding {
            line: None,
            span: None,
            category: "rhythm_issue".to_string(),
            severity: "medium".to_string(),
            text: "sentence rhythm is unusually uniform".to_string(),
            context: String::new(),
            suggested_fix: "Mix short, medium, and longer sentences.".to_string(),
            detector: "rust".to_string(),
        });
    }
    (clip01(rhy), cv, "normal".to_string(), findings)
}

fn meta_subscore(doc: &PreparedDocument) -> (f64, usize, Vec<Finding>) {
    let patterns = [
        r"(?i)\bin this section\b",
        r"(?i)\blet's\s+(?:dive in|explore|unpack(?: this)?|break this down|take a closer look|dig deeper)\b",
        r"(?i)\bhere's what you need to know\b",
        r"(?i)\bthe key takeaway(?: here)? is\b",
        r"(?i)\bto summarize\b",
        r"(?i)\bin summary\b",
        r"(?i)\bin conclusion\b",
        r"(?i)\bit is important to note\b",
        r"(?i)\bit's important to note\b",
        r"(?i)\bit should be noted\b",
        r"(?i)\bit's worth noting that\b",
        r"(?i)\bby now you should\b",
        r"(?i)\bwe(?:'ll| will)\s+examine\b",
        r"(?i)\b(?:step by step|one by one)\b",
        r"(?:이\s+섹션에서는|이\s+장에서는|이번\s+장에서는|이\s+글에서는|정리하면|요약하면|종합하면|정리하자면|결론적으로|핵심은|다시\s+말해|다음과\s+같은|다음과\s+같이)",
    ];
    let mut findings = Vec::new();
    let mut total = 0;
    for pattern in patterns {
        let regex = re(pattern);
        for mat in regex.find_iter(&doc.clean_text) {
            total += 1;
            let line = line_for_span(&doc.clean_text, mat.start());
            findings.push(Finding {
                line: Some(line),
                span: Some((mat.start(), mat.end())),
                category: "meta_commentary".to_string(),
                severity: "medium".to_string(),
                text: mat.as_str().to_string(),
                context: line_context(&doc.clean_text, line),
                suggested_fix: "Delete the meta commentary or replace it with the actual point."
                    .to_string(),
                detector: "rust".to_string(),
            });
        }
    }
    let density = 1000.0 * total as f64 / doc.word_count as f64;
    (clip01(density / META_DENSITY_MAX), total, findings)
}

fn markdown_subscore(doc: &PreparedDocument) -> (f64, usize, Vec<Finding>) {
    let bullet = re(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)");
    let heading = re(r"^\s{0,3}#{1,6}\s+");
    let table = re(r"^\s*\|.+\|\s*$");
    let emphasis =
        re(r#"(?:\*\*|__)[^*_][^\n]*?(?:\*\*|__)|`[^`\n]*`|["'“‘][가-힣A-Za-z0-9\s]{1,30}["'”’]"#);
    let mut bullet_lines = 0usize;
    let mut heading_lines = 0usize;
    let mut table_lines = 0usize;
    let mut emphasis_spans = doc.inline_code_spans;
    let mut findings = Vec::new();
    for (idx, line) in doc.prose_lines.iter().enumerate() {
        let mut hits = 0;
        if bullet.is_match(line) {
            bullet_lines += 1;
            hits += 1;
        }
        if heading.is_match(line) {
            heading_lines += 1;
            hits += 1;
        }
        if table.is_match(line) {
            table_lines += 1;
            hits += 1;
        }
        let emph = emphasis.find_iter(line).count();
        emphasis_spans += emph;
        hits += emph;
        if hits > 0 {
            findings.push(Finding {
                line: Some(idx + 1),
                span: None,
                category: "markdown_overuse".to_string(),
                severity: "low".to_string(),
                text: line.trim().to_string(),
                context: line.trim().to_string(),
                suggested_fix: "Use formatting only when it helps the reader.".to_string(),
                detector: "rust".to_string(),
            });
        }
    }
    emphasis_spans += doc.clean_text.matches('—').count();
    let line_count = doc.prose_lines.len().max(1) as f64;
    let bullet_penalty = clip01((bullet_lines as f64 / line_count - 0.12) / 0.28);
    let heading_penalty = clip01((heading_lines as f64 / line_count - 0.08) / 0.17);
    let table_penalty = clip01((table_lines as f64 / line_count) / 0.20);
    let emphasis_density = 1000.0 * emphasis_spans as f64 / doc.word_count as f64;
    let emphasis_penalty = clip01((emphasis_density - 3.0) / 17.0);
    let md = 0.40 * bullet_penalty
        + 0.20 * heading_penalty
        + 0.20 * table_penalty
        + 0.20 * emphasis_penalty;
    (
        clip01(md),
        bullet_lines + heading_lines + table_lines + emphasis_spans,
        findings,
    )
}

fn doc_patterns(text: &str, doc: &PreparedDocument) -> DocPatterns {
    let lengths: Vec<f64> = doc
        .sentences
        .iter()
        .map(|sentence| word_tokens(sentence).len() as f64)
        .collect();
    let avg_len = mean(&lengths);
    let sd = stddev(&lengths);
    let sentence_cv = if avg_len > 0.0 {
        sd / avg_len.max(1.0)
    } else {
        0.0
    };
    let paragraph_lengths: Vec<f64> = re(r"\n\s*\n+")
        .split(&doc.clean_text)
        .filter(|p| !p.trim().is_empty())
        .map(|p| word_tokens(p).len() as f64)
        .collect();
    let paragraph_cv = if paragraph_lengths.len() > 1 {
        stddev(&paragraph_lengths) / mean(&paragraph_lengths).max(1.0)
    } else {
        0.0
    };
    let mut opener_counts: HashMap<String, usize> = HashMap::new();
    for sentence in &doc.sentences {
        if let Some(first) = word_tokens(&sentence.to_lowercase()).first()
            && first.len() > 3
        {
            *opener_counts.entry(first.clone()).or_default() += 1;
        }
    }
    let repeated_openers = opener_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|(word, count)| format!("{word} ({count}x)"))
        .collect();
    let bullet_count = text
        .lines()
        .filter(|line| re(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)").is_match(line))
        .count();
    let line_count = text.lines().count().max(1);
    let ratio = bullet_count as f64 / line_count as f64;
    let bullet_density = if ratio > 0.3 {
        "high"
    } else if ratio > 0.15 {
        "medium"
    } else {
        "low"
    }
    .to_string();
    let lower = doc.clean_text.to_lowercase();
    let banned_word_count = banned_replacements()
        .iter()
        .filter(|(entry, _)| !entry.contains(' '))
        .map(|(entry, _)| lower.matches(entry).count())
        .sum();

    let honest = re(r"(?i)\b(?:honestly|to be fair|frankly)\b|솔직히|솔직히\s+말하면|까놓고\s+말하면|인정하자면|사실은").find_iter(&doc.clean_text).count();
    let specific = re(r"(?i)\b(?:for example|for instance|e\.g\.)\b|예를\s+들어|예컨대|실제로|\d+(?:[.,]\d+)*(?:%|년|월|일|원|명|개|배|초|분|시간)?").find_iter(&doc.clean_text).count();
    let perspective = re(r"(?i)\b(?:in my experience|my view|I think|I've found|I’ve found)\b|제\s+경험(?:으로는|상)|개인적으로|제가\s+보기(?:엔|에는)|나는|저는").find_iter(&doc.clean_text).count();
    let flow =
        re(r"(?i)\b(?:but|so|still|anyway|that said)\b|그런데|그래도|다만|그래서|어쨌든|한편")
            .find_iter(&doc.clean_text)
            .count();
    let mut h = BTreeMap::new();
    h.insert("marker_count".into(), json!(honest));
    h.insert(
        "examples".into(),
        json!(["솔직히 말하면", "to be fair", "honestly"]),
    );
    let mut u = BTreeMap::new();
    u.insert("sentence_length_cv".into(), json!(round4(sentence_cv)));
    u.insert("paragraph_length_cv".into(), json!(round4(paragraph_cv)));
    let mut m = BTreeMap::new();
    m.insert("specific_marker_count".into(), json!(specific));
    m.insert(
        "examples".into(),
        json!(["numbers/dates", "예를 들어", "for example"]),
    );
    let mut a = BTreeMap::new();
    a.insert("marker_count".into(), json!(perspective));
    a.insert(
        "examples".into(),
        json!(["제 경험으로는", "개인적으로", "in my experience"]),
    );
    let mut n = BTreeMap::new();
    n.insert("conversational_connector_count".into(), json!(flow));
    DocPatterns {
        avg_sentence_length: round4(avg_len),
        sentence_length_std_dev: round4(sd),
        sentence_length_cv: round4(sentence_cv),
        paragraph_length_cv: round4(paragraph_cv),
        repeated_openers,
        bullet_density,
        heading_count: text
            .lines()
            .filter(|line| re(r"^\s{0,3}#{1,6}\s+").is_match(line))
            .count(),
        banned_word_count,
        human_framework: HumanFramework {
            H_honest_human_flaws: h,
            U_unpredictable_structure: u,
            M_memorable_specifics: m,
            A_authentic_perspective: a,
            N_natural_flow: n,
        },
    }
}

pub fn analyze_text(text: &str, source: &str) -> Analysis {
    let doc = prepare_document(text);
    let (bwd, weighted_banned_hits, mut banned_findings) = banned_word_density(&doc);
    let (spv, weighted_structural_hits, mut structural_findings) = structural_subscore(&doc);
    let (rhy, rhythm_cv, rhythm_confidence, mut rhythm_findings) = rhythm_subscore(&doc);
    let (meta, meta_hits, mut meta_findings) = meta_subscore(&doc);
    let (md, markdown_hits, mut markdown_findings) = markdown_subscore(&doc);
    let components = Components {
        BWD: round4(bwd),
        SPV: round4(spv),
        RHY: round4(rhy),
        META: round4(meta),
        MD: round4(md),
    };
    let mut findings = Vec::new();
    findings.append(&mut banned_findings);
    findings.append(&mut structural_findings);
    findings.append(&mut rhythm_findings);
    findings.append(&mut meta_findings);
    findings.append(&mut markdown_findings);
    findings.sort_by_key(|f| {
        (
            f.line.unwrap_or(usize::MAX),
            f.category.clone(),
            f.text.clone(),
        )
    });
    let score = score_from_components(&components);
    Analysis {
        score,
        components,
        findings,
        details: Details {
            word_count: doc.word_count,
            sentence_count: doc.sentences.len(),
            weighted_banned_hits,
            weighted_structural_hits: round4(weighted_structural_hits),
            meta_hits,
            markdown_hits,
            rhythm_cv: round4(rhythm_cv),
            rhythm_sample_confidence: rhythm_confidence,
        },
        doc_patterns: doc_patterns(text, &doc),
        source: source.to_string(),
        detector: "rust".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeFinding {
    pub category: String,
    pub smell: String,
    pub severity: String,
    pub text: String,
    pub line: Option<usize>,
    pub symbol: String,
    pub source: String,
    pub suggested_fix: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeSmellReport {
    pub rules: Value,
    pub files: Vec<String>,
    pub findings: Vec<CodeFinding>,
    pub summary: BTreeMap<String, Value>,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    line: usize,
    body: String,
}

fn cleanup_rules() -> Value {
    json!({
        "behavior_lock": "Lock behavior with regression tests before cleanup edits.",
        "cleanup_plan": "Create a bounded cleanup plan before changing code.",
        "issue_categories": ["Duplication", "Dead code", "Needless abstraction", "Boundary violations", "Missing tests"],
        "pass_order": ["Dead code", "Duplicate", "Naming/error", "Test reinforcement"],
        "quality_gates": ["regression tests", "lint", "typecheck", "tests", "static/security scan", "evidence-dense report"]
    })
}

fn detect_language(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "py" => "python",
        "js" | "jsx" | "ts" | "tsx" => "javascript",
        "rs" => "rust",
        _ => "text",
    }
}

fn matching_brace(text: &str, open: usize) -> usize {
    let mut depth = 0i32;
    for (offset, ch) in text[open..].char_indices() {
        if ch == '{' {
            depth += 1;
        }
        if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return open + offset;
            }
        }
    }
    text.len()
}

fn extract_functions(source: &str, language: &str) -> Vec<FunctionInfo> {
    let pattern = match language {
        "python" => r"(?m)^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\):",
        "rust" => r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)[^{;]*\{",
        "javascript" => {
            r"(?:\bfunction\s+([A-Za-z_$][\w$]*)\s*\([^)]*\)\s*\{|\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*\([^)]*\)\s*=>\s*\{)"
        }
        _ => return Vec::new(),
    };
    let regex = re(pattern);
    let mut functions = Vec::new();
    for mat in regex.captures_iter(source) {
        let whole = mat.get(0).unwrap();
        let name = mat
            .get(1)
            .or_else(|| mat.get(2))
            .unwrap()
            .as_str()
            .to_string();
        let line = line_for_span(source, whole.start());
        let body = if language == "python" {
            let rest = &source[whole.end()..];
            rest.lines()
                .take_while(|line| {
                    line.trim().is_empty() || line.starts_with(' ') || line.starts_with('\t')
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            let open = source[whole.start()..]
                .find('{')
                .map(|idx| whole.start() + idx)
                .unwrap_or(whole.end());
            let close = matching_brace(source, open);
            source[open + 1..close.min(source.len())].to_string()
        };
        functions.push(FunctionInfo { name, line, body });
    }
    functions
}

fn normalize_body(body: &str) -> String {
    let no_comments = re(r"(?m)#.*$|//.*$").replace_all(body, "").to_string();
    re(r"[A-Za-z_][A-Za-z0-9_]*")
        .replace_all(&no_comments, "id")
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

pub fn analyze_code_text(
    source: &str,
    language: &str,
    source_name: &str,
    tests_text: Option<&str>,
) -> Vec<CodeFinding> {
    let functions = extract_functions(source, language);
    let mut findings = Vec::new();
    let mut by_name: HashMap<String, Vec<&FunctionInfo>> = HashMap::new();
    for function in &functions {
        by_name
            .entry(function.name.clone())
            .or_default()
            .push(function);
    }
    for (name, defs) in by_name {
        if defs.len() > 1 {
            let first = normalize_body(&defs[0].body);
            for other in defs.iter().skip(1) {
                if first == normalize_body(&other.body)
                    || first.len().abs_diff(normalize_body(&other.body).len()) < 12
                {
                    findings.push(CodeFinding {
                        category: "Duplication".into(),
                        smell: "duplicate_function".into(),
                        severity: "high".into(),
                        text: format!("duplicate function definition '{name}' has similar body"),
                        line: Some(other.line),
                        symbol: name.clone(),
                        source: source_name.into(),
                        suggested_fix: "Keep one implementation or extract tested shared logic."
                            .into(),
                        evidence: format!("first definition line {}", defs[0].line),
                    });
                }
            }
        }
    }
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") || trimmed.starts_with("use ") {
            let token = trimmed
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim_end_matches(';')
                .trim_matches('{')
                .to_string();
            let last = token
                .split(['.', ':', '/'])
                .next_back()
                .unwrap_or(&token)
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !last.is_empty() && source.matches(last).count() <= 1 {
                findings.push(CodeFinding {
                    category: "Dead code".into(),
                    smell: "unused_import".into(),
                    severity: "medium".into(),
                    text: format!("unused import '{token}'"),
                    line: Some(idx + 1),
                    symbol: last.into(),
                    source: source_name.into(),
                    suggested_fix: "Delete the import after tests prove it is unused.".into(),
                    evidence: String::new(),
                });
            }
        }
        if (trimmed.starts_with("return")
            || trimmed.starts_with("throw")
            || trimmed.starts_with("panic!"))
            && let Some(next) = source.lines().nth(idx + 1)
            && !next.trim().is_empty()
            && !next.trim().starts_with('}')
        {
            findings.push(CodeFinding {
                category: "Dead code".into(),
                smell: "unreachable_branch".into(),
                severity: "high".into(),
                text: "unreachable statement after terminal control flow".into(),
                line: Some(idx + 2),
                symbol: String::new(),
                source: source_name.into(),
                suggested_fix: "Delete or move the branch.".into(),
                evidence: format!("terminal line {}", idx + 1),
            });
        }
    }
    for function in &functions {
        let body = function.body.trim();
        if re(r"(?s)^\s*return\s+[A-Za-z_][A-Za-z0-9_$.]*\([^;\n]*\)\s*;?\s*$|^\s*[A-Za-z_][A-Za-z0-9_$.]*\([^;\n]*\)\s*$").is_match(body) {
            findings.push(CodeFinding { category: "Needless abstraction".into(), smell: "pass_through_wrapper".into(), severity: "medium".into(), text: format!("pass-through wrapper '{}' only delegates", function.name), line: Some(function.line), symbol: function.name.clone(), source: source_name.into(), suggested_fix: "Inline unless it owns validation or a stable boundary.".into(), evidence: String::new() });
        }
        let call_pat = re(&format!(r"\b{}\s*\(", regex::escape(&function.name)));
        if !function.name.starts_with('_') && call_pat.find_iter(source).count() == 2 {
            findings.push(CodeFinding {
                category: "Needless abstraction".into(),
                smell: "single_use_helper".into(),
                severity: "low".into(),
                text: format!("single-use helper '{}' is called once", function.name),
                line: Some(function.line),
                symbol: function.name.clone(),
                source: source_name.into(),
                suggested_fix: "Inline if the name does not clarify a separate concept.".into(),
                evidence: String::new(),
            });
        }
    }
    let normalized_path = source_name.replace('\\', "/").to_lowercase();
    let ui_layer = normalized_path.contains("/ui/")
        || normalized_path.contains("app/ui/")
        || normalized_path.contains("/view");
    if ui_layer {
        for (idx, line) in source.lines().enumerate() {
            let lower = line.to_lowercase();
            if (lower.contains("import") || lower.contains("use "))
                && [
                    ".data",
                    ".db",
                    ".repo",
                    ".repository",
                    ".persistence",
                    "sqlalchemy",
                ]
                .iter()
                .any(|token| lower.contains(token))
            {
                findings.push(CodeFinding {
                    category: "Boundary violations".into(),
                    smell: "wrong_layer_import".into(),
                    severity: "high".into(),
                    text: "UI layer imports persistence/data module".into(),
                    line: Some(idx + 1),
                    symbol: String::new(),
                    source: source_name.into(),
                    suggested_fix: "Move persistence behind an application/service boundary."
                        .into(),
                    evidence: line.trim().into(),
                });
            }
        }
    }
    if let Some(tests) = tests_text {
        for function in &functions {
            if !function.name.starts_with('_')
                && !function.name.starts_with("test_")
                && function.name != "main"
                && !tests.contains(&function.name)
                && !tests.contains(&format!("test_{}", function.name))
            {
                findings.push(CodeFinding {
                    category: "Missing tests".into(),
                    smell: "missing_function_test".into(),
                    severity: "medium".into(),
                    text: format!("function '{}' has no obvious test coverage", function.name),
                    line: Some(function.line),
                    symbol: function.name.clone(),
                    source: source_name.into(),
                    suggested_fix: "Add/link a regression test before cleanup.".into(),
                    evidence: String::new(),
                });
            }
        }
    }
    findings.sort_by_key(|f| {
        (
            f.category.clone(),
            f.line.unwrap_or(usize::MAX),
            f.text.clone(),
        )
    });
    findings
}

fn collect_code_files(paths: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if matches!(detect_language(path), "python" | "javascript" | "rust") {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let p = entry.path();
                if p.is_dir() {
                    files.extend(collect_code_files(&[p])?);
                } else if matches!(detect_language(&p), "python" | "javascript" | "rust") {
                    files.push(p);
                }
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn read_tests(path: Option<&Path>) -> io::Result<Option<String>> {
    let Some(path) = path else {
        return Ok(None);
    };
    if path.is_file() {
        return fs::read_to_string(path).map(Some);
    }
    let files = collect_code_files(&[path.to_path_buf()])?;
    let mut text = String::new();
    for file in files {
        text.push_str(&fs::read_to_string(file)?);
        text.push('\n');
    }
    Ok(Some(text))
}

pub fn analyze_code_paths(
    paths: &[PathBuf],
    tests_path: Option<&Path>,
) -> io::Result<CodeSmellReport> {
    let files = collect_code_files(paths)?;
    let tests_text = read_tests(tests_path)?;
    let mut findings = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file)?;
        findings.extend(analyze_code_text(
            &source,
            detect_language(file),
            &file.to_string_lossy(),
            tests_text.as_deref(),
        ));
    }
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_severity: BTreeMap<String, usize> = BTreeMap::new();
    for finding in &findings {
        *by_category.entry(finding.category.clone()).or_default() += 1;
        *by_severity.entry(finding.severity.clone()).or_default() += 1;
    }
    let mut summary = BTreeMap::new();
    summary.insert("total".into(), json!(findings.len()));
    summary.insert("files_analyzed".into(), json!(files.len()));
    summary.insert("by_category".into(), json!(by_category));
    summary.insert("by_severity".into(), json!(by_severity));
    summary.insert(
        "pass_order".into(),
        json!([
            "Dead code",
            "Duplicate",
            "Naming/error",
            "Test reinforcement"
        ]),
    );
    Ok(CodeSmellReport {
        rules: cleanup_rules(),
        files: files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        findings,
        summary,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RalphIteration {
    pub iteration: usize,
    pub score: u32,
    pub top_findings: Vec<Finding>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RalphResult {
    pub status: String,
    pub message: String,
    pub final_score: u32,
    pub threshold: u32,
    pub iterations: Vec<RalphIteration>,
    pub output: Option<String>,
    pub text: String,
}

fn top_findings(analysis: &Analysis) -> Vec<Finding> {
    let mut findings = analysis.findings.clone();
    findings.sort_by_key(|f| {
        (
            std::cmp::Reverse(match f.severity.as_str() {
                "critical" => 4,
                "high" => 3,
                "medium" => 2,
                _ => 1,
            }),
            f.line.unwrap_or(usize::MAX),
        )
    });
    findings.into_iter().take(3).collect()
}

pub fn replace_banned_words(text: &str) -> String {
    let mut entries: Vec<_> = banned_replacements().iter().collect();
    entries.sort_by_key(|(entry, _)| std::cmp::Reverse(entry.len()));
    let mut cleaned = text.to_string();
    for (entry, replacement) in entries {
        cleaned = entry_regex(entry)
            .replace_all(&cleaned, *replacement)
            .to_string();
    }
    re(r"[ \t]+([,.!?;:])")
        .replace_all(&cleaned, "$1")
        .to_string()
}

pub fn remove_markdown_decoration(text: &str) -> String {
    let cleaned = re(r"(?:\*\*|__)([^\n]*?)(?:\*\*|__)")
        .replace_all(text, "$1")
        .to_string();
    let cleaned = re(r"(?:\*\*|__)+").replace_all(&cleaned, "").to_string();
    cleaned
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if re(r"^(?:---+|\*\*\*+|___+)$").is_match(trimmed) {
                None
            } else {
                Some(re(r"^\s{0,3}#{1,6}\s+").replace(line, "").to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn simplify_translationese(text: &str) -> String {
    let replacements = [
        (r"에\s*있어(?:서)?", "에서"),
        (r"에\s*대(?:해(?:서)?|하여)", "을"),
        (r"(?:을|를)\s*통(?:해|하여)", "로"),
        (r"매우\s+", ""),
    ];
    let mut cleaned = text.to_string();
    for (pattern, replacement) in replacements {
        cleaned = re(pattern).replace_all(&cleaned, replacement).to_string();
    }
    re(r" {2,}").replace_all(&cleaned, " ").to_string()
}

pub fn break_repetitive_sentence_structures(text: &str) -> String {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    for sentence in split_sentences_keep_punctuation(text) {
        let tokens = word_tokens(sentence);
        if let Some(first) = tokens.first() {
            let count = seen.entry(first.to_lowercase()).or_default();
            *count += 1;
            if *count >= 3 && ["This", "The", "It", "We", "You"].contains(&first.as_str()) {
                out.push(sentence[first.len()..].trim_start().to_string());
                continue;
            }
        }
        out.push(sentence.to_string());
    }
    out.join(" ")
}

fn split_sentences_keep_punctuation(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if matches!(ch, '.' | '!' | '?') {
            let mut end = idx + ch.len_utf8();
            while let Some((next_idx, next_ch)) = chars.peek().copied() {
                if next_ch.is_whitespace() {
                    end = next_idx + next_ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            let part = text[start..end].trim();
            if !part.is_empty() {
                parts.push(part);
            }
            start = end;
        }
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
}

pub fn clean_text_once(text: &str) -> String {
    let cleaned = replace_banned_words(text);
    let cleaned = break_repetitive_sentence_structures(&cleaned);
    let cleaned = remove_markdown_decoration(&cleaned);
    simplify_translationese(&cleaned).trim().to_string()
}

pub fn run_ralph(
    file: &Path,
    threshold: u32,
    max_iterations: usize,
    output: Option<&Path>,
    overwrite: bool,
) -> io::Result<RalphResult> {
    let mut text = fs::read_to_string(file)?;
    let mut iterations = Vec::new();
    let mut written_to = None;
    if max_iterations == 0 {
        let analysis = analyze_text(&text, &file.to_string_lossy());
        return Ok(RalphResult {
            status: "max_iterations_reached".into(),
            message: "max iterations reached before cleanup".into(),
            final_score: analysis.score,
            threshold,
            iterations,
            output: None,
            text,
        });
    }
    for iteration in 1..=max_iterations {
        let analysis = analyze_text(&text, &file.to_string_lossy());
        if analysis.score <= threshold {
            iterations.push(RalphIteration {
                iteration,
                score: analysis.score,
                top_findings: top_findings(&analysis),
                changed: false,
            });
            return Ok(RalphResult {
                status: "success".into(),
                message: format!("score {} is <= threshold {threshold}", analysis.score),
                final_score: analysis.score,
                threshold,
                iterations,
                output: written_to,
                text,
            });
        }
        let top = top_findings(&analysis);
        let cleaned = clean_text_once(&text);
        let changed = cleaned != text;
        iterations.push(RalphIteration {
            iteration,
            score: analysis.score,
            top_findings: top,
            changed,
        });
        if !changed {
            return Ok(RalphResult {
                status: "stalled".into(),
                message: "no rule-based cleanup changed the text".into(),
                final_score: analysis.score,
                threshold,
                iterations,
                output: written_to,
                text,
            });
        }
        text = cleaned;
        if let Some(out) = output {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(out, &text)?;
            written_to = Some(out.to_string_lossy().to_string());
        } else if overwrite {
            fs::write(file, &text)?;
            written_to = Some(file.to_string_lossy().to_string());
        }
    }
    let final_score = analyze_text(&text, output.unwrap_or(file).to_string_lossy().as_ref()).score;
    Ok(RalphResult {
        status: "max_iterations_reached".into(),
        message: format!("max iterations reached with score {final_score}"),
        final_score,
        threshold,
        iterations,
        output: written_to,
        text,
    })
}

pub async fn run_mcp_stdio() -> io::Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    while let Some(message) = read_mcp_message(&mut reader).await? {
        let Some(response) = handle_mcp_message(message) else {
            continue;
        };
        write_mcp_message(&mut stdout, &response).await?;
    }
    Ok(())
}

async fn read_mcp_message(reader: &mut BufReader<tokio::io::Stdin>) -> io::Result<Option<Value>> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line).await?;
    if bytes == 0 {
        return Ok(None);
    }
    if line.trim_start().starts_with('{') {
        return Ok(serde_json::from_str(line.trim()).ok());
    }
    let mut content_length = None;
    loop {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse::<usize>().ok();
        }
        if trimmed.is_empty() {
            break;
        }
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }
    }
    let Some(len) = content_length else {
        return Ok(None);
    };
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(serde_json::from_slice(&buf).ok())
}

async fn write_mcp_message(stdout: &mut tokio::io::Stdout, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    stdout
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await?;
    stdout.write_all(&body).await?;
    stdout.flush().await
}

pub fn handle_mcp_message(message: Value) -> Option<Value> {
    let id = message.get("id").cloned().unwrap_or(Value::Null);
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");
    if method.starts_with("notifications/") {
        return None;
    }
    let result = match method {
        "initialize" => {
            json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"ai-slop-cleaner-rs","version":env!("CARGO_PKG_VERSION")}})
        }
        "tools/list" => json!({"tools":[
            {"name":"ai_slop_score","description":"Return the AI Slop Score.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"file":{"type":"string"}}}},
            {"name":"ai_slop_analyze","description":"Return full AI-slop analysis.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"file":{"type":"string"}}}},
            {"name":"ai_slop_check","description":"Return pass/fail against a threshold.","inputSchema":{"type":"object","properties":{"text":{"type":"string"},"file":{"type":"string"},"threshold":{"type":"integer"}}}}
        ]}),
        "tools/call" => {
            let params = message.get("params").cloned().unwrap_or_default();
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_default();
            match call_tool(name, &args) {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":serde_json::to_string_pretty(&value).unwrap()}],"isError":false})
                }
                Err(err) => json!({"content":[{"type":"text","text":err}],"isError":true}),
            }
        }
        "shutdown" => Value::Null,
        _ => {
            return Some(
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"method not found"}}),
            );
        }
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}

fn text_from_tool_args(args: &Value) -> Result<(String, String), String> {
    if let Some(text) = args.get("text").and_then(Value::as_str) {
        return Ok((text.to_string(), "inline text".into()));
    }
    if let Some(file) = args.get("file").and_then(Value::as_str) {
        return fs::read_to_string(file)
            .map(|text| (text, file.to_string()))
            .map_err(|err| err.to_string());
    }
    Err("tool requires text or file".into())
}

fn call_tool(name: &str, args: &Value) -> Result<Value, String> {
    let (text, source) = text_from_tool_args(args)?;
    let analysis = analyze_text(&text, &source);
    match name {
        "ai_slop_score" => Ok(
            json!({"score":analysis.score,"components":analysis.components,"details":analysis.details,"detector":analysis.detector,"source":source}),
        ),
        "ai_slop_analyze" => serde_json::to_value(analysis).map_err(|err| err.to_string()),
        "ai_slop_check" => {
            let threshold = args.get("threshold").and_then(Value::as_u64).unwrap_or(25) as u32;
            let pass = analysis.score <= threshold;
            Ok(
                json!({"status": if pass {"pass"} else {"fail"}, "pass":pass, "score":analysis.score, "threshold":threshold, "components":analysis.components, "detector":analysis.detector}),
            )
        }
        _ => Err(format!("unknown tool {name}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_formula_matches_python_contract() {
        let components = Components {
            BWD: 0.50,
            SPV: 0.30,
            RHY: 0.40,
            META: 0.50,
            MD: 0.25,
        };
        assert_eq!(score_from_components(&components), 39);
    }

    #[test]
    fn korean_pattern_port_has_sixty_rules_and_matches_example() {
        let names: Vec<_> = structural_patterns()
            .into_iter()
            .filter(|p| p.name.starts_with("ko_"))
            .map(|p| p.name)
            .collect();
        assert_eq!(names.len(), 60);
        let analysis = analyze_text("AI 규제에 대해 논의할 필요가 있다.", "inline");
        assert!(
            analysis
                .findings
                .iter()
                .any(|f| f.suggested_fix.contains("ko_A1_about_regarding"))
        );
    }

    #[test]
    fn sloppy_prose_scores_high() {
        let text = "## Robust Landscape\n**In this section** we delve into a comprehensive and transformative landscape.\nThis chapter highlights pivotal choices. This chapter highlights seamless workflows. This chapter highlights nuanced outcomes.\nThe result is not a helper, it is a game-changer.\n- First, orchestrate the journey.\n- Second, navigate the hurdles.\n- Third, elucidate the tapestry.";
        assert!(analyze_text(text, "inline").score >= 45);
    }

    #[test]
    fn code_smells_find_duplicate_and_missing_tests() {
        let source =
            "import os\n\ndef demo(x):\n    return str(x)\n\ndef demo(x):\n    return str(x)\n";
        let findings = analyze_code_text(source, "python", "app/ui/view.py", Some(""));
        assert!(findings.iter().any(|f| f.category == "Duplication"));
        assert!(findings.iter().any(|f| f.category == "Missing tests"));
        assert!(findings.iter().any(|f| f.text.contains("unused import")));
    }

    #[test]
    fn ralph_cleanup_replaces_banned_words_and_markdown() {
        let cleaned = clean_text_once(
            "**In conclusion**, let's delve into the comprehensive landscape.\n## Title\n이 문제에 있어서 매우 중요하다.",
        );
        assert!(!cleaned.contains("**"));
        assert!(!cleaned.contains("delve into"));
        assert!(!cleaned.contains("에 있어서"));
    }

    #[test]
    fn mcp_initialize_response_is_available() {
        let response =
            handle_mcp_message(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}))
                .unwrap();
        assert_eq!(
            response["result"]["serverInfo"]["name"],
            "ai-slop-cleaner-rs"
        );
    }
}
