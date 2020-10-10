use serde::Deserialize;
use std::collections::HashMap;

#[cfg(test)]
mod test;

#[derive(Deserialize, Debug)]
pub struct Variant {
    pub model: String,
    pub x: Option<usize>,
    pub y: Option<usize>,
    pub uvlock: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Variants {
    Single(Variant),
    Many(Vec<Variant>),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Blockstate {
    Variants(HashMap<String, Variants>),
    Multipart(Vec<Part>),
}

#[derive(Deserialize, Debug)]
pub struct Part {
    //when: Option<serde_json::Value>,
    pub apply: Variants,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Model {
    pub parent: Option<String>,
    pub textures: Option<HashMap<String, String>>,
    pub elements: Option<Vec<Element>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Element {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub faces: HashMap<String, Face>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Face {
    texture: String,
    uv: Option<[f32; 4]>,
}

pub type Texture = [u8; 4 * 16 * 16]; // RGBA 16x16 image.

#[derive(Debug)]
pub enum Error {
    Unsupported,
    MissingBlockstate(String),
    MissingVariant(String, String),
    MissingModel(String),
    MissingModelTextures, // Missing textures object in model when we expected them.
    MissingTexture(String), // Missing the actual texture, ie the PNG.
    MissingTextureVariable, // A texture variable eg '#all' had no value assigned.
}

fn merge_models(child: &Model, mut parent: Model) -> Result<Model> {
    match parent.textures {
        Some(ref mut parent_textures) => {
            let child_textures = child.textures.as_ref().ok_or(Error::MissingModelTextures)?;

            // We need to record the textures we do have, as they will probably
            // be variables for textures in the parent model.
            for (_, tvalue) in parent_textures.iter_mut() {
                // If the value is a variable (ie begins with "#"), we need to
                // look it up in the current texture map. Given that we process
                // the models from child to parent, they should always be
                // present.
                match tvalue.strip_prefix("#") {
                    Some(rest) => {
                        *tvalue = child_textures
                            .get(rest) // we just checked with 'starts_with'.
                            .ok_or(Error::MissingTextureVariable)?
                            .clone()
                    }
                    None => {}
                }
            }

            Ok(parent)
        }
        None => {
            // We're in a model with no textures. This likely means that we're
            // at a model that describes the actual geometry of the textures
            // now.
            parent.textures = child.textures.clone();

            // Copy any geometry elements from child to parent.
            match parent.elements {
                None => parent.elements = child.elements.clone(),
                Some(ref mut pels) => {
                    for el in child.elements.iter().flatten() {
                        pels.push(el.clone());
                    }
                }
            }

            Ok(parent)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Render {
    fn get_top(&mut self, id: &str, encoded_props: &str) -> Result<Texture>;
}
pub struct Renderer {
    blockstates: HashMap<String, Blockstate>,
    models: HashMap<String, Model>,
    textures: HashMap<String, Texture>,
}

impl Renderer {
    pub fn new(
        blockstates: HashMap<String, Blockstate>,
        models: HashMap<String, Model>,
        textures: HashMap<String, Texture>,
    ) -> Self {
        Self {
            blockstates,
            models,
            textures,
        }
    }

    fn model_get_top(&self, id: &str, encoded_props: &str, model: &str) -> Result<Texture> {
        let model = self.flatten_model(model)?;
        self.extract_texture(&model.textures.unwrap()["up"])
    }

    fn get_model(&self, model: &str) -> Result<&Model> {
        self.models
            .get(model)
            .or_else(|| self.models.get(&("minecraft:".to_string() + model)))
            .ok_or(Error::MissingModel(model.to_string()))
    }

    pub fn flatten_model(&self, model: &str) -> Result<Model> {
        let mut model = self.get_model(model)?.clone();

        while let Some(parent) = model.parent.as_ref() {
            let parent = self.get_model(parent)?;
            model = merge_models(&model, parent.clone())?;
        }

        Ok(model)
    }

    fn extract_texture(&self, tex_name: &str) -> Result<Texture> {
        self.textures
            .get(tex_name)
            .map(|t| t.clone())
            .ok_or(Error::MissingTexture(tex_name.to_string()))
    }
}

impl Render for Renderer {
    // TODO: Make a trait.
    fn get_top(&mut self, id: &str, encoded_props: &str) -> Result<Texture> {
        let bs = self
            .blockstates
            .get(id)
            .ok_or(Error::MissingBlockstate(id.to_string()))?;

        match bs {
            // Block is made up variants based on its properties.
            Blockstate::Variants(variants) => {
                // Get the variant or variants that correspond to this exact block.
                let v = variants.get(encoded_props).ok_or(Error::MissingVariant(
                    id.to_string(),
                    encoded_props.to_string(),
                ))?;

                match v {
                    Variants::Single(variant) => {
                        let model_name = &variant.model;
                        self.model_get_top(id, encoded_props, model_name)
                    }
                    Variants::Many(variants) => Err(Error::Unsupported),
                }
            }
            Blockstate::Multipart(_) => Err(Error::Unsupported),
        }
    }
}
