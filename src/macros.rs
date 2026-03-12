#[macro_export]
macro_rules! methods {
    ($t:ident [$($x:expr ),* $(,)?]) => {{
            let mut __methods = $t::new();
            $(
                __methods.push($x);
            )*
            __methods
        }};
}

#[macro_export]
macro_rules! hashing_methods {
    [$($x:expr),* $(,)?] => {
        {
            $crate::methods!(HashingMethods[$($x),*])
        }
    };
}

#[macro_export]
macro_rules! modified_images {
    [$($x:expr),* $(,)?] => {
        {
            $crate::methods!(Modifications[$($x),*])
        }
    };
}
