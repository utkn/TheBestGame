use std::{collections::HashSet, path::PathBuf};

use itertools::Itertools;

use super::Sprite;

#[derive(Clone, Debug, Default)]
pub struct SpriteFrames {
    src_name: String,
    representing_tags: HashSet<String>,
    frame_assets: Vec<PathBuf>,
}

impl SpriteFrames {
    /// Constructs a new sprite asset from the given sprite asset folder/file.
    pub fn load(asset_path: PathBuf) -> Option<Self> {
        let file_stem = asset_path.file_stem()?.to_str().to_owned()?;
        let (src_name, tags) = file_stem.split("@").take(2).collect_tuple()?;
        let src_name = src_name.to_owned();
        let representing_tags = tags.split("+").map(|t| t.to_owned()).collect();
        let frame_assets = if asset_path.is_file() {
            vec![asset_path]
        } else {
            glob::glob(&format!("{}/*", asset_path.to_str()?))
                .ok()?
                .into_iter()
                .flatten()
                .filter(|asset_file| asset_file.is_file())
                .flat_map(|asset_file| {
                    let file_num = asset_file
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_owned())
                        .unwrap_or_default()
                        .parse::<usize>();
                    file_num.map(|file_num| (file_num, asset_file))
                })
                .sorted_by_key(|(file_num, _)| *file_num)
                .map(|(_, asset_file)| asset_file)
                .collect_vec()
        };
        if frame_assets.is_empty() {
            return None;
        }
        println!("read frames {:?}", frame_assets);
        Some(Self {
            src_name,
            representing_tags,
            frame_assets,
        })
    }

    /// Returns true iff this asset can represent the current state of the entity, denoted by the given tags.
    pub fn can_represent(&self, tags: &HashSet<String>) -> bool {
        self.representing_tags.is_subset(tags)
    }

    /// Returns the representation score, i.e., the number of intersecting tags with the entity tags.
    pub fn representation_score(&self, tags: &HashSet<String>) -> usize {
        self.representing_tags.intersection(tags).count()
    }

    pub fn get_corresponding_frame<'a>(&'a self, sprite: &Sprite) -> Option<&'a PathBuf> {
        let idx = ((sprite.curr_time / 0.167).floor() as usize) % (self.frame_assets.len());
        self.frame_assets.get(idx)
    }
}
