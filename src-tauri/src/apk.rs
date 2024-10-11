use anyhow::Result;
use axmldecoder::{parse, Node};
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub fn label(contents: &[u8]) -> Result<Option<String>> {
    let cursor = Cursor::new(contents);
    let mut archive = ZipArchive::new(cursor)?;

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
    let Node::Element(a) = xml_root.get_root().as_ref().unwrap() else {
        return Ok(None);
    };

    let Some(attribute) = a.get_children().into_iter().find_map(|child| {
        if let Node::Element(e) = child {
            e.get_attributes().get("android:label")
        } else {
            None
        }
    }) else {
        return Ok(None);
    };

    let id: usize = attribute
        .strip_prefix("ResourceValueType::Reference/")
        .unwrap_or_default()
        .parse()?;
    // let package_id = (id >> 24) & 0xff;
    let type_id = ((id >> 16) & 0xff) - 1;
    let index = id & 0xffff;
    let Some(main_package) = arsc.packages.first() else {
        return Ok(None);
    };

    // let verify_type_id = main_package
    //     .type_names
    //     .strings
    //     .iter()
    //     .position(|s| s == "string")
    //     .unwrap();
    let Some(typez) = main_package.types.get(type_id) else {
        return Ok(None);
    };

    let Some(configs) = typez.configs.iter().find_map(|c| {
        c.resources
            .resources
            .iter()
            .find(|r| r.spec_id == index)
            .map(|c| &c.value)
    }) else {
        return Ok(None);
    };

    let data_index = match configs {
        arsc::ResourceValue::Plain(p) => p.data_index,
        arsc::ResourceValue::Bag { parent: _, values } => values.first().unwrap().1.data_index,
    };

    let me = arsc.global_string_pool.strings.get(data_index);
    Ok(me.cloned())
}
