use std::fs::File;
use std::io::{BufRead, BufReader};
pub fn findmd5sum (fullpath: String) -> (u32, String, String) {
// pub fn findmd5sum (fullpath: String) -> String {
     let mut errcode: u32 = 0;
     let mut errstring: String = "all good".to_string();
     let mut hashval: String = "-".to_string();
     match File::open(&fullpath) {
       Ok(file) => {
//       let f = File::open(fullpath).unwrap();
         // Find the length of the file
         let len = file.metadata().unwrap().len();
     // Decide on a reasonable buffer size (1MB in this case, fastest will depend on hardware)
         let buf_len = len.min(1_000_000) as usize;
         let mut buf = BufReader::with_capacity(buf_len, file);
         let mut context = md5::Context::new();
         loop {
                // Get a chunk of the file
                match buf.fill_buf() {
                  Ok(part) => {
//                  let part = buf.fill_buf().unwrap();
                    // If that chunk was empty, the reader has reached EOF
                    if part.is_empty() {
                        break;
                    }
                    // Add chunk to the md5
                    context.consume(part);
                    // Tell the buffer that the chunk is consumed
                    let part_len = part.len();
                    buf.consume(part_len);
                  }
                  Err(error1) => {
                    errcode = 2;
                    errstring = format!("unable to get buffer file {} error: {}", fullpath, error1);
                    break;
                  }
                };
         }
         if errcode == 0 {
             let digest = context.finalize();
             hashval = format!("{:x}", digest);
         }
       }
       Err(error) => {
         errcode = 1;
         errstring = format!("unable to open file {} error: {}", fullpath, error);
       }
     };
    (errcode, errstring, hashval)
//     hashval
}
