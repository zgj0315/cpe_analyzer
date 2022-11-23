use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use rusqlite::Connection;
use serde_json::Value;

pub async fn download_cve() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./data");
    if !path.exists() {
        fs::create_dir(path).unwrap();
    }
    let mut year = 2002;
    while year <= 2022 {
        let file = format!("nvdcve-1.1-{}.json.zip", year);
        let url = format!("https://nvd.nist.gov/feeds/json/cve/1.1/{}", file);
        let file_path = format!("./data/{}", file);
        let path = Path::new(&file_path);
        let mut file = match File::create(&path) {
            Err(e) => panic!("Error creating {}", e),
            Ok(file) => file,
        };
        let rsp = reqwest::get(url).await?;
        let rsp_bytes = rsp.bytes().await?;
        let _ = file.write_all(&rsp_bytes);
        log::info!("{:?} downloaded successfully", path.file_name().unwrap());
        year += 1;
    }
    Ok(())
}

pub async fn put_cpe_to_db() -> Result<(), Box<dyn std::error::Error>> {
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
    let mut year = 2002;
    while year <= 2022 {
        let zip_file = format!("nvdcve-1.1-{}.json.zip", year);
        let json_file = format!("nvdcve-1.1-{}.json", year);
        let zpi_file_path = format!("./data/{}", zip_file);
        let zip_file = File::open(zpi_file_path).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        let file = archive.by_name(&json_file).unwrap();
        let json: Value = serde_json::from_reader(file).unwrap();
        match json["CVE_Items"].as_array() {
            Some(cve_items) => {
                for cve_item in cve_items.into_iter() {
                    let nodes = &cve_item["configurations"]["nodes"].as_array().unwrap();
                    for node in nodes.into_iter() {
                        let cpe_vec = get_cpe_from_node(node);
                        if cpe_vec.len() == 0 {
                            log::info!("not find cpe, nodes: {:?}", node);
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
                log::info!("no CVE_Items found");
            }
        }
        year += 1;
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
            log::info!("children is None");
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
            log::info!("cpe_match is None");
        }
    }
    cpe_vec
}

pub async fn clean_data() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/cpe.db").unwrap();
    let sql = "DROP TABLE IF EXISTS tbl_cpe_dict";
    conn.execute(sql, ()).unwrap();
    let sql = "CREATE TABLE IF NOT EXISTS tbl_cpe_dict (
        part  TEXT NOT NULL,
        vendor  TEXT NOT NULL,
        product  TEXT NOT NULL
    )";
    conn.execute(sql, ()).unwrap();
    let sql = "INSERT INTO tbl_cpe_dict(part, vendor, product) SELECT DISTINCT part, vendor, product FROM tbl_cpe";
    conn.execute(sql, ()).unwrap();
    let sql = "DROP TABLE IF EXISTS tbl_cpe_cve";
    conn.execute(sql, ()).unwrap();
    let sql = "CREATE TABLE IF NOT EXISTS tbl_cpe_cve (
        part  TEXT NOT NULL,
        vendor  TEXT NOT NULL,
        product  TEXT NOT NULL
    )";
    conn.execute(sql, ()).unwrap();
    let sql = "INSERT INTO tbl_cpe_cve(part, vendor, product) SELECT DISTINCT part, vendor, product FROM tbl_cpe_from_cve";
    conn.execute(sql, ()).unwrap();
    Ok(())
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

    #[tokio::test]
    async fn test_clean_data() {
        let future = clean_data();
        let _ = tokio::join!(future);
    }
}
