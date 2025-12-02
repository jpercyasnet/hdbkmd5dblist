use iced::widget::{Column, button, column, row, text_input, text, horizontal_space,
                   container, checkbox, progress_bar};
use iced::{Alignment, Element, Task, Length, Color, Theme};
use iced::futures;
use futures::channel::mpsc;
extern crate chrono;
use std::path::Path;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use std::time::Duration as timeDuration;
use std::time::Instant as timeInstant;
use std::thread::sleep;
use chrono::prelude::*;
use chrono::Local;
use rusqlite::{Connection, Result};
use rfd::FileDialog;

extern crate walkdir;
use walkdir::WalkDir;

mod get_winsize;
mod inputpress;
mod execpress;
mod findmd5sum;
mod connectdb;
use connectdb::connectdb;
use get_winsize::get_winsize;
use inputpress::inputpress;
use execpress::execpress;
use findmd5sum::findmd5sum;

pub fn main() -> iced::Result {

     let mut widthxx: f32 = 1350.0;
     let mut heightxx: f32 = 750.0;
     let (errcode, _errstring, widtho, heighto) = get_winsize();
     if errcode == 0 {
         if widtho > 1920 {
             widthxx = 1920.0;
         } else {
             widthxx = widtho as f32 - 20.0;
         }
         if heighto > 1080 {
             heightxx = 1080.0;
         } else {
             heightxx = heighto as f32 - 75.0;
         }
     }
     iced::application(Hdbkmd5dblist::title, Hdbkmd5dblist::update, Hdbkmd5dblist::view)
        .window_size((widthxx, heightxx))
        .theme(|_| Theme::SolarizedDark)
        .run_with(Hdbkmd5dblist::new)
}

struct Hdbkmd5dblist {
    hddir: String,
    mess_color: Color,
    msg_value: String,
    refname: String,
    targetdir: String,
    targetname: String,
    startl: String,
    endl: String,
    dbfile: String,
    seqfile: String,
    rpd_bool: bool,
    bde_bool: bool,
    dbconn: Connection,
    do_progress: bool,
    progval: f64,
    tx_send: mpsc::UnboundedSender<String>,
    rx_receive: mpsc::UnboundedReceiver<String>,
}

#[derive(Debug, Clone)]
enum Message {
    HddirPressed,
    MakeseqPressed,
    TestsizePressed,
    DBfilePressed,
    TargetdirPressed,
    RefnameChanged(String),
    TargetnameChanged(String),
    StartlineChg(String),
    EndlineChg(String),
    SeqfilePressed,
    ExecPressed,
    ExecxFound(Result<Execx, Error>),
    ProgressPressed,
    ProgRtn(Result<Progstart, Error>),
    RemovePrefixDir(bool),
    BkupDataBaseEval(bool),
}

impl Hdbkmd5dblist {
    fn new() -> (Self, iced::Task<Message>) {
        let (tx_send, rx_receive) = mpsc::unbounded();
        (  Hdbkmd5dblist {
                       hddir: "--".to_string(),
                       msg_value: "no message".to_string(),
                       targetdir: "--".to_string(),
                       dbfile: "--".to_string(),
                       dbconn: Connection::open_in_memory().unwrap(),
                       seqfile: "--".to_string(),
                       mess_color: Color::from([0.5, 0.5, 1.0]),
                       refname: "--".to_string(),
                       startl: "0".to_string(),
                       endl: "0".to_string(),
                       rpd_bool: false, 
                       bde_bool: false, 
                       targetname: "--".to_string(),
                       do_progress: false,
                       progval: 0.0,
                       tx_send,
                       rx_receive,
          },
          Task::none()
        )
    }

    fn title(&self) -> String {
        String::from("HD & BK file list with md5sum & DB eval")
    }

    fn update(&mut self, message: Message) -> Task<Message>  {
        match message {
            Message::HddirPressed => {
               let mut inputstr: String = self.hddir.clone();
               if !Path::new(&inputstr).exists() {
                   if Path::new(&self.targetdir).exists() {
                       inputstr = self.targetdir.clone();
                   }
               }
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.hddir = newinput;
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::TestsizePressed => {
               let mut bok: bool = true;
               let mut from_int1: i64 = 0;
               let mut to_int1: i64 = 0;
               let mut linenum: u64 = 0;
               let mut totalsz: i64 = 0;
               let mut linehd = String::new();
               if !Path::new(&self.seqfile).exists() {
                   self.msg_value =  "seq file does not exist".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   bok = false;
               } else {
                   if self.startl.len() == 0 {
                       self.msg_value = "Start Line has no value".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       let from_int: i64 = self.startl.parse().unwrap_or(-99);
                       if from_int > 0 {
                           from_int1 = from_int;
                       } else if from_int == -99 {
                            self.msg_value =  "Start Line is not an integer".to_string();
                            self.mess_color = Color::from([1.0, 0.0, 0.0]);
                            bok = false;
                       } else {
                            self.msg_value =  "Start Line not positive integer".to_string();
                            self.mess_color = Color::from([1.0, 0.0, 0.0]);
                            bok = false;
                       }
                   }
               }
               if bok {
                   if self.endl.len() == 0 {
                       self.msg_value = "End Line has no value".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       let to_int: i64 = self.endl.parse().unwrap_or(-99);
                       if to_int > 0 {
                            to_int1 = to_int;
                            if to_int1 < from_int1 {
                                self.msg_value =  "End Line less than Start Line".to_string();
                                self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                bok = false;
                            }
                       } else if to_int == -99 {
                            self.msg_value =  "End Line is not an integer".to_string();
                            self.mess_color = Color::from([1.0, 0.0, 0.0]);
                            bok = false;
                       } else {
                            self.msg_value =  "End Line not positive integer".to_string();
                            self.mess_color = Color::from([1.0, 0.0, 0.0]);
                            bok = false;
                       }
                   }
               }
               if bok {
                   let file = File::open(self.seqfile.clone()).unwrap();
                   let mut reader = BufReader::new(file);
                   loop {
                        match reader.read_line(&mut linehd) {
                            Ok(bytes_read) => {
                               if bytes_read == 0 {
                                   break;
                               }
                               linenum = linenum + 1;
                               let veclinea: Vec<&str> = linehd.split("|").collect();
                               if veclinea.len() != 5 {
                                   self.msg_value =  "seq file not valid 3".to_string();
                                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                   bok = false;
                                   break;
                               } else {
                                   let inline: i64 = veclinea[0].parse().unwrap_or(-99);
                                   if inline == -99 {
                                       self.msg_value =  "inline is not an integer".to_string();
                                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                       bok = false;
                                       break;
                                   } else if inline < 1 {
                                       self.msg_value =  "inline is less than 1".to_string();
                                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                       bok = false;
                                       break;
                                   }
                                   if inline >= from_int1 {
                                       if inline > to_int1 {
                                           break;
                                       }
                                       let filesz: i64 = veclinea[2].parse().unwrap_or(-99);
                                       if filesz == -99 {
                                           self.msg_value =  format!("in file size {} is not an integer: {}", veclinea[2], linehd);
                                           self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                           bok = false;
                                           break;
                                       } else if filesz < 0 {
                                           self.msg_value =  "file size is less than 0".to_string();
                                           self.mess_color = Color::from([1.0, 0.0, 0.0]);
                                           bok = false;
                                           break;
                                       }
                                       totalsz = totalsz + filesz;
                                   }
                               }
                               linehd.clear();
                            }
                            Err(err) => {
                               self.msg_value = format!("error of {} reading {}", err, self.seqfile);
                               self.mess_color = Color::from([1.0, 0.0, 0.0]);
                               bok = false;
                               break;
                            }
                        }
                   }
               }
               if bok {
                   self.msg_value =  format!("seq file from {} to {} length of {}, size is {:.3}gb",  
                                                from_int1, to_int1, (1+to_int1-from_int1), (totalsz as f64 /1000000000.0));
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               }
               Task::none()
            }
            Message::MakeseqPressed => {
               if !Path::new(&self.targetdir).exists() {
                   self.msg_value =  "taget directory does not exist".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               } else if !Path::new(&self.hddir).exists() {
                   self.msg_value =  "hard drive directory does not exist".to_string();
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               } else {
                   let mut outseq: u32 = 1;
                   let mut seqout: String = format!("{}/seqfile{:02}.txt", self.targetdir, outseq);
                   loop {
                       if !Path::new(&seqout).exists() {
                           break;
                       } else {
                           outseq = outseq + 1;
                           seqout = format!("{}/seqfile{:02}.txt", self.targetdir, outseq);
                       }
                   }          
                   let mut seqfile = File::create(seqout.clone()).unwrap();
                   let mut numrows: u64 = 0;
                   let mut totalsz: u64 = 0;
                   for entryx in WalkDir::new(&self.hddir).into_iter().filter_map(|e| e.ok()) {
                        if let Ok(metadata) = entryx.metadata() {
                            if metadata.is_file() {
                                numrows = numrows + 1;
                                totalsz = totalsz + metadata.len() as u64;
                                let datetime: DateTime<Local> = metadata.modified().unwrap().into();
                                let file_date = format!("{}.000", datetime.format("%Y-%m-%d %T")); 
                                let stroutput = format!("{}|{}|{}|{}|",
                                                      numrows,
                                                      entryx.path().display(),
                                                      metadata.len(),
                                                      file_date);
                                writeln!(&mut seqfile, "{}", stroutput).unwrap();
                            }
                        }
                   }
                   self.seqfile = seqout.to_string();
                   self.startl = "1".to_string();
                   self.endl = format!("{}", numrows);
                   self.msg_value =  format!("create {} seq file with total size of {:.3}gb", seqout, (totalsz as f64 /1000000000.0));
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               }
               Task::none()
            }
            Message::DBfilePressed => {
               let mut bok: bool = true;
               let mut newdb: String = " ".to_string();
               if Path::new(&self.dbfile.clone()).exists() {
                   let newfile = FileDialog::new()
                              .set_file_name(self.dbfile.clone())
                              .pick_file();
                   if newfile == None {
                       self.msg_value =  "error getting db file -- possible cancel key hit".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       newdb = newfile.as_ref().expect("REASON").display().to_string();
                   }
               } else {
                   let mut inputstr: String = self.targetdir.clone();
                   if !Path::new(&inputstr).exists() {
                       if Path::new(&self.hddir).exists() {
                           inputstr = self.hddir.clone();
                       } else {
                           inputstr = "/".to_string();
                       }
                   }
                   let newfilex = FileDialog::new()
                              .set_directory(inputstr)
                              .pick_file();
                   if newfilex == None {
                       self.msg_value =  "error getting db file 2 -- possible cancel key hit".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       newdb = newfilex.as_ref().expect("REASON").display().to_string();
                   }
               }
               if bok {
                   let conn = Connection::open(&newdb).unwrap();
                   if let Err(e) = connectdb(&conn) {
                       self.msg_value = format!("data base error: {} for {}", e, newdb);
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   } else {
                       self.dbconn = conn;
                       self.dbfile = newdb;
                       self.msg_value = format!("db file {} has be set", self.dbfile);
                       self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   }
               }
               Task::none()
            }
            Message::SeqfilePressed => {
               let mut bok: bool = true;
               let mut newseq: String = " ".to_string();
               if Path::new(&self.seqfile.clone()).exists() {
                   let newfile = FileDialog::new()
                              .set_file_name(self.seqfile.clone())
                              .pick_file();
                   if newfile == None {
                       self.msg_value =  "error getting seq file -- possible cancel key hit".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       newseq = newfile.as_ref().expect("REASON").display().to_string();
                   }
               } else {
                   let mut inputstr: String = self.targetdir.clone();
                   if !Path::new(&inputstr).exists() {
                       if Path::new(&self.hddir).exists() {
                           inputstr = self.hddir.clone();
                       } else {
                           inputstr = "/".to_string();
                       }
                   }
                   let newfilex = FileDialog::new()
                              .set_directory(inputstr)
                              .pick_file();
                   if newfilex == None {
                       self.msg_value =  "error getting seq file 2 -- possible cancel key hit".to_string();
                       self.mess_color = Color::from([1.0, 0.0, 0.0]);
                       bok = false;
                   } else {
                       newseq = newfilex.as_ref().expect("REASON").display().to_string();
                   }
               }
               if bok {
                   self.seqfile = newseq.to_string();
                   self.msg_value = format!("seq file {} has be set", self.seqfile);
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               }
               Task::none()
            }
            Message::StartlineChg(value) => { self.startl = value; Task::none() }
            Message::EndlineChg(value) => { self.endl = value; Task::none() }
            Message::RefnameChanged(value) => { self.refname = value; Task::none() }
            Message::RemovePrefixDir(picked) => {self.rpd_bool = picked; Task::none()}
            Message::BkupDataBaseEval(picked) => {self.bde_bool = picked; Task::none()}
            Message::TargetnameChanged(value) => { self.targetname = value; Task::none() }
            Message::TargetdirPressed => {
               let mut inputstr: String = self.targetdir.clone();
               if !Path::new(&inputstr).exists() {
                   if Path::new(&self.hddir).exists() {
                       inputstr = self.hddir.clone();
                   }
               }
               let (errcode, errstr, newinput) = inputpress(inputstr);
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.targetdir = newinput.to_string();
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::ExecPressed => {
               let (errcode, errstr) = execpress(&self.dbconn, self.hddir.clone(), self.seqfile.clone(), self.startl.clone(), self.endl.clone(), self.targetdir.clone(), self.refname.clone(), self.targetname.clone(), self.bde_bool.clone());
               self.msg_value = errstr.to_string();
               if errcode == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
                   Task::perform(Execx::execit(self.seqfile.clone(), self.hddir.clone(), self.startl.clone(), self.endl.clone(), self.rpd_bool.clone(), self.targetdir.clone(), self.refname.clone(),
                                 self.targetname.clone(), self.bde_bool.clone(), self.dbfile.clone(), self.tx_send.clone()), Message::ExecxFound)

               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
                   Task::none()
               }
            }
            Message::ExecxFound(Ok(exx)) => {
               self.msg_value = exx.errval.clone();
               if exx.errcd == 0 {
                   self.mess_color = Color::from([0.0, 1.0, 0.0]);
               } else {
                   self.mess_color = Color::from([1.0, 0.0, 0.0]);
               }
               Task::none()
            }
            Message::ExecxFound(Err(_error)) => {
               self.msg_value = "error in copyx copyit routine".to_string();
               self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }
            Message::ProgressPressed => {
                   self.do_progress = true;
                   Task::perform(Progstart::pstart(), Message::ProgRtn)
            }
            Message::ProgRtn(Ok(_prx)) => {
              if self.do_progress {
                let mut inputval  = " ".to_string();
                let mut bgotmesg = false;
                let mut b100 = false;
                while let Ok(Some(input)) = self.rx_receive.try_next() {
                   inputval = input;
                   bgotmesg = true;
                }
                if bgotmesg {
                    let progvec: Vec<&str> = inputval[0..].split("|").collect();
                    let lenpg1 = progvec.len();
                    if lenpg1 == 4 {
                        let prog1 = progvec[0].to_string();
                        if prog1 == "Progress" {
                            let num_flt: f64 = progvec[1].parse().unwrap_or(-9999.0);
                            if num_flt < 0.0 {
                                println!("progress numeric not numeric: {}", inputval);
                            } else {
                                let dem_flt: f64 = progvec[2].parse().unwrap_or(-9999.0);
                                if dem_flt < 0.0 {
                                    println!("progress numeric not numeric: {}", inputval);
                                } else {
                                    self.progval = 100.0 * (num_flt / dem_flt);
                                    if dem_flt == num_flt {
                                        b100 = true;
                                    } else {
                                        self.msg_value = format!("md5sum progress: {:.3}gb of {:.3}gb {}", (num_flt/1000000000.0), (dem_flt/1000000000.0), progvec[3]);
                                        self.mess_color = Color::from([0.5, 0.5, 1.0]);
                                    }
                                }
                            }
                        } else {
                            println!("message not progress: {}", inputval);
                        }
                    } else {
                        println!("message not progress: {}", inputval);
                    }
                } 
                if b100 {
                    Task::none()   
                } else {         
                    Task::perform(Progstart::pstart(), Message::ProgRtn)
                }
              } else {
                Task::none()
              }
            }
            Message::ProgRtn(Err(_error)) => {
                self.msg_value = "error in Progstart::pstart routine".to_string();
                self.mess_color = Color::from([1.0, 0.0, 0.0]);
               Task::none()
            }

        }
    }

    fn view(&self) -> Element<Message> {
            let mut topshow = Column::new().spacing(5);
            topshow = topshow.push(container(row![text("Message:").size(20),
                 text(&self.msg_value).size(30).color(*&self.mess_color),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));
            topshow = topshow.push(container(row![button("Hard drive directory Button").on_press(Message::HddirPressed),
                 text(&self.hddir).size(20).width(Length::Fill),
                 button("Make Seq File").on_press(Message::MakeseqPressed),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));
            topshow = topshow.push(container(row![text("Start Line: "),
                          text_input("No input....", &self.startl)
                                .on_input(Message::StartlineChg).padding(10),
                          text("End Line: "),
                          text_input("No input....", &self.endl)
                                .on_input(Message::EndlineChg).padding(10),
                          button("get seq file").on_press(Message::SeqfilePressed),
                          text(&self.seqfile),
                          button("test size").on_press(Message::TestsizePressed),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
                ));
            topshow = topshow.push(container(row![checkbox("For Backup: remove prefix directory",
                                     self.rpd_bool).on_toggle(Message::RemovePrefixDir,).width(Length::Fill),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));
            topshow = topshow.push(container(row![text("List Reference name: "),
                 text_input("No input....", &self.refname)
                            .on_input(Message::RefnameChanged).padding(10).size(20),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));
            topshow = topshow.push(container(row![button("Target directory Button").on_press(Message::TargetdirPressed),
                 text(&self.targetdir).size(20).width(1000)
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));

            topshow = topshow.push(container(row![text("Target file name: "),
                 text_input(".hdlist", &self.targetname)
                            .on_input(Message::TargetnameChanged).padding(10).size(20),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
            ));

            if self.bde_bool {
                topshow = topshow.push(container(row![checkbox("For Bkup DataBase Eval",
                      self.bde_bool).on_toggle(Message::BkupDataBaseEval,).width(Length::Fill),
                      button("DataBase File Button").on_press(Message::DBfilePressed),
                      text(&self.dbfile), 
                      horizontal_space(),
                      button("Exec Button").on_press(Message::ExecPressed),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
                ));
            } else {
                topshow = topshow.push(container(row![checkbox("For Bkup DataBase Eval",
                      self.bde_bool).on_toggle(Message::BkupDataBaseEval,).width(Length::Fill),
                      horizontal_space(),
                      button("Exec Button").on_press(Message::ExecPressed),
                       ].align_y(Alignment::Center).spacing(10).padding(10),
                ));
            }
            topshow = topshow.push(container(row![button("Start Progress Button").on_press(Message::ProgressPressed),
                 progress_bar(0.0..=100.0,self.progval as f32),
                 text(format!("{:.2}%", &self.progval)).size(30),
            ].align_y(Alignment::Center).spacing(5).padding(10),
            ));


           column![
              topshow,
           ]
            .padding(1)
            .into()

    }

}

#[derive(Debug, Clone)]
struct Execx {
    errcd: u32,
    errval: String,
}

impl Execx {

    async fn execit(seqfile: String, hddir: String, startl: String, endl: String, rpd_bool: bool, targetdir: String, refname: String,  targetname: String,
                      bde_bool: bool, dbfile: String, tx_send: mpsc::UnboundedSender<String>,) -> Result<Execx, Error> {
     let mut errstring  = "Complete harddrive listing".to_string();
     let mut bok: bool = true;
     let mut errcode: u32 = 0;
     let mut linenum: u64 = 0;
     let mut szaccum: u64 = 0;
     let mut numrows: u64 = 0;
//     let mut xrows: u64 = 1000;
     let mut totalsz: u64 = 0;
     let mut totalerr: u64 = 0;
     let mut outseq: u32 = 1;
     let conn = Connection::open(dbfile.clone()).unwrap();
     let start_time = timeInstant::now();
     let mut more1out: String = format!("{}/more1{:02}.excout", targetdir, outseq);
     let mut just1out: String = format!("{}/just1{:02}.neout", targetdir, outseq);
     let mut diffdateout: String = format!("{}/diffdate{:02}.excout", targetdir, outseq);
     let mut nobkupout: String = format!("{}/nobkup{:02}.neout", targetdir, outseq);
     let mut errout: String = format!("{}/generrors{:02}.errout", targetdir, outseq);
     if bde_bool {
         loop {
               if !Path::new(&errout).exists() && !Path::new(&more1out).exists() && !Path::new(&just1out).exists()
                  && !Path::new(&diffdateout).exists() && !Path::new(&nobkupout).exists() {
                   break;
               } else {
                   outseq = outseq + 1;
                   more1out = format!("{}/more1{:02}.excout", targetdir, outseq);
                   just1out = format!("{}/just1{:02}.neout", targetdir, outseq);
                   diffdateout = format!("{}/diffdate{:02}.excout", targetdir, outseq);
                   nobkupout = format!("{}/nobkup{:02}.neout", targetdir, outseq);
                   errout = format!("{}/generrors{:02}.errout", targetdir, outseq);
               }
         }          
     }
     let mut diffdatefile = File::create(diffdateout).unwrap();
     let mut nobkupfile = File::create(nobkupout).unwrap();
     let mut more1file = File::create(more1out).unwrap();
     let mut just1file = File::create(just1out).unwrap();
     let mut errfile = File::create(errout).unwrap();
     let targetfullname: String = format!("{}/{}", targetdir, targetname);
     let mut targetfile = File::create(targetfullname).unwrap();
     let file = File::open(seqfile.clone()).unwrap();
     let mut reader = BufReader::new(file);
     let mut linehd = String::new();
     let from_int: i64 = startl.parse().unwrap_or(-99);
     let to_int: i64 = endl.parse().unwrap_or(-99);
     loop {
         match reader.read_line(&mut linehd) {
             Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                linenum = linenum + 1;
                let veclinea: Vec<&str> = linehd.split("|").collect();
                if veclinea.len() != 5 {
                    errstring  = "seq file not valid 1".to_string();
                    errcode = 1;
                    bok = false;
                    break;
                } else {
                    let inline: i64 = veclinea[0].parse().unwrap_or(-99);
                    if inline == -99 {
                        errstring  =  "inline is not an integer".to_string();
                        errcode = 2;
                        bok = false;
                        break;
                    } else if inline < 1 {
                        errstring =  "inline is less than 1".to_string();
                        errcode = 3;
                        bok = false;
                        break;
                    }
                    if inline >= from_int {
                        if inline > to_int {
                            break;
                        }
                        let filesz: i64 = veclinea[2].parse().unwrap_or(-99);
                        if filesz == -99 {
                            errstring =  format!("in file size {} is not an integer: {}", veclinea[2], linehd);
                            errcode = 4;
                            bok = false;
                            break;
                        } else if filesz < 0 {
                            errstring = "file size is less than 0".to_string();
                            errcode = 5;
                            bok = false;
                            break;
                        }
                        numrows = numrows + 1;
                        totalsz = totalsz + filesz as u64;
                    }
                }
                linehd.clear();
             }
             Err(err) => {
                errstring = format!("error of {} reading {}", err, seqfile);
                errcode = 6;
                bok = false;
                break;
             }
         }
     }
     if bok {
         if numrows < 1 {
             errstring  = "no files on disk".to_string();
             errcode = 7;
             bok = false;
         }
     }
     if bok {
        let diffy = start_time.elapsed();
        let minsy: f64 = diffy.as_secs() as f64/60 as f64;
        let dateyy = Local::now();
        let msgx = format!("Progress|{}|{}| elapsed time {:.1} mins at {} for {} files", szaccum, totalsz, minsy, dateyy.format("%H:%M:%S"), numrows);
        tx_send.unbounded_send(msgx).unwrap();
        let mut linehdx = String::new();
        let filex = File::open(seqfile.clone()).unwrap();
        let mut readerx = BufReader::new(filex);
        linenum = 0;
        loop {
            match readerx.read_line(&mut linehdx) {
               Ok(bytes_read) => {
                  if bytes_read == 0 {
                      break;
                  }
                  linenum = linenum + 1;
                  let inline: i64;
                  let veclinea: Vec<&str> = linehdx.split("|").collect();
                  if veclinea.len() != 5 {
                      errstring  = "seq file not valid 2".to_string();
                      errcode = 8;
//                      bok = false;
                      break;
                  } else {
                      inline = veclinea[0].parse().unwrap_or(-99);
                      if inline == -99 {
                          errstring  =  "inline is not an integer".to_string();
                          errcode = 9;
//                          bok = false;
                          break;
                      } else if inline < 1 {
                          errstring =  "inline is less than 1".to_string();
                          errcode = 10;
//                          bok = false;
                          break;
                      }
                      if inline >= from_int {
                          if inline > to_int {
                              break;
                          }
                          let filesz: i64 = veclinea[2].parse().unwrap_or(-99);
                          if filesz == -99 {
                              errstring =  format!("in file size {} is not an integer: {}", veclinea[2], linehd);
                              errcode = 11;
//                              bok = false;
                              break;
                          } else if filesz < 0 {
                              errstring = "file size is less than 0".to_string();
                              errcode = 12;
//                              bok = false;
                              break;
                          }
                          let fullpath = veclinea[1].to_string();
                          let (errcd1, errmsg1, md5sumv) = findmd5sum(fullpath.clone());
                          if errcd1 == 0 {
                              let lrperpos = fullpath.rfind("/").unwrap();
         		              let file_name = fullpath.get((lrperpos+1)..).unwrap();
                              let mut strtfull = 0;
                              if rpd_bool {
                                  strtfull = hddir.len();
                              }
         		              let file_dir = fullpath.get((strtfull)..(lrperpos)).unwrap();
                              let file_date = veclinea[3].to_string();
                              let filesz: i64 = veclinea[2].parse().unwrap_or(-99);
                              if filesz == -99 {
                                  errstring =  format!("in file size {} is not an integer: {}", veclinea[2], linehd);
                                  errcode = 13;
//                                  bok = false;
                                  break;
                              } else if filesz < 0 {
                                  errstring = "file size is less than 0".to_string();
                                  errcode = 14;
//                                  bok = false;
                                  break;
                              }
                              let stroutput = format!("{}|{}|{}|{}|{}|{}",
                                                      file_name,
                                                      filesz,
                                                      file_date,
                                                      file_dir,
                                                      refname,
                                                      md5sumv);
                              writeln!(&mut targetfile, "{}", stroutput).unwrap();
                              szaccum = szaccum + filesz as u64;
                              if bde_bool {
                                  match conn.prepare("SELECT  rowid, refname, filename, dirname, filesize, filedate, md5sum, locations, notes
                                          FROM blubackup
                                          WHERE filename = :fil AND md5sum = :md5;") {
                                      Err(err) => {
                                          writeln!(&mut errfile, "err {} in sql prepare call for file {} - {}", err, file_name, md5sumv).unwrap();
                                      }
                                      Ok(mut stmt) => {
                                          match stmt.query_map(&[(":fil", &file_name), (":md5", &md5sumv.as_str())], |row| {
                                              Ok(Bkup {
                                                  rowid: row.get(0)?,
                                                  refname: row.get(1)?,
                                                  filename: row.get(2)?,
                                                  dirname: row.get(3)?,
                                                  filesize: row.get(4)?,
                                                  filedate: row.get(5)?,
                                                  md5sum: row.get(6)?,
                                                  locations: row.get(7)?,
                                                  notes: row.get(8)?,
                                              })
                                            })
                                           {
                                              Err(err) => {
                                                writeln!(&mut errfile, "err {} in sql query for file {}", err, file_name).unwrap();
                                              }
                                              Ok(bk_iter) => {
                                                let mut numentries = 0;
                                                let mut numdate = 0;
                                                let mut numsize = 0;
                                                let mut stroutput = format!("{}|{}|{}|{}|{}|{}|", 
                                                        file_name, file_dir, filesz, file_date, refname, md5sumv);
                                                for bk in bk_iter {
                                                     numentries = numentries + 1;
                                                     let bki = bk.unwrap();
                                                     let bksize = format!("{}", bki.filesize);
                                                     let bkdate: String = bki.filedate;
                                                     let bkref: String = bki.refname;
                                                     stroutput = format!("{}|{}|{}|{}|", 
                                                              stroutput, bkref, bksize, bkdate);
                                                     let filelenst = format!("{}", filesz);
                                                     if bksize == filelenst {
                                                         numsize = numsize + 1;
                                                         if bkdate == file_date {
                                                             numdate = numdate + 1;
                                                         }
                                                     }
                                                }
                                                if numentries < 1 {
                                                    writeln!(&mut nobkupfile, "{}", stroutput).unwrap();
                                                } else {
                                                    if numsize < 1 {
                                                        stroutput = format!("{} --NO MATCHING SIZE", stroutput);
                                                        writeln!(&mut errfile, "{}", stroutput).unwrap();
                                                        writeln!(&mut nobkupfile, "{}", stroutput).unwrap();
                                                    } else {
                                                        if numdate < 1 {
                                                            stroutput = format!("{} -- NO MATCHING DATE", stroutput);
                                                            writeln!(&mut diffdatefile, "{}", stroutput).unwrap();
                                                        }
                                                        if numentries == 1 {
                                                            writeln!(&mut just1file, "{}", stroutput).unwrap();
                                                        } else {
                                                            stroutput = format!("{} -- {} entries", stroutput, numentries);
                                                            writeln!(&mut more1file, "{}", stroutput).unwrap();
                                                        }            
                                                    }
                                                }
                                              }
                                           }
                                      } // Ok
                                  } // match conn
                              }
                          } else {
                              totalerr = totalerr + 1;
                              let strerr = format!("ERROR #{}: {}", totalerr, errmsg1);
                              writeln!(&mut errfile, "{}", strerr).unwrap();
                              totalerr = totalerr + 1;
                          }
                      }
                  }
                  let diffx = start_time.elapsed();
                  let minsx: f64 = diffx.as_secs() as f64/60 as f64;
                  let datexx = Local::now();
                  let msgx = format!("Progress|{}|{}| elapsed time {:.1} mins at {} {} of {}", szaccum, totalsz, minsx, datexx.format("%H:%M:%S"), inline, to_int);
                  tx_send.unbounded_send(msgx).unwrap();
                  linehdx.clear();
               }
               Err(err) => {
                  errstring = format!("error of {} reading {}", err, seqfile);
                  errcode = 15;
//                  bok = false;
                  break;
               }
            }
        }
     }
     let msgx = format!("Progress|{}|{}| end of md5sum process", numrows, numrows);
     tx_send.unbounded_send(msgx).unwrap();
     Ok(Execx {
            errcd: errcode,
            errval: errstring,
        })
    }
}
#[derive(Debug, Clone)]
pub enum Error {
}

// loop thru by sleeping for 5 seconds
#[derive(Debug, Clone)]
pub struct Progstart {
}

impl Progstart {

    pub async fn pstart() -> Result<Progstart, Error> {
     sleep(timeDuration::from_secs(5));
     Ok(Progstart {
        })
    }
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

