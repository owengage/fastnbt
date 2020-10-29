use fastanvil::tex::{Model, Renderer};
use std::error::Error;
use std::path::Path;
use std::{collections::HashMap, fmt::Display};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct ErrorMessage(&'static str);
impl std::error::Error for ErrorMessage {}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn load_models(path: &Path) -> Result<HashMap<String, Model>> {
    let mut models = HashMap::<String, Model>::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let json = std::fs::read_to_string(&path)?;

            let json: Model = serde_json::from_str(&json)?;
            models.insert(
                "minecraft:block/".to_owned()
                    + path
                        .file_stem()
                        .ok_or(format!("invalid file name: {}", path.display()))?
                        .to_str()
                        .ok_or(format!("nonunicode file name: {}", path.display()))?,
                json,
            );
        }
    }

    Ok(models)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let root = Path::new(&args[0]);
    let assets = root.to_owned().join("assets").join("minecraft");

    let models = load_models(&assets.join("models").join("block"))?;

    let renderer = Renderer::new(HashMap::new(), models.clone(), HashMap::new());
    let mut failed = 0;
    let mut success = 0;

    for name in models
        .keys()
        .filter(|name| args.get(1).map(|s| s == *name).unwrap_or(true))
    {
        let res = renderer.flatten_model(name);
        match res {
            Ok(model) => {
                println!("{}: {:#?}", name, model);
                success += 1;
            }
            Err(e) => {
                println!("{:?}", e);
                failed += 1;
            }
        };
    }

    println!("success {:?}, failed {:?}", success, failed);
    Ok(())
}
