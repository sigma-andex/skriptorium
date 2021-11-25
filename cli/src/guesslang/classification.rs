extern crate tensorflow;

use crate::types;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::hash::Hash;
use std::path;
use tensorflow::Code;
use tensorflow::Graph;
use tensorflow::SessionOptions;
use tensorflow::SessionRunArgs;
use tensorflow::Status;
use tensorflow::Tensor;
use futures::future::join_all;
use tokio::fs;

#[derive(Debug)]
pub enum ClassificationError {
    ClassificationFailed,
}

impl std::error::Error for ClassificationError {}

impl fmt::Display for ClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClassificationError::ClassificationFailed => write!(f, "Classification failed."),
        }
    }
}

pub struct GuessLangSettings {
    bundle: tensorflow::SavedModelBundle,
    graph: tensorflow::Graph,
    name_to_abbreviation: HashMap<String, String>,
    abbreviation_to_name: HashMap<String, String>,
}

fn load_model(
    path: path::PathBuf,
) -> std::result::Result<(tensorflow::SavedModelBundle, tensorflow::Graph), Box<tensorflow::Status>>
{
    let mut model_folder = path;
    model_folder.push("model");
    let key = "TF_CPP_MIN_LOG_LEVEL";
    env::set_var(key, "2");
    if !model_folder.exists() {
        return Err(Box::new(
            Status::new_set(
                Code::NotFound,
                &format!("Model {:?} not found.", model_folder.to_str()),
            )
            .unwrap(),
        ));
    }

    let mut graph = Graph::new();
    let bundle = tensorflow::SavedModelBundle::load(
        &SessionOptions::new(),
        &["serve"],
        &mut graph,
        model_folder,
    )?;
    Ok((bundle, graph))
}

pub fn classify(
    guess_lang_settings: &GuessLangSettings,
    snippet: String,
) -> std::result::Result<Vec<(String, String, f32)>, Box<tensorflow::Status>> {
    let GuessLangSettings {
        bundle,
        graph,
        abbreviation_to_name,
        name_to_abbreviation,
    } = &guess_lang_settings;

    let mut content = tensorflow::Tensor::new(&[1]);
    content[0] = snippet;

    let mut args = SessionRunArgs::new();

    let serving_signature = bundle.meta_graph_def().get_signature("serving_default")?;
    let inputs_info = &serving_signature.get_input("inputs")?;
    let op_inputs = graph.operation_by_name_required(&inputs_info.name().name)?;
    args.add_feed(&op_inputs, 0, &content);

    let classes_info = &serving_signature.get_output("classes")?;
    let op_classes = graph.operation_by_name_required(&classes_info.name().name)?;
    let classes = args.request_fetch(&op_classes, 0);

    let scores_info = &serving_signature.get_output("scores")?;
    let op_scores = graph.operation_by_name_required(&scores_info.name().name)?;
    let scores = args.request_fetch(&op_scores, 0);
    let session = &bundle.session;
    session.run(&mut args)?;

    let scores_res: Tensor<f32> = args.fetch(scores)?;
    let classes_res: Tensor<String> = args.fetch(classes)?;

    let results: Vec<(String, f32)> = classes_res
        .iter()
        .zip(scores_res.iter())
        .map(|(abbr, score)| (abbr.to_string(), score.clone()))
        .collect();

    let sorted_results = {
        let mut mapped: Vec<(String, String, f32)> = results
            .iter()
            .flat_map(|(abbr, score)| {
                {
                    abbreviation_to_name
                        .get(abbr)
                        .iter()
                        .map(|name| (abbr.to_string(), name.to_string(), score.clone()))
                }
                .collect::<Vec<(String, String, f32)>>()
            })
            .collect();
        mapped.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        mapped
    };

    Ok(sorted_results)
}

fn swap<K: Eq + Hash + Clone, V: Eq + Hash + Clone>(hashmap: HashMap<K, V>) -> HashMap<V, K> {
    let mut swapped: HashMap<V, K> = HashMap::new();
    for (k, v) in hashmap.into_iter() {
        swapped.insert(v, k);
    }
    swapped
}
async fn load_languages_config(
    path: path::PathBuf,
) -> types::Result<(HashMap<String, String>, HashMap<String, String>)> {
    let mut languages_file = path;
    languages_file.push("languages.json");
    let json = fs::read_to_string(languages_file).await?;

    let name_to_abbreviation: HashMap<String, String> = serde_json::from_str(json.as_str())?;
    let abbreviation_to_name: HashMap<String, String> = swap(name_to_abbreviation.clone());
    Ok((name_to_abbreviation, abbreviation_to_name))
}

pub async fn load_settings(path: path::PathBuf) -> types::Result<GuessLangSettings> {
    let (name_to_abbreviation, abbreviation_to_name) = load_languages_config(path.to_owned()).await?;
    let (bundle, graph) = load_model(path)?;
    Ok(GuessLangSettings {
        bundle,
        graph,
        name_to_abbreviation,
        abbreviation_to_name,
    })
}
