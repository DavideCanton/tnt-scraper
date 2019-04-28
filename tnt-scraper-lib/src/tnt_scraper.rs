use crate::{selector_cache::SelectorCache, Error, Result, ScrapeResult, TntEntry, TntResult};
use scraper::{ElementRef, Html};
/*
<tr>
            <td>
                <a href='http://forum.tntvillage.scambioetico.org/index.php?act=Attach&type=post&id=15345599'>
                    <img src='images/icon_bt_16x16.png' alt='Download torrent' />
                </a>
            </td>
            <td width='20px'>
                <a href='magnet:?xt=urn:btih:639569FDE2237F885F1BFA0C8DCE40B4A34DAF1C&dn=Kill%2BBill%2B-%2BVolume%2B2%2B%25282004%2529&tr=http%3A%2F%2Ftracker.tntvillage.scambioetico.org%3A2710%2Fannounce&tr=udp%3A%2F%2Ftracker.tntvillage.scambioetico.org%3A2710%2Fannounce&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2FIPv6.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.tiny-vps.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.mg64.net%3A2710%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A80%2Fannounce'>
                    <img src='images/icon_magnet_16x16.png' alt='Magnet link' />
                </a>
            </td>
            <td width='20px'>
                <a href='http://forum.tntvillage.scambioetico.org/index.php?act=allreleases&st=0&filter=&sb=1&sd=0&cat=4' target='_blank'>
                    <img src='http://forum.tntvillage.scambioetico.org/style_images/mkportal-636/icon4.gif' height='16px'/>
                </a>
            </td>
            <td width='10px'>0</td>
            <td width='10px'>8</td>
            <td width='10px'>1664</td>
            <td>
                <a href='http://forum.tntvillage.scambioetico.org/index.php?showtopic=524560' target='_blank'>Kill Bill - Volume 2 (2004)</a>&nbsp;[BDrip 720p - H265 - Ita Eng Ac3 5.1 - Sub NUIta Eng] Action - PirateMKV [CURA] Arti Marziali
            </td>
        </tr>
*/

fn extract_desc(elements: &[ElementRef], _cache: &SelectorCache) -> Result<String> {
    let desc_td: &ElementRef = elements
        .get(6)
        .ok_or(Error::ParseError("6 not found".to_owned()))?;

    let res = desc_td.text().collect::<Vec<_>>().join(" ");

    Ok(res)
}

fn extract_torrent(elements: &[ElementRef], cache: &SelectorCache) -> Result<String> {
    let torrent_td: &ElementRef = elements
        .get(0)
        .ok_or(Error::ParseError("0 not found".to_owned()))?;

    let res = torrent_td
        .select(&cache.add_and_get_selector("a"))
        .nth(0)
        .ok_or(Error::ParseError("a not found".to_owned()))?
        .value()
        .attr("href")
        .map_or("".to_owned(), |v| v.to_owned());

    Ok(res)
}

fn build_entry(el: &ElementRef, i: usize, cache: &SelectorCache) -> Result<TntEntry> {
    let td_sel = cache.add_and_get_selector("td");
    let tds = el.select(&td_sel).collect::<Vec<_>>();

    let entry = TntEntry {
        id: i as u32,
        desc: extract_desc(&tds, cache)?,
        c: 1,
        leechers: 1,
        seeders: 1,
        download_link: extract_torrent(&tds, cache)?,
        url: "".to_string(),
    };

    Ok(entry)
}

pub fn scrape(html_result: &str, selector_cache: &mut SelectorCache) -> ScrapeResult {
    let doc = Html::parse_document(html_result);

    let selector = selector_cache.add_and_get_selector("tr");
    let res = doc
        .select(&selector)
        .enumerate()
        .skip(1)
        .filter_map(|(i, el)| build_entry(&el, i, selector_cache).ok())
        .collect();

    let page_selector = selector_cache.add_and_get_selector(".total");
    let npages = doc
        .select(&page_selector)
        .nth(0)
        .and_then(|e| e.value().attr("a"))
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(1);

    Ok(TntResult::new(res, npages))
}
