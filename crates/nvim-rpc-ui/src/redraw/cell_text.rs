use super::decode::ascii_text;

use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::sync::Arc;

const MAX_ENTRIES: usize = 512;
const MAX_TEXT_BYTES: usize = 64;
const SAMPLE: u16 = 128;
const MIN_HITS: u16 = 96;
const BYPASS: u16 = 2048;

/// Reuses short cell strings within one model.
///
/// Each hash selects one slot; a collision replaces its cache reference without
/// affecting cells or retained frames. Lookup/replacement visits one slot even
/// under collisions. The lazy table retains at most 512 strings of 64 bytes
/// (32 KiB of text), plus slot storage and Arc headers. Empty and single-byte
/// ASCII text bypasses the table. Low-reuse samples temporarily bypass lookup.
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

impl Reuse {
    fn bypass(&mut self) -> bool {
        let Self::Bypass(remaining) = self else {
            return false;
        };
        if let Some(n) = remaining.checked_sub(1).filter(|&n| n > 0) {
            *remaining = n;
        } else {
            *self = Self::default();
        }
        true
    }

    fn observe(&mut self, hit: bool) {
        let Self::Sample { remaining, hits } = self else {
            return;
        };
        // Samples start at SAMPLE and reset on the lookup that reaches zero.
        // Thus remaining is in 1..=SAMPLE and hits <= SAMPLE - remaining;
        // incrementing hits and decrementing remaining both fit u16.
        *hits += u16::from(hit);
        *remaining -= 1;
        if *remaining == 0 {
            *self = if *hits >= MIN_HITS {
                Self::default()
            } else {
                Self::Bypass(BYPASS)
            };
        }
    }
}

impl Cache {
    pub(super) fn intern(&mut self, text: &str) -> Arc<str> {
        if let Some(text) = ascii_text(text) {
            return text;
        }
        if text.len() > MAX_TEXT_BYTES || self.reuse.bypass() {
            return Arc::from(text);
        }
        if self.slots.is_empty() {
            self.slots.resize_with(MAX_ENTRIES, || None);
        }
        let mut hash = DefaultHasher::new();
        text.hash(&mut hash);
        // MAX_ENTRIES is nonzero, and initialization sets exactly that many
        // slots. Modulo therefore selects an in-bounds slot on every call.
        let slot = &mut self.slots[hash.finish() as usize % MAX_ENTRIES];
        if let Some(cached) = slot
            && cached.as_ref() == text
        {
            self.reuse.observe(true);
            return Arc::clone(cached);
        }
        self.reuse.observe(false);
        let text: Arc<str> = Arc::from(text);
        *slot = Some(Arc::clone(&text));
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_long_strings_do_not_populate_the_cache() {
        let mut cache = Cache::default();
        for text in ["", "x", "\0", "\u{7f}"] {
            assert!(Arc::ptr_eq(&cache.intern(text), &cache.intern(text)));
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
            let mut reuse = Reuse::default();
            for n in 0..SAMPLE {
                assert!(!reuse.bypass());
                reuse.observe(n < hits);
            }
            assert_eq!(reuse.bypass(), bypass, "{hits} hits");
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
