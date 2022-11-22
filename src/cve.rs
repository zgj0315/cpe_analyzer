use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use rusqlite::Connection;
use serde_json::Value;

const CVE_DICT: &str = "./data/nvdcve-1.1-2022.json.zip";

async fn download_cve() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./data");
    if !path.exists() {
        fs::create_dir(path).unwrap();
    }
    let path = Path::new(CVE_DICT);
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
        let rsp = reqwest::get("https://nvd.nist.gov/feeds/json/cve/1.1/nvdcve-1.1-2022.json.zip")
            .await?;
        let rsp_bytes = rsp.bytes().await?;
        let _ = file.write_all(&rsp_bytes);
        println!("{:?} downloaded successfully", path.file_name().unwrap());
    }
    Ok(())
}

async fn put_cpe_to_db() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/cpe.db").unwrap();
    conn.execute("DROP TABLE IF EXISTS tbl_cpe_from_cve", ())
        .unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tbl_cpe_from_cve (
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
    let zip_file = File::open(CVE_DICT).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let file = archive.by_name("nvdcve-1.1-2022.json").unwrap();
    let json: Value = serde_json::from_reader(file).unwrap();
    match json["CVE_Items"].as_array() {
        Some(cve_items) => {
            for cve_item in cve_items.into_iter() {
                let nodes = &cve_item["configurations"]["nodes"].as_array().unwrap();
                for node in nodes.into_iter() {
                    let cpe_vec = get_cpe_from_node(node);
                    if cpe_vec.len() == 0 {
                        println!("not find cpe, nodes: {:?}", node);
                    }
                    for cpe in cpe_vec {
                        let cpe_vec: Vec<&str> = cpe.split(":").collect();
                        let sql = "INSERT INTO tbl_cpe_from_cve (
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
        }
        None => {
            println!("no CVE_Items found");
        }
    }
    Ok(())
}

fn get_cpe_from_node(node: &serde_json::Value) -> Vec<&str> {
    let mut cpe_vec = Vec::new();
    match node["children"].as_array() {
        Some(children) => {
            if children.len() > 0 {
                for node_in_children in children {
                    cpe_vec.append(&mut get_cpe_from_node(node_in_children));
                }
                return cpe_vec;
            }
        }
        None => {
            println!("children is None");
        }
    }
    match node["cpe_match"].as_array() {
        Some(cpe_match_vec) => {
            if cpe_match_vec.len() > 0 {
                for cpe in cpe_match_vec {
                    cpe_vec.push(cpe["cpe23Uri"].as_str().unwrap());
                }
                return cpe_vec;
            }
        }
        None => {
            println!("cpe_match is None");
        }
    }
    cpe_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_cve() {
        let future = download_cve();
        let _ = tokio::join!(future);
    }

    #[tokio::test]
    async fn test_put_cpe_to_db() {
        let future = put_cpe_to_db();
        let _ = tokio::join!(future);
    }
}