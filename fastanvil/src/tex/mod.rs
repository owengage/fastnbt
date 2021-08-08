use serde::Deserialize;
use std::collections::HashMap;

#[cfg(test)]
mod test;

#[derive(Deserialize, Debug, Clone)]
pub struct Variant {
    pub model: String,
    pub x: Option<usize>,
    pub y: Option<usize>,
    pub uvlock: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Variants {
    Single(Variant),
    Many(Vec<Variant>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Blockstate {
    Variants(HashMap<String, Variants>),
    Multipart(Vec<Part>),
}

#[derive(Deserialize, Debug, Clone)]
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
    pub rotation: Option<Rotation>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rotation {
    origin: Vec<f32>,
    axis: String,
    angle: f32,
    #[serde(default)]
    rescale: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Face {
    texture: String,
    uv: Option<[f32; 4]>,
}

pub type Texture = Vec<u8>; // RGBA 16x16 image.

#[derive(Debug)]
pub enum Error {
    Unsupported,
    MissingBlockstate(String),
    MissingVariant(String, String),
    MissingModel(String),
    MissingModelTextures, // Missing textures object in model when we expected them.
    MissingTexture(String, String, String), // Missing the actual texture, ie the PNG.
    MissingElements(String, String, String),
    MissingTextureVariable(String, String, String, String), // A texture variable eg '#all' had no value assigned.
}

fn merge_models(child: &Model, mut parent: Model) -> Result<Model> {
    if parent.textures.is_none() {
        parent.textures = Some(HashMap::new());
    }

    if let Some(ref mut parent_textures) = parent.textures {
        let child_textures = child.textures.as_ref().ok_or(Error::MissingModelTextures)?;

        // Insert all the childs textures into the parent.
        // Textures can be referenced in a parent model that are only
        // defined in child models so we need to copy them down.
        for (k, v) in child_textures.iter() {
            parent_textures.insert(k.clone(), v.clone());
        }

        // We need to record the textures we do have, as they will probably
        // be variables for textures in the parent model.
        for (_, tvalue) in parent_textures.iter_mut() {
            // If the value is a variable (ie begins with "#"), we need to
            // look it up in the current texture map. Given that we process
            // the models from child to parent, they should always be
            // present.
            if let Some(rest) = tvalue.strip_prefix('#') {
                *tvalue = child_textures
                    .get(rest) // we just checked with 'starts_with'.
                    .ok_or_else(|| {
                        Error::MissingTextureVariable(
                            "?".to_owned(),
                            "?".to_owned(),
                            "?".to_owned(),
                            (*tvalue).clone(),
                        )
                    })?
                    .clone()
            }
        }
    }

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

    fn model_get_top(&self, id: &str, encoded_props: &str, model_name: &str) -> Result<Texture> {
        let model = self.flatten_model(model_name)?;
        // Look at elements. Try just looking in the first one for 'up'.

        let els = &model.elements.ok_or_else(|| {
            Error::MissingElements(
                id.to_owned(),
                encoded_props.to_owned(),
                model_name.to_owned(),
            )
        })?;

        let el = els.get(0).ok_or_else(|| {
            Error::MissingElements(
                id.to_owned(),
                encoded_props.to_owned(),
                model_name.to_owned(),
            )
        })?;

        let face = el.faces.get("up").ok_or_else(|| {
            Error::MissingElements(
                id.to_owned(),
                encoded_props.to_owned(),
                model_name.to_owned(),
            )
        })?;

        let tex = &face.texture;

        let tex = match tex.strip_prefix('#') {
            Some(rest) => {
                model
                    .textures
                    .ok_or(Error::MissingModelTextures)?
                    .get(rest) // we just checked with 'starts_with'.
                    .ok_or_else(|| {
                        Error::MissingTextureVariable(
                            id.to_owned(),
                            encoded_props.to_owned(),
                            model_name.to_owned(),
                            (*tex).clone(),
                        )
                    })?
                    .clone()
            }
            None => (*tex).clone(),
        };

        self.extract_texture(&tex)
    }

    fn get_model(&self, model: &str) -> Result<&Model> {
        self.models
            .get(model)
            .or_else(|| self.models.get(&("minecraft:".to_string() + model)))
            .ok_or_else(|| Error::MissingModel(model.to_string()))
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
        // Sometimes the texture is not prefixed with `minecraft:`, so if the
        // initial look up fails we can prepend it with this and check that too.
        match self.textures.get(tex_name) {
            Some(tex) => Ok(tex.clone()), // TODO: We keep cloning these textures, which are Vectors.
            None => match self.textures.get(&("minecraft:".to_string() + tex_name)) {
                Some(tex) => Ok(tex.clone()),
                None => Err(Error::MissingTexture(
                    "?".to_owned(),
                    "?".to_owned(),
                    tex_name.to_string(),
                )),
            },
        }
    }
}

impl Render for Renderer {
    // TODO: Make a trait.
    fn get_top(&mut self, id: &str, encoded_props: &str) -> Result<Texture> {
        let bs = self
            .blockstates
            .get(id)
            .ok_or_else(|| Error::MissingBlockstate(id.to_string()))?;

        match bs {
            // Block is made up variants based on its properties.
            Blockstate::Variants(variants) => {
                // Get the variant or variants that correspond to this exact block.
                let v = variants.get(encoded_props).ok_or_else(|| {
                    Error::MissingVariant(id.to_string(), encoded_props.to_string())
                })?;

                match v {
                    Variants::Single(variant) => {
                        let model_name = &variant.model;
                        self.model_get_top(id, encoded_props, model_name)
                    }
                    Variants::Many(variants) => {
                        // TODO: Should probably actually pick one at random or
                        // something. How does the game actually decide which
                        // variant to use?
                        let model_name = &variants[0].model;
                        self.model_get_top(id, encoded_props, model_name)
                    }
                }
            }
            Blockstate::Multipart(_) => Err(Error::Unsupported),
        }
    }
}
