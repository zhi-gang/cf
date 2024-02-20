//! The module to wrap MongoDB features

use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
    options::FindOptions,
    results::{InsertOneResult, UpdateResult},
    Client, Collection,
};

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

pub async fn insert_doc(
    db: &'_ str,
    collection: &'_ str,
    doc: Document,
) -> anyhow::Result<InsertOneResult> {
    let client = check_init()?;
    let db = client.database(db);
    let c = db.collection(collection);
    let r = c.insert_one(doc, None).await?;
    Ok(r)
}

pub async fn delete_doc(db: &'_ str, collection: &'_ str, filter: Document) -> anyhow::Result<()> {
    let client = check_init()?;
    let db = client.database(db);
    let c: Collection<Document> = db.collection(collection);
    c.delete_one(filter, None).await?;
    Ok(())
}

pub async fn update_doc(
    db: &'_ str,
    collection: &'_ str,
    id: Bson,
    doc: Document,
) -> anyhow::Result<UpdateResult> {
    let client = check_init()?;
    let db = client.database(db);
    let c: Collection<Document> = db.collection(collection);
    let filter = doc! {"_id" : id};
    let res = c.update_one(filter.clone(), doc, None).await?;
    Ok(res)
}

pub async fn find_docs(
    db: &'_ str,
    collection: &'_ str,
    filter: Document,
    size: u32,
) -> anyhow::Result<Vec<Document>> {
    let client = check_init()?;
    let db = client.database(db);
    let c: Collection<Document> = db.collection(collection);
    let option = FindOptions::builder().batch_size(size).build();
    let mut cursor = c.find(filter, option).await?;

    let mut docs = Vec::<Document>::new();
    while let Some(d) = cursor.try_next().await? {
        docs.push(d);
    }
    Ok(docs)
}

#[cfg(test)]
mod test {
    use mongodb::bson::doc;

    use super::*;

    #[tokio::test]
    async fn test() {
        let client = Client::with_uri_str("mongodb://127.0.0.1:27017")
            .await
            .expect("connect to db");
        init(client);
        let doc = doc! {"a": 1};
        let doc2 = doc! {"$set":{"a": 2}};
        let res = insert_doc("db", "config", doc).await.expect("insert db");
        println!("{:?}", res.inserted_id);
        let res2 = update_doc("db", "config", res.inserted_id.clone(), doc2)
            .await
            .expect("update");
        println!("update result: {:?}", res2);
        assert_eq!(res2.matched_count, 1);
        let docs = find_docs("db", "config", doc! {"_id":res.inserted_id.clone()}, 2)
            .await
            .expect("find");
        println!("docs: {:?}", docs);
        delete_doc("db", "config", doc! {"_id":res.inserted_id})
            .await
            .expect("delete");
    }

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Configurations {
        key: String,
        value: String,  //json string
    }
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Value {
       host:String,
       port:u32
    }


    #[tokio::test]
    async fn test2() {
        let val = Value {
            host:"localhost".to_string(),
            port :3333
        };
        let val_json = serde_json::to_string(&val).unwrap();
        let doc = Configurations {
            key: "a".to_string(),
            value: val_json,
        };
        let val2 = Value {
            host:"localhost".to_string(),
            port :1111
        };
        let val2_json = serde_json::to_string(&val2).unwrap();
        let client = Client::with_uri_str("mongodb://127.0.0.1:27017").await.expect("connect to db");
        
        let db = client.database("db");
        let c: Collection<Configurations> = db.collection("config");
        let _r = c.insert_one(doc.clone(), None).await.expect("1");
        let f = c.find_one(doc!{"key":"a"}, None).await.expect("2");
        println!("sss {:?}", serde_json::from_str::<Value>(&*f.unwrap().value).unwrap());
        let _u = c.update_many(doc!{"key":"a"}, doc!{"$set": {"value":val2_json}}, None).await.expect("2");
        let f2 = c.find_one(doc!{"key":"a"}, None).await.expect("2");
        println!("sss {:?}", f2);
        c.delete_many(doc!{"key":"a"}, None).await.expect("2");

    }
}
