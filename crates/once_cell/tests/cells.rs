use once_cell::race::OnceBox;
use once_cell::sync::{Lazy, OnceCell};
use std::sync::atomic::{AtomicUsize, Ordering};

static CALLS: AtomicUsize = AtomicUsize::new(0);
static LAZY: Lazy<Vec<u32>> = Lazy::new(|| {
    CALLS.fetch_add(1, Ordering::SeqCst);
    vec![1, 2, 3]
});
static BOXED: OnceBox<String> = OnceBox::new();

#[test]
fn cells_initialise_once() {
    let cell = OnceCell::new();
    assert!(cell.get().is_none());
    assert_eq!(*cell.get_or_init(|| 5), 5);
    assert_eq!(cell.set(6), Err(6));
    assert_eq!(cell.get(), Some(&5));
    let fallible: OnceCell<u8> = OnceCell::new();
    assert_eq!(
        fallible.get_or_try_init(|| Err::<u8, &str>("no")),
        Err("no")
    );
    assert!(
        fallible.get().is_none(),
        "a failed initialiser leaves the cell unset"
    );
    assert_eq!(fallible.get_or_try_init(|| Ok::<u8, &str>(9)), Ok(&9));
    assert_eq!(fallible.clone().get(), Some(&9));
    let threads: Vec<_> = (0..8).map(|_| std::thread::spawn(|| LAZY.len())).collect();
    for t in threads {
        assert_eq!(t.join().unwrap(), 3);
    }
    assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(BOXED.get_or_init(|| Box::new("a".into())), "a");
    assert!(BOXED.set(Box::new("b".into())).is_err());
    assert_eq!(BOXED.get().map(String::as_str), Some("a"));
}
