use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use rusqlite::Connection;
use xml::reader::XmlEvent;
use xml::EventReader;

pub static SQLITE_DB: &str = "./data/cpe.db";
static CPE_DICT_LOCAL_PATH: &str = "./data/official-cpe-dictionary_v2.3.xml.zip";
static CPE_DICT_URL: &str =
    "https://nvd.nist.gov/feeds/xml/cpe/dictionary/official-cpe-dictionary_v2.3.xml.zip";
static CPE_DICT_XML_FILE_NAME: &str = "official-cpe-dictionary_v2.3.xml";

pub async fn download_cpe() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./data");
    if !path.exists() {
        fs::create_dir(path).unwrap();
    }
    let path = Path::new(CPE_DICT_LOCAL_PATH);
    let mut file = match File::create(&path) {
        Err(e) => panic!("Error creating {}", e),
        Ok(file) => file,
    };
    let rsp = reqwest::get(CPE_DICT_URL).await?;
    let rsp_bytes = rsp.bytes().await?;
    let _ = file.write_all(&rsp_bytes);
    log::info!("{:?} downloaded successfully", path.file_name().unwrap());
    Ok(())
}

pub async fn put_cpe_to_db() -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = File::open(CPE_DICT_LOCAL_PATH).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let file = archive.by_name(CPE_DICT_XML_FILE_NAME).unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let conn = Connection::open(SQLITE_DB).unwrap();
    conn.execute("DROP TABLE IF EXISTS tbl_cpe", ()).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tbl_cpe (
        part  TEXT NOT NULL,
        vendor  TEXT NOT NULL,
        product  TEXT NOT NULL,
        version  TEXT NOT NULL,
        update_  TEXT NOT NULL,
        edition  TEXT NOT NULL,
        language  TEXT NOT NULL,
        sw_edition  TEXT NOT NULL,
        target_sw  TEXT NOT NULL,
        target_hw  TEXT NOT NULL,
        other  TEXT NOT NULL
    )",
        (),
    )
    .unwrap();
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
                        let cpe_vec: Vec<&str> = cpe23uri.split(":").collect();
                        let sql = "INSERT INTO tbl_cpe (
                            part, vendor, product, version, update_, edition, 
                            language, sw_edition, target_hw, target_sw, other) VALUES (
                                ?1, ?2, ?3, ?4, ?5, ?6,
                                ?7, ?8, ?9, ?10, ?11
                            )";
                        conn.execute(
                            sql,
                            (
                                cpe_vec[2],
                                cpe_vec[3],
                                cpe_vec[4],
                                cpe_vec[5],
                                cpe_vec[6],
                                cpe_vec[7],
                                cpe_vec[8],
                                cpe_vec[9],
                                cpe_vec[10],
                                cpe_vec[11],
                                cpe_vec[12],
                            ),
                        )
                        .unwrap();
                    }
                }
            }

            Err(e) => {
                log::error!("Error: {}", e);
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
