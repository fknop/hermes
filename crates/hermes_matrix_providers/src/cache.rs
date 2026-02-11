use std::{
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use fxhash::FxHasher64;
use tracing::debug;

use crate::{travel_matrices::TravelMatrices, travel_matrix_provider::TravelMatrixProvider};

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

pub trait MatricesCache {
    fn cache_key<P>(&self, provider: &TravelMatrixProvider, points: &[P]) -> String
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let mut hasher = FxHasher64::default();

        hash_points(points, &mut hasher);
        provider.hash(&mut hasher);

        format!("{:016x}", hasher.finish())
    }

    fn cache<P>(
        &self,
        provider: &TravelMatrixProvider,
        points: &[P],
        matrices: &TravelMatrices,
    ) -> Result<(), anyhow::Error>
    where
        for<'a> &'a P: Into<geo_types::Point>;
    fn get_cached<P>(
        &self,
        provider: &TravelMatrixProvider,
        points: &[P],
    ) -> Result<Option<TravelMatrices>, anyhow::Error>
    where
        for<'a> &'a P: Into<geo_types::Point>;
}

pub struct FileCache {
    directory: PathBuf,
}

impl FileCache {
    pub fn new(path: &str) -> Self {
        let directory = Path::new(&path).to_path_buf();

        if !directory.is_dir() {
            panic!("Path {path} is not a directory");
        }

        Self { directory }
    }
}

impl MatricesCache for FileCache {
    fn cache<P>(
        &self,
        provider: &TravelMatrixProvider,
        points: &[P],
        matrices: &TravelMatrices,
    ) -> Result<(), anyhow::Error>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let cache_key = self.cache_key(provider, points);
        let filename = format!("{}.json", cache_key);

        let file = std::fs::File::create(self.directory.join(&filename))?;
        let mut writer = BufWriter::with_capacity(64 * 1024, file);
        serde_json::to_writer(&mut writer, &matrices)?;
        writer.flush()?;

        debug!("Saved matrix to {}", filename);

        Ok(())
    }

    fn get_cached<P>(
        &self,
        provider: &TravelMatrixProvider,
        points: &[P],
    ) -> Result<Option<TravelMatrices>, anyhow::Error>
    where
        for<'a> &'a P: Into<geo_types::Point>,
    {
        let cache_key = self.cache_key(provider, points);
        let filename = format!("{}.json", cache_key);
        let file_path = self.directory.join(&filename);

        if !file_path.is_file() {
            return Ok(None);
        }

        let file = std::fs::File::open(file_path)?;

        let reader = BufReader::new(file);
        let matrices: TravelMatrices = serde_json::from_reader(reader)?;

        Ok(Some(matrices))
    }
}
