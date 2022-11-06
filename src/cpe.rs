use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use xml::reader::XmlEvent;
use xml::EventReader;
const CPE_DICT: &str = "./data/official-cpe-dictionary_v2.3.xml.zip";
pub async fn download_cpe() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(CPE_DICT);
    if path.exists() {
        println!(
            "{:?} exists, splitting download_cpe",
            path.file_name().unwrap()
        );
    } else {
        let mut file = match File::create(&path) {
            Err(e) => panic!("Error creating {}", e),
            Ok(file) => file,
        };
        let rsp = reqwest::get(
            "https://nvd.nist.gov/feeds/xml/cpe/dictionary/official-cpe-dictionary_v2.3.xml.zip",
        )
        .await?;
        let rsp_bytes = rsp.bytes().await?;
        let _ = file.write_all(&rsp_bytes);
        println!("{:?} downloaded successfully", path.file_name().unwrap());
    }
    Ok(())
}

pub async fn put_cpe_to_db() -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = File::open(CPE_DICT).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let file = archive.by_name("official-cpe-dictionary_v2.3.xml").unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name,
                attributes,
                namespace: _,
            }) => {
                let local_name = &name.local_name;
                if "cpe23-item" == local_name {
                    for attribute in &attributes {
                        let cpe23uri = &attribute.value;
                        println!("{}", cpe23uri);
                    }
                }
            }

            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_download_cpe() {
        let future_download_cpe = download_cpe();
        let _ = tokio::join!(future_download_cpe);
    }

    #[tokio::test]
    async fn test_put_cpe_to_db() {
        let future_put_cpe_to_db = put_cpe_to_db();
        let _ = tokio::join!(future_put_cpe_to_db);
    }
}
