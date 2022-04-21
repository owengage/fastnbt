use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Block {
    pub(crate) name: String,
    pub(crate) encoded: String,
    pub(crate) archetype: BlockArchetype,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockArchetype {
    Normal,
    Airy,
    Watery,
    Snowy,
}

impl Block {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn snowy(&self) -> bool {
        self.archetype == BlockArchetype::Snowy
    }

    /// A string of the format "id|prop1=val1,prop2=val2". The properties are
    /// ordered lexigraphically. This somewhat matches the way Minecraft stores
    /// variants in blockstates, but with the block ID/name prepended.
    pub fn encoded_description(&self) -> &str {
        &self.encoded
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BlockRaw {
    name: String,

    #[serde(default)]
    properties: HashMap<String, String>,
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: BlockRaw = Deserialize::deserialize(deserializer)?;
        let snowy = raw.properties.get("snowy").map(String::as_str) == Some("true");

        let mut id = raw.name.clone() + "|";
        let mut sep = "";

        let mut props = raw
            .properties
            .iter()
            .filter(|(k, _)| *k != "waterlogged") // TODO: Handle water logging. See note below
            .filter(|(k, _)| *k != "powered") // TODO: Handle power
            .collect::<Vec<_>>();

        // need to sort the properties for a consistent ID
        props.sort_unstable();

        for (k, v) in props {
            id = id + sep + k + "=" + v;
            sep = ",";
        }

        let arch = if snowy {
            BlockArchetype::Snowy
        } else if is_watery(&raw.name) {
            BlockArchetype::Watery
        } else if is_airy(&raw.name) {
            BlockArchetype::Airy
        } else {
            BlockArchetype::Normal
        };

        Ok(Self {
            name: raw.name,
            archetype: arch,
            encoded: id,
        })
    }
}

/// Blocks that are considered as if they are water when determining colour.
fn is_watery(block: &str) -> bool {
    matches!(
        block,
        "minecraft:water"
            | "minecraft:bubble_column"
            | "minecraft:kelp"
            | "minecraft:kelp_plant"
            | "minecraft:sea_grass"
            | "minecraft:tall_seagrass"
    )
}

/// Blocks that are considered as if they are air when determining colour.
fn is_airy(block: &str) -> bool {
    matches!(block, "minecraft:air" | "minecraft:cave_air")
}
