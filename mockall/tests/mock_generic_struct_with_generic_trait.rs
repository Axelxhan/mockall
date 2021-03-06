// vim: tw=80

use mockall::*;

trait Foo<T: 'static> {
    fn foo(&self, x: T) -> T;
}
mock! {
    Bar<T: 'static, Z: 'static> {}
    trait Foo<T: 'static> {
        fn foo(&self, x: T) -> T;
    }
}

#[test]
fn returning() {
    let mut mock = MockBar::<u32, u64>::new();
    mock.expect_foo()
        .returning(|x| x);
    assert_eq!(5u32, mock.foo(5u32));
}
