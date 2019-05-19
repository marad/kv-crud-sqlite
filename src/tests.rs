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

#[test]
fn should_find_with_pagination() {
    // given
    let mut  storage = SqliteStorage::new(":memory:").unwrap();
    let a = MyData { id: 42, data: 24 };
    let b = MyData { id: 53, data: 35 };
    let c = MyData { id: 64, data: 46 };

    storage.save(&a).unwrap();
    storage.save(&b).unwrap();
    storage.save(&c).unwrap();

    // when
    let first = storage.find_all_with_page(&Page::new(0, 1)).unwrap();
    let second = storage.find_all_with_page(&Page::new(1, 2)).unwrap();
    let third = storage.find_all_with_page(&Page::new(0, 3)).unwrap();

    // then
    assert_eq!(vec![a.clone()], first);
    assert_eq!(vec![c.clone()], second);
    assert_eq!(vec![a, b, c], third);
}

#[test]
fn should_delete_entry_by_id() {
    // given a storage with one entity
    let mut  storage = SqliteStorage::new(":memory:").unwrap();
    let entity = MyData { id: 42, data: 24 };
    storage.save(&entity).unwrap();

    // when removing by id
    <SqliteStorage as Delete<u32, MyData>>::remove_by_id(&mut storage, &entity.id).unwrap();

    // then entity should be removed
    match <SqliteStorage as Read<u32, MyData>>::find_by_id(&mut storage, &42u32) {
        Ok(_) => panic!("Entity shouldn't be in storage"),
        Err(_) => assert!(true),
    }


}