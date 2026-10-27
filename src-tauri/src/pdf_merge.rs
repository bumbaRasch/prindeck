#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt;
use std::io;

use lopdf::Dictionary;
use lopdf::Document;
use lopdf::Error as PdfError;
use lopdf::Object;
use lopdf::ObjectId;

#[derive(Debug)]
pub enum MergeError {
    Empty,
    NoPageTree,
    Parse(PdfError),
    Write(io::Error),
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeError::Empty => write!(f, "no pages to merge"),
            MergeError::NoPageTree => write!(f, "a page PDF has no page tree"),
            MergeError::Parse(error) => write!(f, "malformed page PDF: {error}"),
            MergeError::Write(error) => write!(f, "failed to serialize the merged PDF: {error}"),
        }
    }
}

impl std::error::Error for MergeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MergeError::Parse(error) => Some(error),
            MergeError::Write(error) => Some(error),
            MergeError::Empty | MergeError::NoPageTree => None,
        }
    }
}

pub fn merge_pdfs(pages: &[Vec<u8>]) -> Result<Vec<u8>, MergeError> {
    if pages.is_empty() {
        return Err(MergeError::Empty);
    }

    let documents = pages
        .iter()
        .map(|bytes| Document::load_mem(bytes).map_err(MergeError::Parse))
        .collect::<Result<Vec<_>, _>>()?;

    let mut merged = merge_documents(documents)?;
    let mut bytes = Vec::new();
    merged.save_to(&mut bytes).map_err(MergeError::Write)?;
    Ok(bytes)
}

fn merge_documents(documents: Vec<Document>) -> Result<Document, MergeError> {
    let mut max_id = 1;
    let mut pages: Vec<(ObjectId, Object)> = Vec::new();
    let mut objects: BTreeMap<ObjectId, Object> = BTreeMap::new();

    for mut doc in documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for page_id in doc.get_pages().into_values() {
            let page = doc
                .get_object(page_id)
                .map_err(MergeError::Parse)?
                .to_owned();
            pages.push((page_id, page));
        }

        objects.extend(doc.objects);
    }

    let mut merged = Document::with_version("1.5");
    let mut catalog: Option<(ObjectId, Dictionary)> = None;
    let mut page_tree: Option<(ObjectId, Dictionary)> = None;

    for (object_id, object) in objects {
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                if catalog.is_none() {
                    let dictionary = object.as_dict().map_err(MergeError::Parse)?.clone();
                    catalog = Some((object_id, dictionary));
                }
            }
            b"Pages" => {
                let mut dictionary = object.as_dict().map_err(MergeError::Parse)?.clone();
                if let Some((first_id, existing)) = page_tree.take() {
                    dictionary.extend(&existing);
                    page_tree = Some((first_id, dictionary));
                } else {
                    page_tree = Some((object_id, dictionary));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                merged.objects.insert(object_id, object);
            }
        }
    }

    let (catalog_id, mut catalog_dict) = catalog.ok_or(MergeError::NoPageTree)?;
    let (pages_id, mut pages_dict) = page_tree.ok_or(MergeError::NoPageTree)?;

    let kids = pages
        .iter()
        .map(|(page_id, _)| Object::Reference(*page_id))
        .collect::<Vec<_>>();
    pages_dict.set("Kids", kids);
    pages_dict.set("Count", pages.len() as u32);
    merged
        .objects
        .insert(pages_id, Object::Dictionary(pages_dict));

    for (page_id, mut page) in pages {
        if let Ok(dictionary) = page.as_dict_mut() {
            dictionary.set("Parent", pages_id);
        }
        merged.objects.insert(page_id, page);
    }

    catalog_dict.set("Pages", pages_id);
    catalog_dict.remove(b"Outlines");
    merged
        .objects
        .insert(catalog_id, Object::Dictionary(catalog_dict));

    merged.trailer.set("Root", catalog_id);
    merged.max_id = merged.objects.len() as u32;
    merged.renumber_objects();
    merged.adjust_zero_pages();
    merged.compress();

    Ok(merged)
}
