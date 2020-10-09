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

#[derive(Deserialize, Debug)]
pub struct Model {
    pub parent: Option<String>,
    pub textures: Option<HashMap<String, String>>,
    pub elements: Option<Vec<Element>>,
}

#[derive(Deserialize, Debug)]
pub struct Element {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub faces: HashMap<String, Face>,
}

#[derive(Deserialize, Debug)]
pub struct Face {
    texture: String,
    uv: Option<[f32; 4]>,
}

pub type Texture = [u8; 4 * 16 * 16]; // RGBA 16x16 image.

#[derive(Debug)]
pub enum Error {
    MissingBlockstate(String),
    MissingVariant(String, String),
    MissingModel(String, String, String),
    MissingModelTextures, // Missing textures object in model when we expected them.
    MissingTexture(String), // Missing the actual texture, ie the PNG.
    MissingTextureVariable, // A texture variable eg '#all' had no value assigned.
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

    fn model_get_top(
        &self,
        id: &str,
        encoded_props: &str,
        model: &Model,
        texture_variables: &mut HashMap<String, String>,
    ) -> Result<Texture> {
        let model_textures = model.textures.as_ref().ok_or(Error::MissingModelTextures)?;

        // The model textures here probably won't contain the 'up' texture. But
        // if it does then we can extract that texture and be done!

        if let Some(tex_name) = model_textures.get("up") {
            let tex_name = if tex_name.starts_with("#") {
                texture_variables
                    .get(tex_name.strip_prefix("#").unwrap())
                    .ok_or(Error::MissingTextureVariable)?
            } else {
                tex_name
            };

            self.extract_texture(tex_name)
        } else {
            // We need to record the textures we do have, as they will probably
            // be variables for textures in the parent model.
            for (tname, tvalue) in model_textures.iter() {
                // If the value is a variable (ie begins with "#"), we need to
                // look it up in the current texture map. Given that we process
                // the models from child to parent, they should always be present.
                let tvalue = if tname.starts_with("#") {
                    texture_variables
                        .get(tname.strip_prefix("#").unwrap())
                        .ok_or(Error::MissingTextureVariable)?
                } else {
                    tvalue
                }
                .clone();

                // Store the variables value.
                texture_variables.insert(tname.clone(), tvalue);
            }

            match &model.parent {
                Some(parent) => {
                    let parent = self.models.get(parent).ok_or(Error::MissingModel(
                        id.to_string(),
                        encoded_props.to_string(),
                        parent.to_string(),
                    ))?;

                    self.model_get_top(id, encoded_props, parent, texture_variables)
                }
                None => panic!(),
            }
        }
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
                        let model = self.models.get(model_name).ok_or(Error::MissingModel(
                            id.to_string(),
                            encoded_props.to_string(),
                            model_name.to_string(),
                        ))?;
                        self.model_get_top(id, encoded_props, model, &mut HashMap::new())
                    }
                    Variants::Many(variants) => panic!(),
                }
            }
            Blockstate::Multipart(_) => panic!(),
        }
    }
}
