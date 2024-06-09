use std::env;

fn main() {
    println!("cargo::rerun-if-changed=.env");
    if dotenvy::dotenv().is_ok(){
        println!("cargo::rustc-env=DISCORD_CLIENT_ID={}",env::var("DISCORD_CLIENT_ID").unwrap());
    }
}