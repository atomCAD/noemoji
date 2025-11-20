// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Unicode compliance linter that removes obvious AI authorship signatures
//!
//! ## Why This Tool Exists
//!
//! LLMs have a distinctive and off-putting tendency to overuse Unicode characters - emojis,
//! fancy arrows, decorative bullets, and other visual flourishes that human developers rarely
//! use. This creates an immediate "uncanny valley" effect that signals AI-generated content.
//!
//! While using AI tools for development is perfectly valid, these Unicode signatures can:
//! - Make developers immediately distrust the code or documentation
//! - Create a jarring, unprofessional appearance
//! - Signal low-quality or unreviewed AI output
//! - Distract from the actual content
//!
//! This tool enforces ASCII-preferred character rules to ensure AI-assisted code maintains
//! the clean, professional appearance that developers expect from human-written content.
//!
//! ## Character Rules
//!
//! ### NEVER ALLOWED (clear AI tells to avoid)
//!
//! - **Emojis**: No emoji characters anywhere
//! - **Arrows**: Use `->`, `<-`, `=>` instead of →, ←, ⇒, ↑, ↓
//! - **Checkmarks/Crosses**: Use `[x]` and `[ ]` instead of ✓, ✔, ✗, ✘
//! - **Box Drawing**: Use ASCII art (`+--`, `|`, etc.) instead of ┌, ─, │, └
//! - **Math Symbols**: Use `<=`, `>=`, `!=` instead of ≤, ≥, ≠
//! - **Superscripts/Subscripts**: Use `^2`, `_1` notation instead of ²³⁴, ₁₂₃
//! - **Fractions**: Use `1/2`, `3/4` instead of ½, ¾
//! - **Decorative**: No stars, bullets, or shapes like ★, ●, ♦
//! - **Lookalike Punctuation**: Use ASCII apostrophes and quotes, not Unicode variants
//! - **Special Spaces**: Only regular ASCII spaces, not non-breaking or other Unicode spaces
//!
//! ### ALLOWED EXCEPTIONS
//!
//! - **International Content**: Non-ASCII required for international text (café, 世界, Москва)
//! - **Legal/Formal Symbols**: © (copyright), ® (registered trademark), ™ (trademark), ℠ (service mark), § (section sign), ¶ (pilcrow), † (dagger), ‡ (double dagger) - These have specific legal or formal meanings in professional contexts
//! - **Currency Symbols**: All Unicode currency symbols (¢, £, ¥, €, ₹, ₽, ₩, etc.) - Unicode category `CurrencySymbol`
//! - **Technical/Scientific Symbols**: ° (degree), ∞ (infinity) - Used for measurements, tolerances, and technical specifications
//!
//! ### Guiding Principle
//!
//! Good documentation looks like a human wrote it. When in doubt, use ASCII.

// EOF
