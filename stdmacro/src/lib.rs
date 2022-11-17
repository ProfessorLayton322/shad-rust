#![forbid(unsafe_code)]

#[macro_export]
macro_rules! deque {
    () => {
        __VecDeque::new()
    };
    ($($x:expr),+) => {
        __VecDeque::from([$($x),+])
    };
    ($elem:expr; $n:expr) => {
        __VecDeque::from([$elem; $n])
    };
}

#[macro_export]
macro_rules! sorted_vec {
    () => {
        Vec::new()
    };
    ($($x:expr),+) => {
        {
            let mut ans = Vec::from([$($x),+]);
            ans.sort();
            ans
        }
    };
}

#[macro_export]
macro_rules! map {
    () => {
        __HashMap::new()
    };
    ($($x:expr => $y:expr),+) => {
        __HashMap::from([$(($x, $y)),+])
    };
    ($($x:expr => $y:expr),+, $(,)?) => {
        __HashMap::from([$(($x, $y)),+])
    }
}
