use prost::Message;

use crate::diagnostic::{ClutterError, Result};
use crate::model::AndroidMetadata;

const MAX_XML_NODES: usize = 1_000_000;

#[derive(Clone, PartialEq, Message)]
struct XmlNode {
    #[prost(message, optional, tag = "1")]
    element: Option<XmlElement>,
}

#[derive(Clone, PartialEq, Message)]
struct XmlElement {
    #[prost(string, tag = "2")]
    namespace_uri: String,
    #[prost(string, tag = "3")]
    name: String,
    #[prost(message, repeated, tag = "4")]
    attributes: Vec<XmlAttribute>,
    #[prost(message, repeated, tag = "5")]
    children: Vec<XmlNode>,
}

#[derive(Clone, PartialEq, Message)]
struct XmlAttribute {
    #[prost(string, tag = "1")]
    namespace_uri: String,
    #[prost(string, tag = "2")]
    name: String,
    #[prost(string, tag = "3")]
    value: String,
    #[prost(uint32, tag = "5")]
    resource_id: u32,
    #[prost(message, optional, tag = "6")]
    compiled_item: Option<Item>,
}

#[derive(Clone, PartialEq, Message)]
struct Item {
    #[prost(message, optional, tag = "7")]
    primitive: Option<Primitive>,
}

#[derive(Clone, PartialEq, Message)]
struct Primitive {
    #[prost(oneof = "primitive::Value", tags = "6, 7, 8")]
    value: Option<primitive::Value>,
}

mod primitive {
    use prost::Oneof;

    #[derive(Clone, PartialEq, Oneof)]
    pub enum Value {
        #[prost(int32, tag = "6")]
        Decimal(i32),
        #[prost(uint32, tag = "7")]
        Hexadecimal(u32),
        #[prost(bool, tag = "8")]
        Boolean(bool),
    }
}

pub fn parse_proto_xml(bytes: &[u8]) -> Result<AndroidMetadata> {
    let root = XmlNode::decode(bytes).map_err(|error| {
        ClutterError::InvalidArtifact(format!(
            "AAB AndroidManifest.xml is not valid AAPT protobuf XML: {error}"
        ))
    })?;
    let mut metadata = AndroidMetadata::default();
    let mut version_code_major = 0u64;
    let mut stack = vec![&root];
    let mut visited = 0usize;

    while let Some(node) = stack.pop() {
        visited += 1;
        if visited > MAX_XML_NODES {
            return Err(ClutterError::InvalidArtifact(format!(
                "AAB manifest exceeds the {MAX_XML_NODES} node limit"
            )));
        }
        let Some(element) = node.element.as_ref() else {
            continue;
        };
        stack.extend(element.children.iter());

        for attribute in &element.attributes {
            let string_value = (!attribute.value.is_empty()).then_some(attribute.value.as_str());
            let numeric_value =
                compiled_integer(attribute).or_else(|| attribute.value.parse::<u64>().ok());
            match (element.name.as_str(), attribute.name.as_str()) {
                ("manifest", "package") => {
                    metadata.package_name = string_value.map(str::to_owned);
                }
                ("manifest", "versionName") => {
                    metadata.version_name = string_value.map(str::to_owned);
                }
                ("manifest", "versionCode") => metadata.version_code = numeric_value,
                ("manifest", "versionCodeMajor") => {
                    version_code_major = numeric_value.unwrap_or_default();
                }
                ("uses-sdk", "minSdkVersion") => {
                    metadata.min_sdk = numeric_value.and_then(|value| u32::try_from(value).ok());
                }
                ("uses-sdk", "targetSdkVersion") => {
                    metadata.target_sdk = numeric_value.and_then(|value| u32::try_from(value).ok());
                }
                ("uses-permission", "name") => {
                    if let Some(value) = string_value {
                        metadata.permissions.push(value.to_owned());
                    }
                }
                _ => {}
            }
        }
    }

    if version_code_major != 0 {
        metadata.version_code =
            Some((version_code_major << 32) | metadata.version_code.unwrap_or_default());
    }
    metadata.permissions.sort();
    metadata.permissions.dedup();
    Ok(metadata)
}

fn compiled_integer(attribute: &XmlAttribute) -> Option<u64> {
    match attribute
        .compiled_item
        .as_ref()?
        .primitive
        .as_ref()?
        .value
        .as_ref()?
    {
        primitive::Value::Decimal(value) => u64::try_from(*value).ok(),
        primitive::Value::Hexadecimal(value) => Some(u64::from(*value)),
        primitive::Value::Boolean(value) => Some(u64::from(*value)),
    }
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::{Item, Primitive, XmlAttribute, XmlElement, XmlNode, parse_proto_xml, primitive};

    #[test]
    fn reads_aab_manifest_metadata() {
        let root = XmlNode {
            element: Some(XmlElement {
                namespace_uri: String::new(),
                name: "manifest".to_owned(),
                attributes: vec![
                    XmlAttribute {
                        namespace_uri: String::new(),
                        name: "package".to_owned(),
                        value: "dev.example.app".to_owned(),
                        resource_id: 0,
                        compiled_item: None,
                    },
                    XmlAttribute {
                        namespace_uri: String::new(),
                        name: "versionCode".to_owned(),
                        value: String::new(),
                        resource_id: 0,
                        compiled_item: Some(Item {
                            primitive: Some(Primitive {
                                value: Some(primitive::Value::Decimal(42)),
                            }),
                        }),
                    },
                ],
                children: vec![XmlNode {
                    element: Some(XmlElement {
                        namespace_uri: String::new(),
                        name: "uses-permission".to_owned(),
                        attributes: vec![XmlAttribute {
                            namespace_uri: String::new(),
                            name: "name".to_owned(),
                            value: "android.permission.INTERNET".to_owned(),
                            resource_id: 0,
                            compiled_item: None,
                        }],
                        children: Vec::new(),
                    }),
                }],
            }),
        };
        let metadata = parse_proto_xml(&root.encode_to_vec()).unwrap();
        assert_eq!(metadata.package_name.as_deref(), Some("dev.example.app"));
        assert_eq!(metadata.version_code, Some(42));
        assert_eq!(metadata.permissions, ["android.permission.INTERNET"]);
    }
}
