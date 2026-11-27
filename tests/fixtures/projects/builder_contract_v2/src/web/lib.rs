use application::run;
use externo::Client;
use infrastructure::Database;

pub fn handle() {
    run();
    let _ = (Client, Database);
}
