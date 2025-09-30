use anyhow::Result;
use axmldecoder::{parse, Node};
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub fn label(contents: &[u8]) -> Result<Option<String>> {
    let cursor = Cursor::new(contents);
    let mut archive = ZipArchive::new(cursor)?;
    let archive_size_limit = 400 << 20;
    if archive
        .decompressed_size()
        .is_some_and(|estimated_size| estimated_size > archive_size_limit)
    {
        return Ok(Some(String::from("No name")));
    }

    let manifest = {
        let entry = "AndroidManifest.xml";
        let mut reader = archive.by_name(entry)?;
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer)?;
        buffer
        // reader gets auto-dropped
    };

    let entry = "resources.arsc";
    let mut reader = archive.by_name(entry)?;
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;
    let resource = buffer;

    let xml_root = parse(&manifest).unwrap();
    let arsc = arsc::parse_from(Cursor::new(resource))?;
    Ok(label_extract_resource_id(xml_root, arsc))
}

fn as_xml_element(node: &Node) -> Option<&axmldecoder::Element> {
    let Node::Element(el) = node else { return None };
    Some(el)
}

fn label_extract_resource_id(
    xml_root: axmldecoder::XmlDocument,
    arsc: arsc::Arsc,
) -> Option<String> {
    let attribute = as_xml_element(xml_root.get_root().as_ref()?)?
        .get_children()
        .iter()
        .find_map(|child| as_xml_element(child)?.get_attributes().get("android:label"))?;

    let id: usize = attribute
        .strip_prefix("ResourceValueType::Reference/")
        .unwrap_or_default()
        .parse()
        .ok()?;
    // let package_id = (id >> 24) & 0xff;
    let type_id = ((id >> 16) & 0xff) - 1;
    let index = id & 0xffff;
    let main_package = arsc.packages.first()?;

    // let verify_type_id = main_package
    //     .type_names
    //     .strings
    //     .iter()
    //     .position(|s| s == "string")
    //     .unwrap();

    let configs = main_package
        .types
        .get(type_id)?
        .configs
        .iter()
        .find_map(|c| {
            c.resources
                .resources
                .iter()
                .find(|r| r.spec_id == index)
                .map(|c| &c.value)
        })?;
    let data_index = match configs {
        arsc::ResourceValue::Plain(p) => p.data_index,
        arsc::ResourceValue::Bag { parent: _, values } => values.first().unwrap().1.data_index,
    };

    arsc.global_string_pool.strings.get(data_index).cloned()
}
