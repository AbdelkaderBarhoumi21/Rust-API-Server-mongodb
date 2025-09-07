use std::{env, str::FromStr, time::SystemTime};

use chrono::Utc;
use futures_util::StreamExt;
use mongodb::{
    Client, Collection,
    bson::{DateTime, datetime::Error, doc, from_document, oid::ObjectId},
    results::{InsertOneResult, UpdateResult},
};

use crate::models::{
    booking_model::{Booking, FullBooking},
    dog_model::Dog,
    owner_model::Owner,
};

/// Database struct holds typed collections for booking, dog, and owner.
/// Each collection is strongly typed with its respective Rust struct,
/// which makes serialization/deserialization easier and safer.
pub struct Database {
    booking: Collection<Booking>,
    dog: Collection<Dog>,
    owner: Collection<Owner>,
}

impl Database {
    /// Initialize the database connection.
    /// It checks if `MONGO_URI` exists as an environment variable.
    /// If not, it falls back to a default local URI.
    /// Then, it connects to the "dog_walking" database
    /// and stores references to the three collections.
    pub async fn init() -> Self {
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        // Create a new MongoDB client from the connection string.
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("dog_walking");

        // Typed collections
        let booking: Collection<Booking> = db.collection("booking");
        let dog: Collection<Dog> = db.collection("dog");
        let owner: Collection<Owner> = db.collection("owner");

        Database {
            booking,
            dog,
            owner,
        }
    }

    /// Insert a new owner into the "owner" collection.
    /// Returns the result of the insertion (including the inserted_id).
    pub async fn create_owner(&self, owner: Owner) -> Result<InsertOneResult, Error> {
        let result = self
            .owner
            .insert_one(owner) // Insert the Owner struct into MongoDB
            .await
            .ok()
            .expect("Error creating owner");

        Ok(result)
    }

    /// Insert a new dog into the "dog" collection.
    pub async fn create_dog(&self, dog: Dog) -> Result<InsertOneResult, Error> {
        let result = self
            .dog
            .insert_one(dog)
            .await
            .ok()
            .expect("Error creating dog");

        Ok(result)
    }

    /// Insert a new booking into the "booking" collection.
    pub async fn create_booking(&self, booking: Booking) -> Result<InsertOneResult, Error> {
        let result = self
            .booking
            .insert_one(booking)
            .await
            .ok()
            .expect("Error creating booking");

        Ok(result)
    }

    /// Cancel a booking by updating its "cancelled" field to true.
    /// Takes the booking_id as a &str, parses it to ObjectId,
    /// and runs an update operation.
    pub async fn cancel_booking(&self, booking_id: &str) -> Result<UpdateResult, Error> {
        let result = self
            .booking
            .update_one(
                // Filter: find by ObjectId
                doc! {"_id":ObjectId::from_str(booking_id).expect("Failed to parse booking id")},
                // Update: set "cancelled" = true
                doc! {
                    "$set":doc! {
                        "cancelled":true
                    }
                },
            )
            .await
            .ok()
            .expect("Error cancelling booking");

        Ok(result)
    }

    /// Get all upcoming bookings (not cancelled, start_time >= now).
    /// The query uses an aggregation pipeline to:
    /// 1. $match: filter only active bookings in the future
    /// 2. $lookup: join with owner collection to get owner details
    /// 3. $unwind: flatten the "owner" array into a single object
    /// 4. $lookup: join with dog collection to fetch all dogs belonging to the owner
    pub async fn get_bookings(&self) -> Result<Vec<FullBooking>, Error> {
        let now: SystemTime = Utc::now().into();

        let mut results = self
            .booking
            .aggregate(vec![
                // Step 1: Filter only bookings that are not cancelled
                // and whose start_time is greater or equal to now.
                doc! {
                    "$match" :{
                        "cancelled":false,
                        "start_time":{
                            "$gte":DateTime::from_system_time(now)
                        }
                    }
                },
                // Step 2: Lookup to join booking.owner with owner._id
                doc! {
                    "$lookup":doc! {
                        "from":"owner",
                        "localField":"owner",
                        "foreignField": "_id",
                        "as" : "owner"
                    }
                },
                // Step 3: Unwind the owner array so that "owner": [ {...} ]
                // becomes "owner": { ... }
                doc! {
                    "$unwind":doc! {
                        "path":"$owner"
                    }
                },
                // Step 4: Lookup dogs whose "owner" field matches owner._id
                // and put them in an array called "dogs".
                doc! {
                    "$lookup":{
                        "from":"dog",
                        "localField":"owner._id",
                        "foreignField":"owner",
                        "as":"dogs"
                    }
                },
            ])
            .await
            .ok()
            .expect("Error getting bookings");

        let mut bookings: Vec<FullBooking> = Vec::new();

        // Iterate over the aggregation cursor (stream of documents).
        while let Some(result) = results.next().await {
            match result {
                // If the document was retrieved successfully:
                Ok(doc) => {
                    // Deserialize BSON document into FullBooking struct.
                    let booking: FullBooking =
                        from_document(doc).expect("Error converting document to FullBookin");
                    bookings.push(booking); // Add to results vector
                }
                // If there was an error while fetching the document:
                Err(err) => panic!("Error getting booking: {}", err),
            }
        }

        Ok(bookings)
    }
}

/*

Collection booking
{
  "_id": 1,
  "start_time": "2025-09-07T10:00:00Z",
  "cancelled": false,
  "owner": { "_id": 101, "name": "Alice" }
}
-------------------------------------------
Collection dog
{ "_id": 201, "name": "Rex", "owner": 101 }
{ "_id": 202, "name": "Bella", "owner": 101 }
{ "_id": 203, "name": "Max", "owner": 102 }

 ------------------------------------------------
{
  "_id": 1,
  "start_time": "2025-09-07T10:00:00Z",
  "cancelled": false,
  "owner": { "_id": 101, "name": "Alice" },
  "dogs": [
    { "_id": 201, "name": "Rex", "owner": 101 },
    { "_id": 202, "name": "Bella", "owner": 101 }
  ]
}


*/
