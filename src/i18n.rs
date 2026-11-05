use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Chinese, // 默认中文 (zh-CN)
    English, // 英文 (en-US)
}

static CURRENT_LANG: AtomicU8 = AtomicU8::new(0); // 0 = Chinese, 1 = English

impl Language {
    pub fn current() -> Self {
        match CURRENT_LANG.load(Ordering::Relaxed) {
            1 => Language::English,
            _ => Language::Chinese,
        }
    }

    pub fn set(lang: Language) {
        let val = match lang {
            Language::Chinese => 0,
            Language::English => 1,
        };
        CURRENT_LANG.store(val, Ordering::Relaxed);
    }

    pub fn toggle() {
        if Self::current() == Language::Chinese {
            Self::set(Language::English);
        } else {
            Self::set(Language::Chinese);
        }
    }
}

pub fn tr<'a>(zh: &'a str, en: &'a str) -> &'a str {
    match Language::current() {
        Language::Chinese => zh,
        Language::English => en,
    }
}
