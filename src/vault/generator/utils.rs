

// TODO: use chrono
pub fn generate_random_date() -> String {
    let y =   rand::random_range(0..=9);
    let m = f(rand::random_range(1..=12));
    let d = f(rand::random_range(1..=28));


    format!("202{y}-{m:2}-{d:2}")
}

// TODO: use chrono
pub fn generate_random_time() -> String {
    let h = f(rand::random_range(0..24));
    let m = f(rand::random_range(0..60));
    let s = f(rand::random_range(0..60));


    format!("{h}_{m}_{s}")
}

fn f(n: usize) -> String {
    format!("{n:0>2}")
}
