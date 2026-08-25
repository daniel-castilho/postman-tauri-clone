// src-tauri/src/infrastructure/docs/markdown_adapter.rs
use crate::application::ports::docs_generator::DocsGeneratorPort;
use crate::domain::models::{Collection, CollectionItem, Body};
use crate::domain::errors::DomainError;

pub struct MarkdownDocsAdapter;

impl MarkdownDocsAdapter {
    pub fn new() -> Self { Self }

    fn process_items(&self, items: &[CollectionItem], level: usize) -> String {
        let mut md = String::new();
        let indent = "#".repeat(level + 1);

        for item in items {
            match item {
                CollectionItem::Request(req) => {
                    md.push_str(&format!("{} {}\n", indent, req.name));
                    if let Some(desc) = &req.description {
                        md.push_str(&format!("_{}_\n\n", desc));
                    }
                    md.push_str(&format!("`{} {}`\n\n", req.method, req.url.0));
                    
                    if !req.headers.is_empty() {
                        md.push_str("#### Headers\n");
                        md.push_str("| Key | Value |\n| --- | --- |\n");
                        for h in &req.headers {
                            if h.enabled {
                                md.push_str(&format!("| `{}` | `{}` |\n", h.key, h.value));
                            }
                        }
                        md.push('\n');
                    }

                    if let Some(body) = &req.body {
                        md.push_str("#### Body\n");
                        match body {
                            Body::Raw(content, _) => {
                                md.push_str(&format!("```json\n{}\n```\n\n", content));
                            },
                            _ => {
                                md.push_str("_Complex Body Type_\n\n");
                            }
                        }
                    }
                    
                    md.push_str("---\n\n");
                },
                CollectionItem::Folder { name, description, items } => {
                    md.push_str(&format!("{} Folder: {}\n\n", indent, name));
                    if let Some(desc) = description {
                        md.push_str(&format!("_{}_\n\n", desc));
                    }
                    md.push_str(&self.process_items(items, level + 1));
                }
            }
        }
        md
    }
}

impl Default for MarkdownDocsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DocsGeneratorPort for MarkdownDocsAdapter {
    fn generate_markdown(&self, collection: &Collection) -> Result<String, DomainError> {
        let mut md = format!("# Collection: {}\n\n", collection.name);
        if let Some(desc) = &collection.description {
            md.push_str(&format!("_{}_\n\n", desc));
        }
        md.push_str("Documentation auto-generated with **Tyny Pulse**.\n\n");
        md.push_str(&self.process_items(&collection.items, 1));
        Ok(md)
    }
}
