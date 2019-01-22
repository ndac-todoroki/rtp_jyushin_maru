/// `rtps` table
pub struct RTP {
   pub id: uuid::Uuid,
   pub serial: i64,
   pub test_case_id: uuid::Uuid,

   pub version: i32,
   pub padding: bool,
   pub extension: bool,
   pub csrc_count: i32,
   pub marker: bool,
   pub payload_type: i32,
   pub timestamp: i64,
   pub ssrc: i32,
   pub payload: Vec<u8>,
   pub received_at: i32,

   pub inserted_at: chrono::NaiveDateTime,
   pub updated_at: chrono::NaiveDateTime,
}
