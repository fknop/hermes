use std::{
    hash::{Hash, Hasher},
    io::{BufWriter, Write},
    path::Path,
};

use fxhash::FxHasher64;

use crate::{travel_matrices::TravelMatrices, travel_matrix_provider::TravelMatrixProvider};

const CACHE_FOLDER_ENV_VAR: &str = "HERMES_CACHE_FOLDER";

fn hash_points<H, P>(points: &[P], hasher: &mut H)
where
    H: Hasher,
    for<'a> &'a P: Into<geo_types::Point>,
{
    points.len().hash(hasher);
    for point in points {
        let point = point.into();
        hasher.write_u64(point.x().to_bits());
        hasher.write_u64(point.y().to_bits());
    }
}

fn get_filename<P>(points: &[P], provider: &TravelMatrixProvider) -> Result<String, anyhow::Error>
where
    for<'a> &'a P: Into<geo_types::Point>,
{
    let mut hasher = FxHasher64::default();

    hash_points(points, &mut hasher);
    provider.hash(&mut hasher);

    let hash = hasher.finish();
    Ok(format!("{:016x}.json", hash))
}

pub fn cache_matrices<P>(
    points: &[P],
    provider: &TravelMatrixProvider,
    matrices: &TravelMatrices,
) -> Result<(), anyhow::Error>
where
    for<'a> &'a P: Into<geo_types::Point>,
{
    let cache_folder_path = std::env::var(CACHE_FOLDER_ENV_VAR)?;

    let cache_folder = Path::new(&cache_folder_path);

    if !cache_folder.is_dir() {
        return Err(anyhow::anyhow!(format!(
            "Path {} is not a directory",
            cache_folder_path
        )));
    }

    let filename = get_filename(points, provider)?;

    let file = std::fs::File::create(cache_folder.join(filename))?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    serde_json::to_writer(&mut writer, &matrices)?;
    writer.flush()?;

    Ok(())
}

pub fn get_cached_matrices<P>(
    points: &[P],
    provider: &TravelMatrixProvider,
) -> Result<Option<TravelMatrices>, anyhow::Error>
where
    for<'a> &'a P: Into<geo_types::Point>,
{
    let cache_folder_path = std::env::var(CACHE_FOLDER_ENV_VAR)?;

    let cache_folder = Path::new(&cache_folder_path);

    if !cache_folder.is_dir() {
        return Err(anyhow::anyhow!(format!(
            "Path {} is not a directory",
            cache_folder_path
        )));
    }

    let filename = get_filename(points, provider)?;
    let file_path = cache_folder.join(filename);

    if !file_path.is_file() {
        return Ok(None);
    }

    let file = std::fs::File::open(file_path)?;
    let matrices: TravelMatrices = serde_json::from_reader(file)?;

    Ok(Some(matrices))
}
