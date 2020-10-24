#![feature(proc_macro_hygiene, decl_macro)]
#![macro_use]
extern crate rocket;

use rocket_contrib::serve::StaticFiles;

fn main() {
    rocket::ignite()
        .mount("/", StaticFiles::from("tile-server/public"))
        .launch();
}
