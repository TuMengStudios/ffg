use std::{fs, path::Path};

use colored::Colorize;
use lazy_static;

lazy_static::lazy_static! {
    pub static  ref ffg_home:String = {
      let home_dir =  dirs::home_dir().unwrap();
      match  std::env::var("FFG_HOME") {
          Ok(dir) => dir,
          Err(_err) => {
            let path = Path::new(&home_dir).join(".ffg");
            if !path.exists() {
                    fs::create_dir_all(&path).unwrap();
            }
            println!("preset FFG_HOME {}",path.to_string_lossy().green());
            path.to_str().unwrap().to_owned()
          }
      }
    };

    pub static  ref ffg_mirror:String = {
      match std::env::var("FFG_MIRROR") {
        Ok(mirror) => {
          println!("use env FFG_MIRROR {}",mirror.green());
          mirror
        },
        Err(_) => {
            let mirror:&'static str = "https://go.dev";
            println!("preset FFG_MIRROR {}", mirror.magenta());
            mirror.to_owned()
        }
    }
    };

    pub static ref pkgs :String =  {
      let pck = Path::new(&ffg_home.clone()).join("packages");
      if  !pck.exists() {
        std::fs::create_dir_all(&pck).unwrap();
      }
      "packages".to_owned()
    };
}
