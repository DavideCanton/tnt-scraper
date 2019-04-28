extern crate tnt_scraper_lib;

use std::{io, io::prelude::*};
use tnt_scraper_lib::{RequestData, RequestManager, TntCategory, TntResult};

fn print_values(results: &TntResult, page: u8) {
    println!("Pagina corrente: [{}]/[{}]", page, results.npages);
    println!("Trovati {} risultati.", results.entries.len());

    for entry in &results.entries {
        println!("{}", entry);
    }
}

fn get_args() -> clap::ArgMatches<'static> {
    clap::App::new("tnt-scraper")
        .version("1.0")
        .author("Davide C. <davide.canton5@gmail.com>")
        .about("TNTVillage Release list scraper")
        .arg(
            clap::Arg::with_name("category")
                .short("c")
                .takes_value(true)
                .help("category"),
        )
        .arg(
            clap::Arg::with_name("pages")
                .short("p")
                .takes_value(true)
                .default_value("1")
                .help("page"),
        )
        .arg(clap::Arg::with_name("QUERY").help("Query").index(1))
        .get_matches()
}

fn read_int<F: Fn(u8) -> bool>(s: &str, f: F) -> Option<u8> {
    match s.trim().parse::<u8>() {
        Ok(value) if f(value) => Some(value),
        _ => None,
    }
}

fn loop_read_int<F: Fn(u8) -> bool>(prompt: &str, pred: F, cont: bool, err: &str) -> Option<u8> {
    loop {
        let buf = read_string(prompt).expect("Errore di lettura");

        if let Some(v) = read_int(&buf, &pred) {
            return Some(v);
        } else {
            println!("Errore: {}", err);
            if !cont {
                break;
            }
        }
    }

    None
}

fn ask_category() -> u8 {
    for val in TntCategory::values() {
        println!("[{}] {}", val.value(), val.to_string());
    }

    loop_read_int(
        "Seleziona un valore>",
        |v| TntCategory::is_valid_value(v),
        true,
        "Categoria non valida!",
    )
    .unwrap()
}

fn ask_page(max_pages: u8) -> u8 {
    loop_read_int(
        "Pagina da richiedere>",
        |v| v <= max_pages,
        true,
        "Numero di pagina non valido!",
    )
    .unwrap()
}

fn read_string(prompt: &str) -> io::Result<String> {
    let mut buf = String::new();

    print!("{}", prompt);

    let _ = io::stdout().flush();
    io::stdin().read_line(&mut buf)?;

    return Ok(buf);
}

fn ask_query() -> String {
    read_string("Input>").expect("Errore di lettura")
}

fn ask_index() -> Option<u8> {
    loop_read_int(
        "File da scaricare>",
        |_| true,
        false,
        "Indice file non valido!",
    )
}

fn want_download() -> bool {
    let val = read_string("Vuoi scaricare un file (S/N)>")
        .expect("Errore di lettura")
        .trim()
        .to_lowercase();

    val == "s"
}

fn download_loop(mgr: &RequestManager, v: &TntResult) {
    while want_download() {
        if let Some(index) = ask_index() {
            match mgr.download_file(&v.entries[index as usize]) {
                Ok(_) => println!("Download completato!"),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
}

fn start_scrape(mgr: &mut RequestManager, query: &str, category: u8, page: u8) -> Option<u8> {
    let data = RequestData::new(query, category, page);

    let results = mgr.extract_results(&data);

    match results {
        Ok(v) => {
            print_values(&v, page);
            if v.entries.len() > 0 {
                download_loop(&mgr, &v);
                Some(ask_page(v.npages))
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("Errore durante lo scraping: {:?}", e);
            None
        }
    }
}

fn main() {
    let args = get_args();

    let query = args
        .value_of("QUERY")
        .map(|v| v.to_owned())
        .or_else(|| Some(ask_query()))
        .unwrap();

    let category = args
        .value_of("category")
        .map(|v| v.to_owned())
        .or_else(|| Some(ask_category().to_string()))
        .unwrap()
        .parse::<u8>()
        .unwrap();

    let page = args.value_of("pages").unwrap().parse::<u8>().unwrap();
    let mut manager = RequestManager::new();

    let mut current_page = page;

    while let Some(c) = start_scrape(&mut manager, &query, category, current_page) {
        current_page = c;
    }
}
