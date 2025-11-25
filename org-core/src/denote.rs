//! Denote filename parsing utilities
//!
//! Parses Denote-style filenames: `YYYYMMDDTHHMMSS--title__tags.org`
//!
//! # Examples
//! ```
//! use org_core::denote::DenoteFile;
//!
//! let file = DenoteFile::parse("20231128T233500--org-roam-설정__emacs_org.org");
//! assert!(file.is_some());
//! let file = file.unwrap();
//! assert_eq!(file.identifier, "20231128T233500");
//! assert_eq!(file.title, "org-roam-설정");
//! assert_eq!(file.tags, vec!["emacs", "org"]);
//! ```

use serde::{Deserialize, Serialize};

/// Parsed Denote file information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DenoteFile {
    /// Original filename
    pub filename: String,
    /// Denote identifier (timestamp): YYYYMMDDTHHMMSS
    pub identifier: String,
    /// Title extracted from filename (hyphens replaced with spaces)
    pub title: String,
    /// Tags extracted from filename
    pub tags: Vec<String>,
    /// File extension (without dot)
    pub extension: String,
}

impl DenoteFile {
    /// Parse a Denote-style filename
    ///
    /// Format: `YYYYMMDDTHHMMSS--title__tag1_tag2.ext`
    ///
    /// Returns `None` if the filename doesn't match Denote format
    pub fn parse(filename: &str) -> Option<Self> {
        // Extract base name (without directory path)
        let base_name = std::path::Path::new(filename)
            .file_name()?
            .to_str()?;

        // Must have extension
        let (name_without_ext, extension) = base_name.rsplit_once('.')?;

        // Split by "--" to get identifier and rest
        let (identifier, rest) = name_without_ext.split_once("--")?;

        // Validate identifier format: YYYYMMDDTHHMMSS (15 chars)
        if !is_valid_identifier(identifier) {
            return None;
        }

        // Split rest by "__" to get title and tags
        let (title, tags) = if let Some((t, tag_str)) = rest.rsplit_once("__") {
            let tags: Vec<String> = tag_str
                .split('_')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            (t.to_string(), tags)
        } else {
            // No tags
            (rest.to_string(), Vec::new())
        };

        Some(DenoteFile {
            filename: filename.to_string(),
            identifier: identifier.to_string(),
            title,
            tags,
            extension: extension.to_string(),
        })
    }

    /// Get title with hyphens replaced by spaces
    pub fn title_display(&self) -> String {
        self.title.replace('-', " ")
    }

    /// Check if file has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    /// Check if file has any of the specified tags
    pub fn has_any_tag(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.has_tag(t))
    }

    /// Check if file has all of the specified tags
    pub fn has_all_tags(&self, tags: &[String]) -> bool {
        tags.iter().all(|t| self.has_tag(t))
    }
}

/// Validate Denote identifier format: YYYYMMDDTHHMMSS
fn is_valid_identifier(s: &str) -> bool {
    if s.len() != 15 {
        return false;
    }

    let bytes = s.as_bytes();

    // Check format: 8 digits + 'T' + 6 digits
    bytes[0..8].iter().all(|b| b.is_ascii_digit())
        && bytes[8] == b'T'
        && bytes[9..15].iter().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let file = DenoteFile::parse("20231128T233500--org-roam-설정__emacs_org.org");
        assert!(file.is_some());
        let file = file.unwrap();
        assert_eq!(file.identifier, "20231128T233500");
        assert_eq!(file.title, "org-roam-설정");
        assert_eq!(file.tags, vec!["emacs", "org"]);
        assert_eq!(file.extension, "org");
    }

    #[test]
    fn test_parse_no_tags() {
        let file = DenoteFile::parse("20231128T233500--simple-note.org");
        assert!(file.is_some());
        let file = file.unwrap();
        assert_eq!(file.identifier, "20231128T233500");
        assert_eq!(file.title, "simple-note");
        assert!(file.tags.is_empty());
    }

    #[test]
    fn test_parse_multiple_tags() {
        let file = DenoteFile::parse("20220414T125200--title__tag1_tag2_tag3_tag4.org");
        assert!(file.is_some());
        let file = file.unwrap();
        assert_eq!(file.tags, vec!["tag1", "tag2", "tag3", "tag4"]);
    }

    #[test]
    fn test_parse_with_path() {
        let file = DenoteFile::parse("/home/user/org/notes/20231128T233500--note__tag.org");
        assert!(file.is_some());
        let file = file.unwrap();
        assert_eq!(file.identifier, "20231128T233500");
    }

    #[test]
    fn test_parse_invalid_no_identifier() {
        let file = DenoteFile::parse("some-random-file.org");
        assert!(file.is_none());
    }

    #[test]
    fn test_parse_invalid_short_identifier() {
        let file = DenoteFile::parse("20231128--note.org");
        assert!(file.is_none());
    }

    #[test]
    fn test_parse_invalid_no_extension() {
        let file = DenoteFile::parse("20231128T233500--note");
        assert!(file.is_none());
    }

    #[test]
    fn test_title_display() {
        let file = DenoteFile::parse("20231128T233500--org-roam-설정__emacs.org").unwrap();
        assert_eq!(file.title_display(), "org roam 설정");
    }

    #[test]
    fn test_has_tag() {
        let file = DenoteFile::parse("20231128T233500--note__emacs_org_rust.org").unwrap();
        assert!(file.has_tag("emacs"));
        assert!(file.has_tag("EMACS")); // case insensitive
        assert!(!file.has_tag("vim"));
    }

    #[test]
    fn test_has_any_tag() {
        let file = DenoteFile::parse("20231128T233500--note__emacs_org.org").unwrap();
        assert!(file.has_any_tag(&["vim".to_string(), "emacs".to_string()]));
        assert!(!file.has_any_tag(&["vim".to_string(), "neovim".to_string()]));
    }

    #[test]
    fn test_has_all_tags() {
        let file = DenoteFile::parse("20231128T233500--note__emacs_org_rust.org").unwrap();
        assert!(file.has_all_tags(&["emacs".to_string(), "org".to_string()]));
        assert!(!file.has_all_tags(&["emacs".to_string(), "vim".to_string()]));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("20231128T233500"));
        assert!(is_valid_identifier("20220414T125200"));
        assert!(!is_valid_identifier("2023112T233500")); // too short
        assert!(!is_valid_identifier("20231128X233500")); // wrong separator
        assert!(!is_valid_identifier("2023112833500")); // no T
    }

    #[test]
    fn test_real_world_filenames() {
        // Test with actual filenames from ~/org/notes/
        let cases = vec![
            "20211117T190700--모음-이맥스-팁-트릭__bib_tips_emacs_trick_productivity.org",
            "20220330T140100--힣-ai-시대에-왜-우리-개인은-더-지식에-목마른가__ai_autholog_knowledge_individuation.org",
            "20220906T124100--오그롬-제텔카스텐-에버그린-활용법-위키데이터__evergreen_orgroam_pkm_semantic_wikidata_zettelkasten_orgmode.org",
        ];

        for filename in cases {
            let file = DenoteFile::parse(filename);
            assert!(file.is_some(), "Failed to parse: {}", filename);
        }
    }
}
