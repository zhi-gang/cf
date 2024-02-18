//! The module to wrap MongoDB features

use mongodb::{bson::Document, options::FindOptions, Client, Collection};
use futures::stream::TryStreamExt;

static mut CLIENT: Option<Client> = None;

pub fn init(client: Client) {
    unsafe { CLIENT = Some(client) };
}

pub fn check_init() -> anyhow::Result<&'static Client> {
    match unsafe { &CLIENT } {
        Some(c) => Ok(c),
        None => Err(anyhow::Error::msg("Client is None")),
    }
}

pub async fn insert_doc(db: &'_ str, collection: &'_ str, doc: Document) -> anyhow::Result<()> {
    let client = check_init()?;
    let db = client.database(db);
    let c = db.collection(collection);
    c.insert_one(doc, None).await?;
    Ok(())
}

pub async fn delete_doc(db: &'_ str, collection: &'_ str, filter: Document) -> anyhow::Result<()> {
    let client = check_init()?;
    let db = client.database(db);
    let c:Collection<Document> = db.collection(collection);
    c.delete_one(filter, None).await?;
    Ok(())
}

pub async fn query_doc(db: &'_ str, collection: &'_ str, filter: Document,size:u32) -> anyhow::Result<Vec<Document>> {
    let client = check_init()?;
    let db = client.database(db);
    let c:Collection<Document> = db.collection(collection);
    let option = FindOptions::builder().batch_size(size).build();
    let mut cursor = c.find(filter, option).await?;

    let mut docs = Vec::<Document>::new();
    while let Some(d) = cursor.try_next().await? {
        docs.push(d);
    }
    Ok(docs)
}