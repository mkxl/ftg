macro_rules! select {
    ($($futures:expr),*) => {{
        tokio::select! {
            $(value = $futures => value, )*
        }
    }};
}

pub(crate) use select;
