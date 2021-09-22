use std::{collections::HashMap, fmt};

use kdl::KdlNode;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize,
};

use crate::{value, Value};

/// A newtype wrapper for [`KdlNode`] with Serde support.
///
/// ```rust
/// # use kdl::KdlNode;
/// # use serde_kdl::Node;
/// # use serde_json::json;
/// let doc = "parent 1 root=true { child 2 root=false; }";
/// let nodes: Vec<KdlNode> = kdl::parse_document(doc).unwrap();
/// let node: KdlNode = nodes[0].clone();
/// let json = serde_json::to_value(Node(node)).unwrap();
/// assert_eq!(json, json!({
///     "name": "parent",
///     "values": [1],
///     "properties": {
///         "root": true
///     },
///     "children": [
///         {
///             "name": "child",
///             "values": [2],
///             "properties": {
///                 "root": false
///             },
///             "children": []
///         }
///     ]
/// }));
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Node(pub KdlNode);

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut node = serializer.serialize_struct("Node", 4)?;

        node.serialize_field("name", &self.0.name)?;
        let values = value::from_kdl_vec(self.0.values.clone());
        node.serialize_field("values", &values)?;
        let properties = value::from_kdl_map(self.0.properties.clone());
        node.serialize_field("properties", &properties)?;
        let children = from_kdl_vec(self.0.children.clone());
        node.serialize_field("children", &children)?;
        node.end()
    }
}

// TODO: use strum for codegen
const FIELDS: &[&str] = &["name", "values", "properties", "children"];

enum Field {
    Name,
    Values,
    Properties,
    Children,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of: `name`, `values`, `properties`, `children`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Field, E>
            where
                E: de::Error,
            {
                match value {
                    "name" => Ok(Field::Name),
                    "values" => Ok(Field::Values),
                    "properties" => Ok(Field::Properties),
                    "children" => Ok(Field::Children),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NodeVisitor;

        impl<'de> Visitor<'de> for NodeVisitor {
            type Value = Node;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Node")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Node, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let name = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let values = seq
                    .next_element::<Vec<Value>>()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let properties = seq
                    .next_element::<HashMap<String, Value>>()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let children = seq
                    .next_element::<Vec<Node>>()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(Node(KdlNode {
                    name,
                    values: value::into_kdl_vec(values),
                    properties: value::into_kdl_map(properties),
                    children: into_kdl_vec(children),
                }))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Node, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut values = None;
                let mut properties = None;
                let mut children = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Values => {
                            if values.is_some() {
                                return Err(de::Error::duplicate_field("values"));
                            }
                            values = Some(map.next_value()?);
                        }
                        Field::Properties => {
                            if properties.is_some() {
                                return Err(de::Error::duplicate_field("properties"));
                            }
                            properties = Some(map.next_value()?);
                        }
                        Field::Children => {
                            if children.is_some() {
                                return Err(de::Error::duplicate_field("children"));
                            }
                            children = Some(map.next_value()?);
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let values = values.ok_or_else(|| de::Error::missing_field("values"))?;
                let properties =
                    properties.ok_or_else(|| de::Error::missing_field("properties"))?;
                let children = children.ok_or_else(|| de::Error::missing_field("children"))?;
                Ok(Node(KdlNode {
                    name,
                    values: value::into_kdl_vec(values),
                    properties: value::into_kdl_map(properties),
                    children: into_kdl_vec(children),
                }))
            }
        }

        deserializer.deserialize_struct("Node", FIELDS, NodeVisitor)
    }
}

/// Maps `Vec<`[`KdlNode`]`>` to `Vec<`[`Node`]`>`.
pub fn from_kdl_vec(v: Vec<KdlNode>) -> Vec<Node> {
    v.into_iter().map(Node).collect()
}

/// Maps `Vec<`[`Node`]`>` to `Vec<`[`KdlNode`]`>`.
pub fn into_kdl_vec(v: Vec<Node>) -> Vec<KdlNode> {
    v.into_iter().map(|Node(v)| v).collect()
}

// REVIEW: maps of nodes?

// #[allow(dead_code)]
// pub fn from_kdl_map(v: HashMap<String, KdlNode>) -> HashMap<String, Node> {
//     v.into_iter().map(|(k, v)| (k, Node(v))).collect()
// }

// #[allow(dead_code)]
// pub fn into_kdl_map(v: HashMap<String, Node>) -> HashMap<String, KdlNode> {
//     v.into_iter().map(|(k, Node(v))| (k, v)).collect()
// }
