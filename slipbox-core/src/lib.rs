use std::collections::HashSet;
use std::{fs, path::PathBuf};

use std::time::SystemTime;

use std::io::Read;

use pulldown_cmark::{
    CowStr, Event, MetadataBlockKind, Parser, Tag::MetadataBlock, TextMergeStream,
};

use tracing::info;

mod error;
pub use self::error::{Error, Result};


/// The "atomic" Note is a markdown file that contains the contents which make up the note.
/// By default, each note starts off as a draft and can be set to finished manually. The purpose of
/// this is to allow more visualizations and metrics of the knowledge base.
#[derive(Debug, Clone)]
pub struct Note {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    // pub id: String,
    // pub draft: bool,
    // pub created_on: SystemTime,
    // pub last_edited: SystemTime,
    // pub links: Vec<Note>,
}

const TAG_IDENTIFIER: &str = "tags:";

#[derive(Debug)]
pub struct State {
    pub notes: Vec<Note>, // todo: Ideally we cache notes so that we only re-parse notes that have changed
}

impl State {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut state = Self { notes: vec![] };
        Self::_read_notes(&mut state, path)?;
        Ok(state)
    }

    pub fn tags(&self) -> HashSet<String> {
        let mut tag_set: HashSet<String> = HashSet::new();
        self.notes.clone().into_iter().for_each(|note| {
            note.tags.into_iter().for_each(|tag| {
                tag_set.insert(tag);
            })
        });
        tag_set
    }

    pub fn notes_from_tag(&self, tag: String) -> Vec<Note> {
        self.notes
            .clone()
            .into_iter()
            .filter(|note| {
                note.tags
                    .clone()
                    .into_iter()
                    .any(|tag_string| tag_string == tag)
            })
            .collect::<Vec<_>>()
    }

    fn _read_notes(&mut self, path: PathBuf) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "md" {
                let name = path.file_stem().unwrap().to_str().unwrap();
                info!("found note: {:?}", &name);
                self.notes.push(Note {
                    name: String::from(name),
                    tags: Self::_parse_tags(path.clone())?,
                    path,
                });
            }
        }

        Ok(())
    }

    /// Read the notes and parse out relevant information to build internal data structures.
    fn _parse_tags(note_path: PathBuf) -> Result<Vec<String>> {
        // Setup the markdown parser.
        let mut parser_options = pulldown_cmark::Options::empty();
        parser_options.insert(pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        parser_options.insert(pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

        // Read note contents of note files.
        let mut contents = String::new();
        fs::File::open(&note_path)?.read_to_string(&mut contents)?;

        // Parse markdown from string.
        let events = TextMergeStream::new(Parser::new_ext(&contents, parser_options));

        // Parse out relevant state information.
        let meta_data_predicate = |event: &Event| {
            matches!(
                event,
                Event::Start(MetadataBlock(MetadataBlockKind::PlusesStyle))
                    | Event::Start(MetadataBlock(MetadataBlockKind::YamlStyle))
            )
        };

        let text_event = events.skip_while(meta_data_predicate).next();

        match text_event {
            Some(Event::Text(CowStr::Borrowed(tag_text))) => {
                return Ok(Self::_parse_tag_text(tag_text)?);
            }
            _ => {
                return Err(Error::MetaDataError(format!(
                    "Incorrectly formatted metadata tags or missing entirely."
                )))
            }
        }
    }

    fn _parse_tag_text(tag_text: &str) -> Result<Vec<String>> {
        // Extract only the string of the tag itself
        let raw_tags: Vec<&str> = tag_text
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| s.starts_with(TAG_IDENTIFIER))
            .map(|s| s[TAG_IDENTIFIER.len()..].trim())
            .collect();

        if raw_tags.is_empty() {
            return Err(Error::MetaDataError(format!(
                "Must specify at least one tag."
            )));
        }

        let tag_collections: Vec<String> = raw_tags
            .into_iter()
            .map(|s| {
                s.split_whitespace()
                    .map(|s| s.trim_matches(|c| matches!(c, '[' | ']' | ',' | '\"')))
                    .map(|s| String::from(s))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        Ok(tag_collections)
    }
}

/// The main representation of the application state. This struct contains all necessary
/// internal information necessary for the application to function.
#[derive(Debug)]
pub struct Vault {
    pub vault_path: PathBuf,
    pub name: String,
    pub created_on: Option<SystemTime>,
    pub state: State,
}

impl Vault {
    pub fn new(path: PathBuf) -> Result<Self> {
        let directory_name = match path.file_name() {
            Some(x) => String::from(x.to_str().unwrap()),
            None => return Err(Error::InvalidPath),
        };

        info!("Vault-name: {:?}", directory_name);

        Ok(Self {
            vault_path: path.clone(),
            name: directory_name,
            created_on: None,
            state: State::new(path)?,
        })
    }
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid() -> Result<()> {

        let valid_path: PathBuf = PathBuf::from("./tests/vault");
        let vault = Vault::new(valid_path)?;

        assert!(vault.name == "vault");
        assert!(vault.vault_path == PathBuf::from("./tests/vault"));
        assert!(vault.created_on == None);

        let names = vault
            .state
            .notes
            .into_iter()
            .map(|note| note.name.clone())
            .collect::<Vec<_>>();
        let file_names = vec![
            String::from("TestNote01"),
            String::from("TestNote02"),
            String::from("TestNote03"),
        ];

        assert!(file_names == names);

        Ok(())
    }

    #[test]
    fn test_invalid() -> Result<()> {
        let valid_path: PathBuf =
            PathBuf::from("./tests/invalid-vault");
        let vault = Vault::new(valid_path);

        match vault {
            Err(Error::MetaDataError(_)) => {
                return Ok(())
            },
            _ => panic!("Test should fail"),
        }
    }
}
