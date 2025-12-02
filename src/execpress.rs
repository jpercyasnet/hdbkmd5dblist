use std::path::Path;
use rusqlite::Connection;
use crate::connectdb;
#[derive(Debug)]
struct Outpt {
    name: String,
}
#[derive(Debug)]
struct Bkup {
      rowid: u64,
      refname: String,
      filename: String,
      dirname: String,
      filesize: u64,
      filedate: String,
      md5sum: Option<String>,
      locations: Option<String>,
      notes: Option<String>,
}

pub fn execpress (conn: &Connection, hddir: String, seqfile: String, startl: String, endl: String, targetdir: String, refname: String, targetname: String, bde_bool: bool) -> (u32, String) {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good and now process execution".to_string();
     let mut bolok = true;
     if bde_bool {
         if let Err(e) = connectdb(&conn) {
             errstring = format!("data base error: {}", e);
             errcode = 1;
             bolok = false;
         } else {
             // get list of all tables in database
             let strval = "SELECT name FROM sqlite_master WHERE type = \"table\" ";
             match conn.prepare(strval) {
                Ok(mut ss) => {
                   match ss.query_map([], |row| {
                      Ok(Outpt {
                         name: row.get(0)?,
                      })
                   }) {
                       Ok(ss_iter) => {
                            let mut numtables = 0;
                            let mut tablena: String = "---".to_string();
                            for si in ss_iter {
                               numtables = numtables + 1;
                               let sii = si.unwrap();
                               tablena = sii.name.to_string();
                            }
                            // check to see if blubackup is the only table
                            if numtables == 0 {
                                errstring  = format!("no tables in database: tablena: {}", tablena);
                                errcode = 1;
                                bolok = false;
                            } else if !(numtables == 1) {  
                                errstring  = format!("{} tables in database: last tablena: {}", numtables, tablena);
                                errcode = 2;
                                bolok = false;
                            } else {
                                if !(tablena == "blubackup") {
                                    errstring  = format!("invalid table of {}", tablena);
                                    errcode = 3;
                                    bolok = false;
                                } else {
                                    match conn.prepare("SELECT GROUP_CONCAT(NAME,',') FROM PRAGMA_TABLE_INFO('blubackup')") {
                                        Ok(mut ssx) => {
                                            match ssx.query_map([], |row| {
                                               Ok(Outpt {
                                                   name: row.get(0)?,
                                               })
                                            }) {
                                               Ok(ssx_iter) => {
                                                   for six in ssx_iter {
                                                      let _siix = six.unwrap();
                                                   }
                                               }
                                               Err(err) => {
                                                   errstring  = format!("Error doing sql select group {:?}", err);
                                                   errcode = 4;
                                                   bolok = false; 
                                               }
                                            };
                                        }
                                        Err(err) => {
                                            errstring  = format!("Error doing sql select group {:?}", err);
                                            errcode = 5;
                                            bolok = false;
                                        } 
                                    }        
                                }
                            }                     
                       }
                       Err(err) => {
                            errstring  = format!("Error doing sql select group {:?}", err);
                            errcode = 6;
                            bolok = false;
                       }
                   }
                }
                Err(err) => {
                     errstring  = format!("Error doing sql select name {:?}", err);
                     errcode = 7;
                     bolok = false;
                } 
             };
         }
     }
     let mut from_int1: i64 = 0;
     if bolok {
         if !Path::new(&targetdir).exists() {
             errstring = "the target directory does not exist".to_string();
             errcode = 8;
             bolok = false;
         } else {
             if !Path::new(&seqfile).exists() {
                 errstring = "the harddrive seq file does not exist".to_string();
                 errcode = 9;
                 bolok = false;
             } else {
                 if startl.len() == 0 {
                     errstring = "Start Line has no value".to_string();
                     errcode = 10;
                     bolok = false;
                 } else {
                     let from_int: i64 = startl.parse().unwrap_or(-99);
                     if from_int == -99 {
                         errstring = "Start Line is not an integer".to_string();
                         errcode = 11;
                         bolok = false;
                     } else if from_int < 1 {
                         errstring = "Start Line not positive integer".to_string();
                         errcode = 12;
                         bolok = false;
                     } else {
                         from_int1 = from_int;
                     }
                 }
             }
         }
     }
     if bolok {
         if endl.len() == 0 {
             errstring = "End Line has no value".to_string();
             errcode = 13;
             bolok = false;
         } else {
             let to_int: i64 = endl.parse().unwrap_or(-99);
             if to_int == -99 {
                 errstring = "End Line is not an integer".to_string();
                 errcode = 14;
                 bolok = false;
             } else if to_int < 1 {
                 errstring = "End Line not positive integer".to_string();
                 errcode = 15;
                 bolok = false;
             } else {
                 if to_int < from_int1 {
                     errstring = "End Line less than Start Line".to_string();
                     errcode = 16;
                     bolok = false;
                 }
             }
         }
     }
     if bolok {
         if refname.len() < 4 {
             errstring = "the reference name is less than 4 characters".to_string();
             errcode = 17;
         } else {
             if !targetname.contains(".") { 
                 errstring = "target name does not have a file type (ie xx.xxx)".to_string();
                 errcode = 18;
             } else {
                 let lrperpos = targetname.rfind(".").unwrap();
                 if (targetname.len() - lrperpos) < 4 {
                     errstring = "target name does not have a valid type (must be at least 3 characters".to_string();
                     errcode = 19;
                 } else {
                     let lfperpos = targetname.find(".").unwrap();
                     if lfperpos < 3 {
                         errstring = "target name is least than 3 characters".to_string();
                         errcode = 20;
                     } else {
                         let targetfullname: String = format!("{}/{}", targetdir, targetname);
                         if Path::new(&targetfullname).exists() {
                             errstring = "the target output file already exists".to_string();
                             errcode = 21;
                         }
                     }
                 }
             }
         }
     }
     if bolok {
         if !Path::new(&hddir).exists() {
             errstring = "the hard drive directory does not exist".to_string();
             errcode = 22;
//             bolok = false;
         } 
     }
     (errcode, errstring)
}

