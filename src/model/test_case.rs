/// `test_cases` table
pub struct TestCase {
   pub id: uuid::Uuid,

   pub name: String,

   pub inserted_at: chrono::NaiveDateTime,
   pub updated_at: chrono::NaiveDateTime,
}
