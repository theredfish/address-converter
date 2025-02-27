fn main() {
    println!("CLI entrypoint");
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn cli_test() {
        assert_eq!("cli", "cli");
    }
}