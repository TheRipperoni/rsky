// use crate::auth_verifier::{AccessFull, AccessStandardIncludeChecks};
// use crate::common::ContentType;
// use crate::models::{ErrorCode, ErrorMessageResponse};
// use crate::repo::aws::s3::S3BlobStore;
// use crate::repo::block_map::BlockMap;
// use crate::repo::cid_set::CidSet;
// use crate::repo::parse::get_and_parse_record;
// use crate::repo::types::{CommitData, RecordWriteDescript, RepoRecord, VerifiedDiff};
// use crate::repo::{find_blob_refs, ActorStore, Repo};
// use aws_config::SdkConfig;
// use chrono::{DateTime, Utc};
// use rocket::http::Status;
// use rocket::response::status;
// use rocket::serde::json::Json;
// use rocket::{Data, State};
// use rsky_lexicon::com::atproto::repo::BlobOutput;
// use rsky_syntax::aturi::AtUri;
// use std::future::Future;
// use time::Date;
// use crate::car::read_car_bytes;

#[rocket::post("/xrpc/com.atproto.repo.importRepo")]
pub async fn import_repo(
    // auth: AccessFull,
    // blob: Data<'_>,
    // content_type: ContentType,
    // s3_config: &State<SdkConfig>,
)
    // -> anyhow::Result<Json<BlobOutput>, status::Custom<Json<ErrorMessageResponse>>>
{
    unimplemented!()
    // match inner_import_repo(auth, blob, content_type, s3_config).await {
    //     Ok(res) => Ok(Json(res)),
    //     Err(error) => {
    //         eprintln!("{error:?}");
    //         let internal_error = ErrorMessageResponse {
    //             code: Some(ErrorCode::InternalServerError),
    //             message: Some(error.to_string()),
    //         };
    //         return Err(status::Custom(
    //             Status::InternalServerError,
    //             Json(internal_error),
    //         ));
    //     }
    // }
}
//
// async fn inner_import_repo(
//     auth: AccessFull,
//     blob: Data<'_>,
//     content_type: ContentType,
//     s3_config: &State<SdkConfig>,
// ) {
//     //Read car stream
//     let now = Utc::now();
//     let rev = String::from("");
//
//     let requester = auth.access.credentials.unwrap().did.unwrap();
//     let mut actor_store = ActorStore::new(
//         requester.clone(),
//         S3BlobStore::new(requester.clone(), s3_config),
//     );
//     let curr_root;
//     let curr_repo = Repo::load(&mut actor_store.storage, curr_root).await;
//     let mut diff: VerifiedDiff;
//     diff.commit.rev =  rev;
//
//     match actor_store
//         .record
//         .get_record(&"repo_root".to_string(), None, None)
//         .await
//     {
//         Ok(_) => {
//             eprintln!("{now}");
//         }
//         Err(e) => {
//             eprintln!("{e}");
//         }
//     }
//     let commit = CommitData {
//         cid: Default::default(),
//         rev: "".to_string(),
//         since: None,
//         prev: None,
//         new_blocks: BlockMap {
//             map: Default::default(),
//         },
//         removed_cids: CidSet {
//             set: Default::default(),
//         },
//     };
//     let is_create = Some(false);
//     match actor_store.storage.apply_commit(commit, is_create).await {
//         Ok(_) => {}
//         Err(e) => {
//             eprintln!("{e}");
//         }
//     }
// }
//
// async fn test(
//     did: String,
//     write: RecordWriteDescript,
//     blocks: BlockMap,
//     roots: CidSet,
//     actor_store: &mut ActorStore,
//     rev: String,
//     now: String
// ) {
//     match write {
//         RecordWriteDescript::Create(record) => {
//             let uri = AtUri::make(did, Some(record.collection), Some(record.rkey)).unwrap();
//             let parsed_record: RepoRecord;
//             let parsed = get_and_parse_record(&blocks, record.cid).unwrap();
//             parsed_record = parsed.record;
//
//             let index_record = actor_store.record.index_record(
//                 uri.to_string(),
//                 record.cid,
//                 Some(parsed_record.clone()),
//                 Some(record.action),
//                 rev,
//                 Some(now),
//             ).await;
//             let record_blobs = find_blob_refs(parsed_record);
//
//
//         }
//         RecordWriteDescript::Update(record) => {}
//         RecordWriteDescript::Delete(record) => {
//             let uri = AtUri::make(did, Some(record.collection), Some(record.rkey)).unwrap();
//             match actor_store.record.delete_record(uri.to_string()).await {
//                 Ok(_) => {}
//                 Err(e) => {
//                     eprintln!("{e}");
//                 }
//             }
//         }
//     }
// }
