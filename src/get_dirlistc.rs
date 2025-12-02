extern crate exif;
use std::fs;
use std::path::Path;
use std::env;
extern crate walkdir;
use walkdir::WalkDir;
extern crate chrono;
use chrono::prelude::*;

pub fn get_dirlistc (current_dir: String, getdiritemb: bool) -> (u32, String, String, Vec<String>) {
    let errcode: u32;
    let errstring: String;
    let mut strpath: String = current_dir;
    let mut filtype;
    let mut itemdesc;
    let mut listitems: Vec<String> = Vec::new();

    let mut new_path = Path::new(&strpath);
    if !new_path.exists() {
        strpath = env::current_dir().expect("REASON").into_os_string().into_string().unwrap();
        new_path = Path::new(&strpath);
    }

    let mut numentry = 0;
    let listin: String;
    match new_path.parent() {
       Some(val) => {
          listin = format!("DIR | {} | ..parent", val.to_str().unwrap());
       },
       None => {
          listin = "xxx | no parent | ..".to_string();
       }
    }
    listitems.push(listin);
    match fs::read_dir(&new_path) {
       Ok(_result) => {
         for entry1 in fs::read_dir(&new_path).unwrap() {
             let entry = entry1.unwrap();
             if let Ok(metadata) = entry.metadata() {
                 if let Ok(file_name) = entry.file_name().into_string() {
                     if metadata.is_file() {
                         filtype = format!("file");
                         let datetime: DateTime<Local> = metadata.modified().unwrap().into();
                         itemdesc = format!("{}  {}", datetime.format("%Y-%m-%d %T"), metadata.len());
                     } else if metadata.is_dir() {
                         let path = format!("{}/{}", strpath, file_name);
                         filtype = format!("DIR");
                         if getdiritemb {
                             itemdesc = format!("{} items", WalkDir::new(path).into_iter().count());
                         } else {
                             itemdesc = "-- items".to_string();
                         }
                     } else {
                         filtype = format!("{:?}", metadata.file_type());
                         itemdesc = " ".to_string();
                     }
                     let listival = filtype + " | " + &file_name + " | " + &itemdesc;
                     listitems.push(listival);
                     numentry = numentry + 1;
                 }
             }
         }
//         if numentry > 0 {
             errstring = format!("{} items in directory ", numentry);
             errcode = 0;
//         } else {
//             errstring = "********* Directory 1: directory has no items **********".to_string();
//             errcode = 1;
//         }
       },
       Err(err) => {
         errcode = 2;
         errstring = format!("error of {} getting directory information for {}", err, strpath);
       }
    }
    (errcode, errstring, strpath, listitems)
}
