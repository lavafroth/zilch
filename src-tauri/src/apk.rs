use std::io::{Cursor, Read};

use anyhow::Result;
use resand::res_value::ResValueType;
use resand::stream::StreamResult;
use resand::table::{ResTable, ResTableEntryValue};
use resand::xmltree::XMLTree;
use zip::ZipArchive;

pub fn label(contents: &[u8]) -> Result<Option<String>> {
    let cursor = Cursor::new(contents);
    let mut archive = ZipArchive::new(cursor)?;
    let archive_size_limit = 400 << 20;
    if archive
        .decompressed_size()
        .is_some_and(|estimated_size| estimated_size > archive_size_limit)
    {
        return Ok(Some(String::from("APK too large to decompress")));
    }

    let mut manifest_buffer = vec![];
    let mut manifest = {
        let entry = "AndroidManifest.xml";
        let mut reader = archive.by_name(entry)?;
        reader.read_to_end(&mut manifest_buffer)?;
        Cursor::new(manifest_buffer)
        // reader gets auto-dropped
    };

    let entry = "resources.arsc";
    let mut reader = archive.by_name(entry)?;
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;
    let mut resource = Cursor::new(buffer);

    let Ok(xml) = XMLTree::read(&mut manifest) else {
        return Ok(None);
    };
    let StreamResult::Ok(table) = ResTable::read_all(&mut resource) else {
        return Ok(None);
    };
    let name = get_label(xml, table);
    Ok(name)
}

fn get_label(xml: XMLTree, table: ResTable) -> Option<String> {
    let application = xml
        .root
        .get_elements(&["manifest", "application"], &xml.string_pool)
        .pop()?;

    let attr = application.get_attribute("label", &xml.string_pool)?;

    let ResValueType::Reference(attr) = attr.typed_value.data else {
        return None;
    };
    let package = table.packages.first()?;
    let entry = package.resolve_ref(attr)?;
    let ResTableEntryValue::ResValue(value) = &entry.data else {
        return None;
    };

    let ResValueType::String(spo) = value.data.data else {
        return None;
    };

    table.string_pool.resolve(spo).map(ToString::to_string)
}
