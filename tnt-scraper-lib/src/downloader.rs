use crate::{RequestData,Result,Error};
use reqwest::header::{ContentDisposition, ContentLength, DispositionParam, Header, Headers, Raw};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

pub fn request_content(url: &str, data: &RequestData) -> Result<String> {
    let client = reqwest::Client::new();

    let cat = data.cat.value().to_string();
    let page = data.page.to_string();

    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("srcrel", &data.srcrel);
    params.insert("cat", &cat);
    params.insert("page", &page);

    let req = client
        .post(url)
        .form(&params)
        .build()
        .map_err(|e| Error::DownloadError(e))?;

    client
        .execute(req)
        .and_then(|ref mut r| r.text())
        .map_err(|e| Error::DownloadError(e))
}

fn fix_headers(headers: &mut Headers) -> Result<()> {
    let (cl, cd) = {
        let header = headers
            .iter()
            .find(|h| h.name() == ContentDisposition::header_name())
            .ok_or(Error::GenericError("Header CD not found".to_owned()))?;

        let raw = String::from_utf8_lossy(
            header
                .raw()
                .one()
                .ok_or(Error::GenericError("raw not parsed".to_owned()))?,
        );

        let (cd_s, cl_s) = raw.split_at(raw.find("Content-Length:").unwrap());

        let cd_s = cd_s.trim().trim_matches(';');
        let cl_s = cl_s.trim().trim_matches('"').split_at(15).1;

        let cl = ContentLength::parse_header(&Raw::from(cl_s)).map_err(|e| Error::HyperError(e))?;

        let cd =
            ContentDisposition::parse_header(&Raw::from(cd_s)).map_err(|e| Error::HyperError(e))?;

        (cl, cd)
    };

    headers.set(cl);
    headers.set(cd);

    Ok(())
}

fn get_filename_from_header(disposition: &ContentDisposition) -> Result<String> {
    let param = disposition.parameters.iter().find(|&v| match v {
        &DispositionParam::Filename(_, _, _) => true,
        _ => false,
    });

    match param {
        Some(&DispositionParam::Filename(_, _, ref bytes)) => {
            String::from_utf8(bytes.to_vec()).map_err(|e| Error::HyperError(From::from(e)))
        }
        _ => Err(Error::GenericError("Filename not found".to_owned())),
    }
}

fn read_filename(res: &mut reqwest::Response) -> Result<String> {
    let headers = res.headers();

    match headers.get::<ContentDisposition>() {
        Some(v) => get_filename_from_header(v),
        None => {
            let mut headers_cloned = headers.clone();
            fix_headers(&mut headers_cloned)?;

            match headers_cloned.get::<ContentDisposition>() {
                Some(v) => get_filename_from_header(v),
                None => Err(Error::GenericError("Header not found".to_owned())),
            }
        }
    }
}

fn save_file(res: &mut reqwest::Response, path: &PathBuf) -> Result<()> {
    let filename = read_filename(res)?;
    let mut path = path.clone();
    path.push(filename);
    path.set_extension("torrent");

    let mut f = File::create(&path).map_err(|e| Error::IOError(e))?;
    let written = res.copy_to(&mut f).map_err(|e| Error::DownloadError(e))?;
    println!("File scaricato in {:?}, scritti {} bytes.", &path, written);

    Ok(())
}

pub fn download_file(path: &PathBuf, url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let req = client
        .get(url)
        .build()
        .map_err(|e| Error::DownloadError(e))?;

    match client.execute(req) {
        Ok(ref mut r) => save_file(r, path),
        Err(e) => Err(Error::DownloadError(e)),
    }
}
