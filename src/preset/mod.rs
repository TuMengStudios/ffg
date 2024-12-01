use std::{fs, path::Path};

use colored::Colorize;
use lazy_static;

lazy_static::lazy_static! {
    pub static  ref rg_home:String = {
      let home_dir =  dirs::home_dir().unwrap();
      match  std::env::var("RG_HOME") {
          Ok(dir) => dir,
          Err(_err) => {

            let path = Path::new(&home_dir).join(".rg");
            if !path.exists() {
                    fs::create_dir_all(&path).unwrap();
            }
            println!("preset RG_HOME {}",path.to_string_lossy().green());
            path.to_str().unwrap().to_owned()
          }
      }
    };

    pub static  ref rg_mirror:String = {
      match std::env::var("RG_MIRROR") {
        Ok(mirror) => {
          println!("use env RG_MIRROR {}",mirror.green());
          mirror
        },
        Err(_) => {
            let mirror:&'static str = "https://go.dev";
            println!("preset RG_MIRROR {}", mirror.magenta());
            mirror.to_owned()
        }
    }
    };

    pub static ref pkgs :String = "packages".to_owned();
}
