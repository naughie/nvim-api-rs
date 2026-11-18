use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::sync::{Arc, OnceLock};

const MAX_ENTRIES: usize = 512;
const MAX_TEXT_BYTES: usize = 64;
const SAMPLE: u16 = 128;
const MIN_HITS: u16 = 96;
const BYPASS: u16 = 2048;

/// Reuses short cell strings with one bounded, direct-mapped table per model.
#[derive(Default)]
pub(super) struct Cache {
    slots: Vec<Option<Arc<str>>>,
    reuse: Reuse,
}

enum Reuse {
    Sample { remaining: u16, hits: u16 },
    Bypass(u16),
}

impl Default for Reuse {
    fn default() -> Self {
        Self::Sample {
            remaining: SAMPLE,
            hits: 0,
        }
    }
}

impl Cache {
    pub(super) fn intern(&mut self, text: &str) -> Arc<str> {
        if let Some(text) = ascii_text(text) {
            return text;
        }
        if text.len() > MAX_TEXT_BYTES {
            return Arc::from(text);
        }
        let (remaining, hits) = match &mut self.reuse {
            Reuse::Bypass(remaining) => {
                self.reuse = remaining
                    .checked_sub(1)
                    .filter(|&n| n > 0)
                    .map(Reuse::Bypass)
                    .unwrap_or_default();
                return Arc::from(text);
            }
            Reuse::Sample { remaining, hits } => (remaining, hits),
        };
        if self.slots.is_empty() {
            self.slots.resize_with(MAX_ENTRIES, || None);
        }
        let mut hash = DefaultHasher::new();
        text.hash(&mut hash);
        // MAX_ENTRIES is nonzero, and initialization sets exactly that many
        // slots. Modulo therefore selects an in-bounds slot on every call.
        let slot = &mut self.slots[hash.finish() as usize % MAX_ENTRIES];
        // Replacing the cache's reference leaves cells and retained frames valid.
        let (text, hit) = match slot {
            Some(cached) if cached.as_ref() == text => (Arc::clone(cached), true),
            slot => {
                let text: Arc<str> = Arc::from(text);
                *slot = Some(Arc::clone(&text));
                (text, false)
            }
        };
        // A sample starts with remaining = SAMPLE and hits = 0. Each lookup
        // consumes one position, so 1 <= remaining <= SAMPLE and
        // hits <= SAMPLE - remaining. Both operations therefore fit u16.
        *remaining -= 1;
        *hits += u16::from(hit);
        if *remaining == 0 {
            self.reuse = if *hits >= MIN_HITS {
                Reuse::default()
            } else {
                Reuse::Bypass(BYPASS)
            };
        }
        text
    }
}

pub(super) fn shared(text: &str) -> Arc<str> {
    ascii_text(text).unwrap_or_else(|| Arc::from(text))
}

fn ascii_text(text: &str) -> Option<Arc<str>> {
    if text.is_empty() {
        static EMPTY: OnceLock<Arc<str>> = OnceLock::new();
        return Some(Arc::clone(EMPTY.get_or_init(|| Arc::from(""))));
    }
    let [byte] = text.as_bytes() else {
        return None;
    };
    if !byte.is_ascii() {
        return None;
    }
    static ASCII: OnceLock<[Arc<str>; 128]> = OnceLock::new();
    let chars = ASCII.get_or_init(|| {
        std::array::from_fn(|n| {
            let c = [n as u8];
            // from_fn supplies 0 <= n < 128, so c is one valid ASCII byte.
            Arc::from(std::str::from_utf8(&c).expect("ASCII is valid UTF-8"))
        })
    });
    // is_ascii() proves byte < 128, the fixed array's length.
    Some(Arc::clone(&chars[usize::from(*byte)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_long_strings_do_not_populate_the_cache() {
        let mut cache = Cache::default();
        assert!(Arc::ptr_eq(&cache.intern(""), &shared("")));
        for byte in 0..=127 {
            let bytes = [byte];
            let text = std::str::from_utf8(&bytes).unwrap();
            assert!(Arc::ptr_eq(&cache.intern(text), &cache.intern(text)));
            assert!(Arc::ptr_eq(&cache.intern(text), &shared(text)));
        }
        let text = "字".repeat(MAX_TEXT_BYTES);
        let first = cache.intern(&text);
        let second = cache.intern(&text);
        assert_eq!(&*first, text);
        assert_eq!(&*second, text);
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(cache.slots.is_empty());
        assert_eq!(cache.slots.capacity(), 0);
    }

    #[test]
    fn collisions_keep_external_references_valid() {
        let mut cache = Cache::default();
        let first = cache.intern("字");
        assert!(Arc::ptr_eq(&first, &cache.intern("字")));
        let retained: Vec<_> = (0..=MAX_ENTRIES)
            .map(|n| {
                let text = format!("字{n}");
                cache.reuse = Reuse::default();
                let shared = cache.intern(&text);
                (text, shared)
            })
            .collect();
        // MAX_ENTRIES + 1 distinct strings cannot all occupy MAX_ENTRIES slots,
        // regardless of the hash implementation or collision distribution.
        let (text, first) = retained
            .into_iter()
            .find(|(_, shared)| Arc::strong_count(shared) == 1)
            .expect("at least one string was evicted");
        let weak = Arc::downgrade(&first);
        assert_eq!(&*first, text);
        assert!(!Arc::ptr_eq(&first, &cache.intern(&text)));
        drop(first);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn low_reuse_bypasses_lookup_then_probes_again() {
        let mut cache = Cache::default();
        for n in 0..SAMPLE {
            cache.intern(&format!("字{n}"));
        }
        assert!(matches!(cache.reuse, Reuse::Bypass(BYPASS)));
        let cached = Arc::clone(cache.slots.iter().flatten().next().unwrap());
        cache.intern("x");
        cache.intern(&"x".repeat(MAX_TEXT_BYTES + 1));
        assert!(matches!(cache.reuse, Reuse::Bypass(BYPASS)));
        for _ in 0..BYPASS {
            let text = cache.intern(&cached);
            assert_eq!(&*text, &*cached);
            assert!(!Arc::ptr_eq(&text, &cached));
        }
        for _ in 0..SAMPLE {
            assert!(Arc::ptr_eq(&cache.intern(&cached), &cached));
        }
        assert!(matches!(
            cache.reuse,
            Reuse::Sample {
                remaining: SAMPLE,
                hits: 0
            }
        ));
    }

    #[test]
    fn sample_hit_threshold_controls_bypass() {
        for (hits, bypass) in [(MIN_HITS - 1, true), (MIN_HITS, false)] {
            let mut cache = Cache::default();
            let cached = cache.intern("字");
            cache.reuse = Reuse::default();
            for _ in 0..hits {
                assert!(Arc::ptr_eq(&cache.intern("字"), &cached));
            }
            for n in hits..SAMPLE {
                cache.intern(&format!("字{n}"));
            }
            assert_eq!(
                matches!(cache.reuse, Reuse::Bypass(_)),
                bypass,
                "{hits} hits"
            );
        }
    }

    #[test]
    fn retained_entries_and_bytes_stay_bounded() {
        let mut cache = Cache::default();
        for n in 0..MAX_ENTRIES * 16 {
            let text = format!("{n:0>width$}", width = MAX_TEXT_BYTES);
            assert_eq!(&*cache.intern(&text), text);
            let entries = cache.slots.iter().flatten().count();
            let bytes: usize = cache.slots.iter().flatten().map(|s| s.len()).sum();
            assert!(entries <= MAX_ENTRIES);
            assert!(bytes <= 32 * 1024);
            assert_eq!(cache.slots.len(), MAX_ENTRIES);
        }
        let long = "x".repeat(MAX_TEXT_BYTES + 1);
        let first = cache.intern(&long);
        assert_eq!(&*first, long);
        assert!(!Arc::ptr_eq(&first, &cache.intern(&long)));
    }
}
