#[test]
fn capacity_pool_test() {
    use crate::backbone::capacity_pool::CapacityPool;

    let capp = CapacityPool::new(10);
    let mut token_vec = vec![
        capp.take_token().unwrap(),
        capp.take_token().unwrap(),
        capp.take_token().unwrap(),
        capp.take_token().unwrap(),
        capp.take_token().unwrap(),
    ];
    assert_eq!(capp.used(), 5);
    token_vec.pop();
    assert_eq!(capp.free(), 6);
    drop(token_vec);
    assert_eq!(capp.max_capacity(), capp.free());
}
