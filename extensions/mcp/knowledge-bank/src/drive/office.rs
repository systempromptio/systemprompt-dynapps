//! Structured workbook and presentation extraction with source locations.

use crate::error::KnowledgeBankError;
use calamine::{Reader, open_workbook_auto};
use quick_xml::events::Event;
use std::io::{Cursor, Read};
use std::path::Path;

pub fn workbook(path: &Path) -> Result<String, KnowledgeBankError> {
    let mut book = open_workbook_auto(path)
        // Why: calamine's error is variant-less for our purposes; the text is
        // the whole diagnostic.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::InvalidInput(format!("Workbook: {e}")))?;
    let mut output = String::new();
    for sheet in book.sheet_names() {
        let range = book
            .worksheet_range(&sheet)
            // Why: calamine's error is variant-less; the text is the whole
            // diagnostic.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::InvalidInput(format!("Sheet {sheet}: {e}")))?;
        output.push_str(&format!("\n# Sheet: {sheet}\n"));
        let start = range.start().unwrap_or_default();
        for (row, column, cell) in range.cells() {
            let value = cell.to_string();
            if !value.is_empty() {
                output.push_str(&format!(
                    "R{}C{}: {value}\n",
                    row + start.0 as usize + 1,
                    column + start.1 as usize + 1
                ));
            }
        }
        // Why: calamine's error is variant-less; the text is the whole
        // diagnostic.
        // lint-ok: error-adapt
        let formula = book.worksheet_formula(&sheet).map_err(|e| {
            KnowledgeBankError::InvalidInput(format!("Sheet formulas {sheet}: {e}"))
        })?;
        let start = formula.start().unwrap_or_default();
        for (row, column, value) in formula.cells() {
            if !value.is_empty() {
                output.push_str(&format!(
                    "Formula R{}C{}: {value}\n",
                    row + start.0 as usize + 1,
                    column + start.1 as usize + 1
                ));
            }
        }
    }
    Ok(output)
}

pub fn presentation(bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        // Why: a malformed archive is bad input whatever zip's variant says;
        // the text is the whole diagnostic.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::InvalidInput(format!("Presentation: {e}")))?;
    let manifest = read_entry(&mut archive, "ppt/presentation.xml")?;
    let links = relationships(&read_entry(
        &mut archive,
        "ppt/_rels/presentation.xml.rels",
    )?)?;
    let mut reader = quick_xml::Reader::from_str(&manifest);
    let mut slides = Vec::new();
    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(tag) | Event::Empty(tag) if tag.local_name().as_ref() == b"sldId" => {
                for attribute in tag.attributes() {
                    let attribute = attribute.map_err(xml_error)?;
                    if attribute.key.as_ref().ends_with(b":id") {
                        let id = attribute
                            .decode_and_unescape_value(reader.decoder())
                            .map_err(xml_error)?;
                        let target = links.get(id.as_ref()).ok_or_else(|| {
                            KnowledgeBankError::InvalidInput(
                                "Missing slide relationship".to_owned(),
                            )
                        })?;
                        slides.push(resolve_target("ppt", target)?);
                    }
                }
            },
            Event::Eof => break,
            _ => {},
        }
    }
    let mut output = String::new();
    for (index, name) in slides.iter().enumerate() {
        let xml = read_entry(&mut archive, name)?;
        output.push_str(&format!("\n# Slide {}\n{}\n", index + 1, xml_text(&xml)?));
        let path = Path::new(name);
        let parent = path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("ppt/slides");
        let filename = path.file_name().and_then(|p| p.to_str()).unwrap_or("");
        let relationship_path = format!("{parent}/_rels/{filename}.rels");
        if archive.file_names().any(|n| n == relationship_path) {
            let relationships = relationships(&read_entry(&mut archive, &relationship_path)?)?;
            for target in relationships
                .values()
                .filter(|target| target.contains("notesSlide"))
            {
                let notes = read_entry(&mut archive, &resolve_target(parent, target)?)?;
                output.push_str(&format!(
                    "\n# Speaker notes for slide {}\n{}\n",
                    index + 1,
                    xml_text(&notes)?
                ));
            }
        }
    }
    Ok(output)
}

fn read_entry(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, KnowledgeBankError> {
    let mut file = archive.by_name(name).map_err(xml_error)?;
    if file.size() > 32 * 1024 * 1024 {
        return Err(KnowledgeBankError::InvalidInput(
            "Office XML part exceeds 32 MiB".to_owned(),
        ));
    }
    let mut xml = String::new();
    file.read_to_string(&mut xml)
        .map_err(|error| super::google::io_error(&error))?;
    Ok(xml)
}

fn relationships(
    xml: &str,
) -> Result<std::collections::HashMap<String, String>, KnowledgeBankError> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut result = std::collections::HashMap::new();
    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(tag) | Event::Empty(tag)
                if tag.local_name().as_ref() == b"Relationship" =>
            {
                let mut id = None;
                let mut target = None;
                let mut external = false;
                for attribute in tag.attributes() {
                    let attribute = attribute.map_err(xml_error)?;
                    let value = attribute
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(xml_error)?
                        .into_owned();
                    match attribute.key.as_ref() {
                        b"Id" => id = Some(value),
                        b"Target" => target = Some(value),
                        b"TargetMode" => external = value == "External",
                        _ => {},
                    }
                }
                if !external && let (Some(id), Some(target)) = (id, target) {
                    result.insert(id, target);
                }
            },
            Event::Eof => break,
            _ => {},
        }
    }
    Ok(result)
}

fn resolve_target(base: &str, target: &str) -> Result<String, KnowledgeBankError> {
    let combined = if target.starts_with('/') {
        target.trim_start_matches('/').to_owned()
    } else {
        format!("{base}/{target}")
    };
    let mut parts = Vec::new();
    for part in combined.split('/') {
        match part {
            ".." => {
                if parts.pop().is_none() {
                    return Err(KnowledgeBankError::InvalidInput(
                        "Office relationship escapes the package".to_owned(),
                    ));
                }
            },
            "." | "" => {},
            _ => parts.push(part),
        }
    }
    Ok(parts.join("/"))
}

fn xml_error(error: impl std::fmt::Display) -> KnowledgeBankError {
    KnowledgeBankError::InvalidInput(format!("Office XML: {error}"))
}

pub fn xml_text(xml: &str) -> Result<String, KnowledgeBankError> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut output = String::new();
    let mut in_text = false;
    loop {
        match reader
            .read_event()
            // Why: quick-xml's error is variant-less for our purposes; the
            // text is the whole diagnostic.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::InvalidInput(format!("Office XML: {e}")))?
        {
            Event::Start(tag) => {
                if tag.local_name().as_ref() == b"t" {
                    in_text = true;
                }
            },
            Event::End(tag) => {
                if tag.local_name().as_ref() == b"t" {
                    in_text = false;
                    output.push(' ');
                }
                if tag.local_name().as_ref() == b"p" {
                    output.push('\n');
                }
            },
            Event::Text(text) if in_text => {
                let decoded = text
                    .decode()
                    .map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))?;
                output.push_str(
                    &quick_xml::escape::unescape(&decoded)
                        .map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))?,
                );
            },
            Event::GeneralRef(reference) if in_text => {
                let decoded = reference
                    .decode()
                    .map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))?;
                output.push_str(
                    &quick_xml::escape::unescape(&format!("&{decoded};"))
                        .map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))?,
                );
            },
            Event::Eof => break,
            _ => {},
        }
    }
    Ok(output)
}
