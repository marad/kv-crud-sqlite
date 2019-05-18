use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct MyData {
    id: u32,
    data: u32,
}

impl Entity<u32> for MyData {
    fn get_id(&self) -> u32 {
        self.id
    }
}

#[test]
fn should_be_able_to_save_and_read() {
    // given
    let mut storage = SqliteStorage::new(":memory:").unwrap();
    let entity = MyData { id: 42, data: 24 };

    // when
    storage.save(&entity).unwrap();
    let read_entity = storage.find_by_id(&42).unwrap();

    // then
    assert_eq!(entity, read_entity);
}

#[test]
fn should_result_in_an_error_when_searching_for_unset_key() {
    // given
    let mut storage = SqliteStorage::new(":memory:").unwrap();

    // when
    let result: Result<MyData> = storage.find_by_id(&42);

    // then
    match result {
        Err(Error::NotFound(id)) => assert_eq!("42", id),
        _ => assert!(false, "Invalid result!"),
    };
}

#[test]
fn should_overwrite_values_upon_save() {
    // given
    let mut storage = SqliteStorage::new(":memory:").unwrap();
    let entity = MyData { id: 42, data: 24 };
    storage.save(&entity).unwrap();

    // when
    let updated_entity = MyData { id: 42, data: 100 };
    storage.save(&updated_entity).unwrap();

    // then
    let loaded = storage.find_by_id(&42).unwrap();
    assert_eq!(updated_entity, loaded);
}
