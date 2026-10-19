//! The first branch whose predicate holds wins.

cfg_if::cfg_if! {
    if #[cfg(any())] {
        fn pick() -> u32 { 1 }
    } else if #[cfg(all())] {
        fn pick() -> u32 { 2 }
    } else if #[cfg(all())] {
        fn pick() -> u32 { 3 }
    } else {
        fn pick() -> u32 { 4 }
    }
}

cfg_if::cfg_if! {
    if #[cfg(any())] {
        fn fallback() -> u32 { 1 }
    } else {
        fn fallback() -> u32 { 2 }
        const ALSO: u32 = 7;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all())] {
        fn only() -> bool { true }
    }
}

cfg_if::cfg_if! {
    if #[cfg(test)] {
        fn testing() -> bool { true }
    } else {
        fn testing() -> bool { false }
    }
}

fn tail_expression() -> u32 {
    cfg_if::cfg_if! {
        if #[cfg(any())] {
            1
        } else {
            let x = 40;
            x + 2
        }
    }
}

#[test]
fn first_true_branch_wins() {
    assert_eq!(tail_expression(), 42);
    assert_eq!(pick(), 2);
    assert_eq!(fallback(), 2);
    assert_eq!(ALSO, 7);
    assert!(only());
    assert!(testing());
}
