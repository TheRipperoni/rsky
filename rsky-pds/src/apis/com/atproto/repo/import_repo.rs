use crate::auth_verifier::{AccessFull, AccessStandardIncludeChecks};
use crate::common::tid::{Ticker, TID};
use crate::common::ContentType;
use crate::models::{ErrorCode, ErrorMessageResponse};
use crate::repo::aws::s3::S3BlobStore;
use crate::repo::block_map::BlockMap;
use crate::repo::cid_set::CidSet;
use crate::repo::parse::get_and_parse_record;
use crate::repo::types::{CommitData, Lex, RecordCreateOrDeleteDescript, RecordCreateOrUpdateOp, RecordDeleteOp, RecordUpdateDescript, RecordWriteDescript, VerifiedDiff, WriteOpAction};
use crate::repo::{find_blob_refs, ActorStore, Repo};
use crate::SharedSequencer;
use aws_config::SdkConfig;
use chrono::Utc;
use libipld::cid::Cid;
use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::State;
use rsky_lexicon::com::atproto::repo::BlobOutput;
use rsky_syntax::aturi::AtUri;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use anyhow::bail;
use diesel::PgJsonbExpressionMethods;
use rsky_lexicon::com::atproto::sync::SubscribeRepos;
use crate::car::read_bytes_to_blockmap;
use crate::repo::data_diff::DataDiff;
use crate::repo::mst::NodeEntry::MST;
use crate::repo::util::parse_data_key;
use crate::storage::SqlRepoReader;

#[rocket::post("/xrpc/com.atproto.repo.importRepo", data = "<blob>")]
pub async fn import_repo(
    auth: AccessStandardIncludeChecks,
    blob: Data<'_>,
    content_type: ContentType,
    s3_config: &State<SdkConfig>,
) -> anyhow::Result<Json<()>, status::Custom<Json<ErrorMessageResponse>>> {
    match inner_import_repo(auth, blob, content_type, s3_config).await {
        Ok(res) => Ok(Json(res)),
        Err(error) => {
            eprintln!("{error:?}");
            let internal_error = ErrorMessageResponse {
                code: Some(ErrorCode::InternalServerError),
                message: Some(error.to_string()),
            };
            return Err(status::Custom(
                Status::InternalServerError,
                Json(internal_error),
            ));
        }
    }
}

async fn inner_import_repo(
    auth: AccessStandardIncludeChecks,
    blob: Data<'_>,
    content_type: ContentType,
    s3_config: &State<SdkConfig>,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let rev = Ticker::new().next(None).to_string();
    let requester = auth.access.credentials.unwrap().did.unwrap();
    let mut actor_store = ActorStore::new(
        requester.clone(),
        S3BlobStore::new(requester.clone(), s3_config),
    );

    let y = blob.open(100.megabytes());
    let z = y.into_bytes().await.unwrap().value;
    let blocks = read_bytes_to_blockmap(z).await?;

    let is_create;
    let existing_repo;
    let curr_repo = Repo::load(&mut actor_store.storage, None).await;
    match curr_repo {
        Ok(repo) => {
            is_create = false;
            existing_repo = repo;
        }
        Err(e) => {
            eprintln!("{e}");
            is_create = false;
            panic!("Panic")
        }
    }

    let mut diff: VerifiedDiff = verify_diff(&existing_repo, &blocks, existing_repo.cid).await;
    diff.commit.rev = rev.clone();

    match actor_store
        .storage
        .apply_commit(diff.commit, Some(is_create))
        .await
    {
        Ok(_) => {
            eprintln!("commit applied");
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }

    // write diffs
    for write in diff.write {
        match write {
            RecordWriteDescript::Create(create_write) => {
                let uri = AtUri::make(
                    requester.clone(),
                    Some(create_write.collection),
                    Some(create_write.rkey),
                )
                    .unwrap();
                let parsed_record;
                match get_and_parse_record(&blocks, create_write.cid) {
                    Ok(record) => {
                        parsed_record = record.record;
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        panic!()
                    }
                }

                //Index Record
                actor_store.record.index_record(
                        uri.to_string(),
                        create_write.cid,
                        Some(parsed_record.clone()),
                        Some(create_write.action),
                        rev.clone(),
                        Some(now.to_string()),
                    ).await.unwrap();

                //Index Blob References
                let record_blobs = find_blob_refs(Lex::Map(parsed_record.clone()), None, None);
                if !record_blobs.is_empty() {
                    //TODO insert into record_blob
                }
            }
            RecordWriteDescript::Update(update_write) => {}
            RecordWriteDescript::Delete(delete_write) => {
                let uri = AtUri::make(
                    requester.clone(),
                    Some(delete_write.collection),
                    Some(delete_write.rkey),
                ).unwrap();
                actor_store.record.delete_record(uri.to_string()).await?;
            }
        }
    }
    Ok(())
}

fn diff_to_write_descripts(diff: &DataDiff) -> Vec<RecordWriteDescript> {
    let mut result = Vec::new();
    let updates = &diff.updates;
    for update in updates {
        let record_path = parse_data_key(update.0).unwrap();
        result.push(RecordWriteDescript::Update(RecordUpdateDescript {
            action: WriteOpAction::Update,
            collection: record_path.collection,
            rkey: record_path.rkey,
            prev: Default::default(),
            cid: Default::default(),
        }).clone())
    }
    let adds = &diff.adds;
    for add in adds {
        let record_path = parse_data_key(add.0).unwrap();
        result.push(RecordWriteDescript::Create(RecordCreateOrDeleteDescript {
            action: WriteOpAction::Create,
            collection: record_path.collection,
            rkey: record_path.rkey,
            cid: Default::default(),
        }).clone())
    }
    let deletes = &diff.deletes;
    for delete in deletes {
        let record_path = parse_data_key(delete.0).unwrap();
        result.push(RecordWriteDescript::Delete(RecordCreateOrDeleteDescript {
            action: WriteOpAction::Delete,
            collection: record_path.collection,
            rkey: record_path.rkey,
            cid: Default::default(),
        }).clone())
    }
    result
}

async fn verify_diff(repo: &Repo, mut imported_blocks: &BlockMap, imported_root: Cid) -> VerifiedDiff {
    let now = Utc::now();
    let mut reader = SqlRepoReader {
        cache: BlockMap::new(),
        blocks: imported_blocks.clone(),
        root: Some(imported_root),
        rev: None,
        now: now.to_string(),
        did: repo.commit.did.clone(),
    };
    let mut updated = Repo::load(&mut reader, Some(imported_root)).await.unwrap();
    let diff = DataDiff::of(&mut updated.data, Some(&mut repo.data.clone())).unwrap();
    let writes = diff_to_write_descripts(&diff);
    let mut new_blocks = diff.new_mst_blocks;
    let leaves = imported_blocks.clone().get_many(diff.new_leaf_cids.to_list()).unwrap();
    new_blocks.add_map(leaves.blocks).unwrap();
    let mut removed_cids = diff.removed_cids;
    let commit_cid = new_blocks.add(updated.commit.clone()).unwrap();
    if commit_cid == repo.cid {
        new_blocks.delete(commit_cid).unwrap();
    } else {
        removed_cids.add(repo.cid);
    }

    VerifiedDiff {
        write: writes,
        commit: CommitData {
            cid: updated.cid.clone(),
            rev: updated.commit.rev.clone(),
            since: Some(repo.commit.rev.clone()),
            prev: Some(repo.cid.clone()),
            new_blocks,
            removed_cids,
        },
    }
}
