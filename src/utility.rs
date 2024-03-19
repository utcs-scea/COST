#[macro_export]
macro_rules! const_switch_bool {
    ($b: expr, |$i: ident| $s:stmt) => {
        match $b {
            true => {
                const $i: bool = true;
                $s
            }
            false => {
                const $i: bool = false;
                $s
            }
        }
    };
}
