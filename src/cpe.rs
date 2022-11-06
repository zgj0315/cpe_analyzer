use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

use rusqlite::Connection;
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

#[derive(Debug)]
struct Cpe23Uri {
    part: String,
    vendor: String,
    product: String,
    version: String,
    update: String,
    edition: String,
    language: String,
    sw_edition: String,
    target_sw: String,
    target_hw: String,
    other: String,
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
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // let mut stmt = conn.prepare("SELECT * from tbl_cpe").unwrap();
    // let rows = stmt
    //     .query_map([], |row| {
    //         Ok(Cpe23Uri {
    //             part: row.get(0).unwrap(),
    //             vendor: row.get(1).unwrap(),
    //             product: row.get(2).unwrap(),
    //             version: row.get(3).unwrap(),
    //             update: row.get(4).unwrap(),
    //             edition: row.get(5).unwrap(),
    //             language: row.get(6).unwrap(),
    //             sw_edition: row.get(7).unwrap(),
    //             target_sw: row.get(8).unwrap(),
    //             target_hw: row.get(9).unwrap(),
    //             other: row.get(10).unwrap(),
    //         })
    //     })
    //     .unwrap();
    // for row in rows {
    //     println!("cpe23uri: {:?}", row);
    // }

    let mut stmt = conn
        .prepare("SELECT part, count(*) from tbl_cpe GROUP BY part")
        .unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(GroupByOne {
                group_name: row.get(0).unwrap(),
                count: row.get(1).unwrap(),
            })
        })
        .unwrap();
    for row in rows {
        println!("group by part: {:?}", row);
    }

    let mut stmt = conn
        .prepare("SELECT part, vendor count(*) from tbl_cpe GROUP BY part, vendor")
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
    for row in rows {
        println!("group by part, vendor: {:?}", row);
    }

    let mut stmt = conn
        .prepare(
            "SELECT part, vendor, product, count(*) from tbl_cpe GROUP BY part, vendor, product",
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
    for row in rows {
        println!("group by part, vendor: {:?}", row);
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
