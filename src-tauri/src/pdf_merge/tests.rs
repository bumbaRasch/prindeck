use lopdf::Document;
use lopdf::dictionary;

use super::{MergeError, merge_pdfs};

const PAGE: &[u8] = include_bytes!("../../../fixtures/escl/platen-300-gray.pdf");

fn page_stack(count: usize) -> Vec<Vec<u8>> {
    vec![PAGE.to_vec(); count]
}

#[test]
fn merges_three_device_pages_into_one_document() {
    let merged = merge_pdfs(&page_stack(3)).expect("the merge to succeed");

    let document = Document::load_mem(&merged).expect("the merged PDF to parse");

    assert_eq!(document.get_pages().len(), 3);
}

#[test]
fn merge_single_page_yields_one_page() {
    let merged = merge_pdfs(&page_stack(1)).expect("the merge to succeed");

    let document = Document::load_mem(&merged).expect("the merged PDF to parse");

    assert_eq!(document.get_pages().len(), 1);
}

#[test]
fn merge_rejects_garbage() {
    let result = merge_pdfs(&[PAGE.to_vec(), b"not a pdf".to_vec()]);

    assert!(matches!(result, Err(MergeError::Parse(_))));
}

#[test]
fn merged_bytes_start_with_pdf_header() {
    let merged = merge_pdfs(&page_stack(3)).expect("the merge to succeed");

    assert!(merged.starts_with(b"%PDF-"));
}

#[test]
fn merge_rejects_empty_input() {
    let result = merge_pdfs(&[]);

    assert!(matches!(result, Err(MergeError::Empty)));
}

#[test]
fn merge_rejects_pageless_pdf() {
    let mut doc = Document::with_version("1.5");
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
    });
    doc.trailer.set("Root", catalog_id);
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes).expect("the pageless PDF to save");

    let result = merge_pdfs(&[bytes]);

    assert!(matches!(result, Err(MergeError::NoPageTree)));
}
