mod params;
mod print;
mod wormod;

use params::*;
use wormod::run;

fn main() {
    if let Err(e) = run(Params::parse().validate()) {
        print_err!("{}", e.to_string());
        std::process::exit(1);
    }
}
