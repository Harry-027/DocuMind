// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    env::var("BACKEND_URL").expect("BACKEND_URL not found");
    documind_lib::run()
}
