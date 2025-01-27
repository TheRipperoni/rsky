use std::io::Cursor;
use crate::repo::block_map::BlockMap;
use crate::repo::types::CidAndBytes;
use crate::vendored::iroh_car::{CarHeader, CarReader, CarWriter};
use anyhow::Result;
use futures::TryStreamExt;
use lexicon_cid::Cid;

pub async fn read_car_bytes(root: Option<&Cid>, blocks: BlockMap) -> Result<Vec<u8>> {
    let roots = match root {
        Some(root) => vec![*root],
        None => vec![],
    };
    let car_header = CarHeader::new_v1(roots);
    let buf: Vec<u8> = Default::default();
    let mut car_writer = CarWriter::new(car_header, buf);

    for CidAndBytes { cid, bytes } in blocks.entries()? {
        car_writer.write(cid, bytes).await?;
    }
    Ok(car_writer.finish().await?)
}

pub async fn read_bytes_to_blockmap(data: Vec<u8>) -> Result<BlockMap> {
    let reader = Cursor::new(&data);
    let car_reader = CarReader::new(reader).await.unwrap();
    let files: Vec<_> = car_reader.stream().try_collect().await.unwrap();
    let mut blocks = BlockMap::new();
    for file in files {
        blocks.set(file.0, file.1);
    }
    Ok(blocks)
}