//! In-memory document store for open files.

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;

/// Thread-safe store of open document contents.
pub struct DocumentStore {
	docs: DashMap<Url, String>,
}

impl Default for DocumentStore {
	fn default() -> Self {
		Self {
			docs: DashMap::new(),
		}
	}
}

impl DocumentStore {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn open(&self, uri: Url, text: String) {
		self.docs.insert(uri, text);
	}

	pub fn update(&self, uri: &Url, text: String) {
		self.docs.insert(uri.clone(), text);
	}

	pub fn close(&self, uri: &Url) {
		self.docs.remove(uri);
	}

	pub fn get(&self, uri: &Url) -> Option<String> {
		self.docs.get(uri).map(|r| r.value().clone())
	}

	pub fn all(&self) -> Vec<(Url, String)> {
		self.docs
			.iter()
			.map(|entry| (entry.key().clone(), entry.value().clone()))
			.collect()
	}
}
