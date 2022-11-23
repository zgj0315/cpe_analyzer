use std::{fs::File, io::Write};

use rusqlite::Connection;

pub async fn cpe_clean() -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn cpe_stat() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("./data/cpe.db").unwrap();
    let mut stmt = conn
        .prepare("SELECT part, count(*) from tbl_cpe_cve GROUP BY part ORDER BY count(*) DESC")
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
        .prepare("SELECT part, vendor, count(*) from tbl_cpe_cve GROUP BY part, vendor ORDER BY count(*) DESC")
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
            "SELECT part, vendor, product, count(*) from tbl_cpe_cve GROUP BY part, vendor, product ORDER BY count(*) DESC",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cpe_clean() {
        let future = cpe_clean();
        let _ = tokio::join!(future);
    }

    #[tokio::test]
    async fn test_cpe_stat() {
        let future_cpe_stat = cpe_stat();
        let _ = tokio::join!(future_cpe_stat);
    }
}
