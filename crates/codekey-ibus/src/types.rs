//! IBus GVariant wire types (public protocol layout).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zvariant::{OwnedValue, Type, Value};

type Attachments = HashMap<String, OwnedValue>;

fn empty_attachments() -> Attachments {
    HashMap::new()
}

fn as_variant<T: Into<Value<'static>>>(value: T) -> OwnedValue {
    let value: Value<'static> = value.into();
    OwnedValue::try_from(value).expect("IBus value to OwnedValue")
}

const ATTR_TYPE_UNDERLINE: u32 = 1;
const ATTR_UNDERLINE_SINGLE: u32 = 1;

/// Signature `(sa{sv}uuuu)`
#[derive(Debug, Clone, Type, Serialize, Deserialize, Value, OwnedValue)]
pub struct IBusAttribute {
    pub name: String,
    pub attachments: Attachments,
    pub kind: u32,
    pub value: u32,
    pub start_index: u32,
    pub end_index: u32,
}

/// Signature `(sa{sv}av)`
#[derive(Debug, Clone, Type, Serialize, Deserialize, Value, OwnedValue)]
pub struct IBusAttrList {
    pub name: String,
    pub attachments: Attachments,
    pub attributes: Vec<OwnedValue>,
}

/// Signature `(sa{sv}sv)`
#[derive(Debug, Clone, Type, Serialize, Deserialize, Value, OwnedValue)]
pub struct IBusText {
    pub name: String,
    pub attachments: Attachments,
    pub text: String,
    pub attr_list: OwnedValue,
}

impl IBusText {
    pub fn plain(text: impl Into<String>) -> Self {
        let attrs = IBusAttrList {
            name: "IBusAttrList".into(),
            attachments: empty_attachments(),
            attributes: Vec::new(),
        };
        Self {
            name: "IBusText".into(),
            attachments: empty_attachments(),
            text: text.into(),
            attr_list: as_variant(attrs),
        }
    }

    pub fn underlined(text: impl Into<String>) -> Self {
        let text = text.into();
        let end = text.chars().count() as u32;
        let attr = IBusAttribute {
            name: "IBusAttribute".into(),
            attachments: empty_attachments(),
            kind: ATTR_TYPE_UNDERLINE,
            value: ATTR_UNDERLINE_SINGLE,
            start_index: 0,
            end_index: end,
        };
        let attrs = IBusAttrList {
            name: "IBusAttrList".into(),
            attachments: empty_attachments(),
            attributes: vec![as_variant(attr)],
        };
        Self {
            name: "IBusText".into(),
            attachments: empty_attachments(),
            text,
            attr_list: as_variant(attrs),
        }
    }

    pub fn into_variant(self) -> OwnedValue {
        as_variant(self)
    }
}
