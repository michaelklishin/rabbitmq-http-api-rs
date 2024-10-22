#[macro_export]
macro_rules! path_one_part {
    ($val:expr, $part:literal) => {
        $val.push('/');
        $val.push_str($part);
    };
    ($val:expr, $part:expr) => {
        let part = $part.as_ref();
        let encoded =
            percent_encoding::utf8_percent_encode(part, percent_encoding::NON_ALPHANUMERIC);
        $val.push('/');
        $val.extend(encoded);
    };
}

#[macro_export]
macro_rules! path {
    ($part1:expr, $($part:expr),+) => {{
        let mut url = String::from($part1);
        $(
            $crate::path_one_part!(&mut url, $part);
        )+
        url
    }}
}
