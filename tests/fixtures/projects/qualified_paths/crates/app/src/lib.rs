pub fn run() {
    let _ = domain::Value;
    let _: domain::Other = domain::Other;
    let _ = std::mem::size_of::<u8>();
}

fn helper() {
    use domain::Renamed;
    let _ = Renamed;
}

#[cfg(test)]
mod tests {
    use domain::Secret;

    #[test]
    fn test_only() {
        let _ = Secret;
    }
}
