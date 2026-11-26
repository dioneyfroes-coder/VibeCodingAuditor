use domain::Value;

mod other;

pub const PRODUCTION_MARKER: &str = "linha de producao repetida deliberadamente para permanecer visivel";

pub fn run() {
    let _ = Value;
    other::touch();
}

#[cfg(test)]
mod tests {
    use domain::Other;

    #[test]
    fn only_test_line() {
        let shared_prefix = "valor compartilhado exclusivo entre testes para duplicacao";
        let _ = (shared_prefix, Other);
    }
}
