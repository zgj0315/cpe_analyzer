use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

use rusqlite::Connection;
use xml::reader::XmlEvent;
use xml::EventReader;
const CPE_DICT: &str = "./data/official-cpe-dictionary_v2.3.xml.zip";
pub async fn download_cpe() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./data");
    if !path.exists() {
        fs::create_dir(path).unwrap();
    }
    let path = Path::new(CPE_DICT);
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
    log::info!("{:?} downloaded successfully", path.file_name().unwrap());
    Ok(())
}

#[derive(Debug)]
struct GroupByOne {
    group_name: String,
    count: u32,
}

#[derive(Debug)]
struct GroupByTwo {
    group_name_a: String,
    group_name_b: String,
    count: u32,
}
#[derive(Debug)]
struct GroupByThree {
    group_name_a: String,
    group_name_b: String,
    group_name_c: String,
    count: u32,
}
pub async fn put_cpe_to_db() -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = File::open(CPE_DICT).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let file = archive.by_name("official-cpe-dictionary_v2.3.xml").unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);
    let conn = Connection::open("./data/cpe.db").unwrap();
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

pub async fn cpe_stat() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/cpe.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT part, count(*) from tbl_cpe GROUP BY part ORDER BY count(*) DESC")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(GroupByOne {
                group_name: row.get(0).unwrap(),
                count: row.get(1).unwrap(),
            })
        })
        .unwrap();
    let mut output = File::create("./data/group_by_part.csv").expect("create failed");
    for row in rows {
        let group_by_one = row.unwrap();
        let line = format!("{},{}\n", group_by_one.group_name, group_by_one.count);
        output.write_all(line.as_bytes()).expect("write failed");
    }

    let mut stmt = conn
        .prepare("SELECT part, vendor, count(*) from tbl_cpe GROUP BY part, vendor ORDER BY count(*) DESC")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(GroupByTwo {
                group_name_a: row.get(0).unwrap(),
                group_name_b: row.get(1).unwrap(),
                count: row.get(2).unwrap(),
            })
        })
        .unwrap();
    let mut output = File::create("./data/group_by_part_vendor.csv").expect("create failed");
    for row in rows {
        let group_by_two = row.unwrap();
        let line = format!(
            "{},{},{}\n",
            group_by_two.group_name_a, group_by_two.group_name_b, group_by_two.count
        );
        output.write_all(line.as_bytes()).expect("write failed");
    }

    let mut stmt = conn
        .prepare(
            "SELECT part, vendor, product, count(*) from tbl_cpe GROUP BY part, vendor, product ORDER BY count(*) DESC",
        )
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(GroupByThree {
                group_name_a: row.get(0).unwrap(),
                group_name_b: row.get(1).unwrap(),
                group_name_c: row.get(2).unwrap(),
                count: row.get(3).unwrap(),
            })
        })
        .unwrap();
    let mut output =
        File::create("./data/group_by_part_vendor_product.csv").expect("create failed");
    for row in rows {
        let group_by_three = row.unwrap();
        let line = format!(
            "{},{},{},{}\n",
            group_by_three.group_name_a,
            group_by_three.group_name_b,
            group_by_three.group_name_c,
            group_by_three.count
        );
        output.write_all(line.as_bytes()).expect("write failed");
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

    #[tokio::test]
    async fn test_cpe_stat() {
        let future_cpe_stat = cpe_stat();
        let _ = tokio::join!(future_cpe_stat);
    }
}
