mod params;
mod print;
mod wormod;

use params::*;
use wormod::run;

fn main() {
    if let Err(e) = run(Params::parse().validate()) {
        exit_err!(("{}", e.to_string()));
    }
}
