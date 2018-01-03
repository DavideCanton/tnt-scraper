extern crate reqwest;
extern crate hyper;

mod downloader;
mod scraper;
mod selector_cache;

use std::env::home_dir;
use std::fmt::{Display, Formatter, Result as FResult};
use std::collections::HashSet;
use std::io;

#[derive(Debug)]
pub enum Error {
    DownloadError(reqwest::Error),
    HyperError(hyper::Error),
    ScrapeError(String),
    ParseError(String),
    GenericError(String),
    IOError(io::Error)
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum TntCategory {
    TutteLeCategorie = 0,
    Film = 4,
    Musica = 2,
    FilmTVProgrammi = 1,
    Documentari = 14,
    StudentsReleases = 13,
    EBooks = 3,
    Linux = 6,
    Anime = 7,
    Cartoni = 8,
    Macintosh = 9,
    WindowsSoftware = 10,
    PcGame = 11,
    Playstation = 12,
    VideoMusicali = 21,
    Sport = 22,
    Teatro = 23,
    Wrestling = 24,
    Varie = 25,
    Xbox = 26,
    ImmaginiSfondi = 27,
    AltriGiochi = 28,
    SerieTV = 29,
    Fumetteria = 30,
    Trash = 31,
    Nintendo = 32,
    ABook = 34,
    Podcast = 35,
    Edicola = 36,
    Mobile = 37,
}

impl TntCategory {
    pub fn value(&self) -> u8 {
        *self as u8
    }

    pub fn values() -> Vec<TntCategory> {
        let mut values: HashSet<TntCategory> = HashSet::new();

        for i in 0..38 {
            values.insert(i.into());
        }

        let mut ret = values.into_iter().collect::<Vec<_>>();
        ret.sort_unstable();
        ret
    }

    pub fn is_valid_value(c: u8) -> bool {
        let invalid_values = vec![5, 15, 16, 17, 18, 19, 20, 33];
        !invalid_values.contains(&c) && c <= 37
    }
}

impl From<u8> for TntCategory {
    fn from(c: u8) -> Self {
        if !TntCategory::is_valid_value(c) {
            TntCategory::TutteLeCategorie
        } else {
            unsafe { ::std::mem::transmute(c) }
        }
    }
}

impl Display for TntCategory {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct TntEntry {
    id: u32,
    download_link: String,
    leechers: u32,
    seeders: u32,
    c: u32,
    url: String,
    desc: String,
}

impl Display for TntEntry {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "[{}] {} ({})", self.id, self.desc, self.download_link)
    }
}

pub struct TntResult {
    pub entries: Vec<TntEntry>,
    pub npages: u8,
}

impl TntResult {
    pub fn new(entries: Vec<TntEntry>, npages: u8) -> Self {
        TntResult {
            entries,
            npages,
        }
    }
}

const URL: &'static str = "http://www.tntvillage.scambioetico.org/src/releaselist.php";

pub type Result<T> = std::result::Result<T, Error>;
pub type ScrapeResult = Result<TntResult>;

#[derive(Debug)]
pub struct RequestData {
    page: u8,
    cat: TntCategory,
    srcrel: String,
}

impl RequestData {
    pub fn new(query: &str, cat: TntCategory, page: u8) -> Self {
        RequestData {
            cat,
            page,
            srcrel: query.to_owned(),
        }
    }
}


pub fn extract_results(query_data: &RequestData) -> ScrapeResult {
    let html_result = downloader::request_content(URL, &query_data)?;
    let entries = scraper::scrape(&html_result)?;

    Ok(entries)
}

pub fn download_file(entry: &TntEntry) -> Result<()> {
    let mut path = home_dir().unwrap();
    path.push("Downloads");
    downloader::download_file(&path, &entry.download_link)
}