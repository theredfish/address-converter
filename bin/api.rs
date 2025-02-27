/// This is just an example file to demonstrate the case  
/// of an API binary.

fn main() {
    println!("API entrypoint");
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn api_test() {
        assert_eq!("api", "api");
    }
}