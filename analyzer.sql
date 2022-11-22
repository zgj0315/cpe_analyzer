
-- 不在字典里的cpe

select count(*) from tbl_cpe_from_cve where product not in (select product from tbl_cpe)

select * from tbl_cpe_from_cve where product not in (select product from tbl_cpe)

-- 不在cve里的cpe

select count(*) from tbl_cpe where product not in (select product from tbl_cpe_from_cve)

select * from tbl_cpe where product not in (select product from tbl_cpe_from_cve)

-- 字典和cve里都有的cpe

select count(*) from tbl_cpe_from_cve where product in (select product from tbl_cpe)

select count(*) from tbl_cpe where product in (select product from tbl_cpe_from_cve)

select * from tbl_cpe_from_cve where product in (select product from tbl_cpe)

select * from tbl_cpe where product in (select product from tbl_cpe_from_cve)

-- 整理数据

select distinct part, vendor, product from tbl_cpe;

select distinct part, vendor, product from tbl_cpe_from_cve;

drop table tbl_cpe_dict;

create table tbl_cpe_dict as select part, vendor, product from tbl_cpe limit 0;

insert into tbl_cpe_dict(part, vendor, product) select distinct part, vendor, product from tbl_cpe;

drop table tbl_cpe_cve;

create table tbl_cpe_cve as select part, vendor, product from tbl_cpe limit 0;

insert into tbl_cpe_cve(part, vendor, product) select distinct part, vendor, product from tbl_cpe_from_cve;

select count(*) from tbl_cpe_dict;

select count(*) from tbl_cpe_cve;

-- 不在字典里的cpe

select count(*) from tbl_cpe_cve where product not in (select product from tbl_cpe_dict)

select * from tbl_cpe_cve where product not in (select product from tbl_cpe_dict)

-- 不在cve里的cpe

select count(*) from tbl_cpe_dict where product not in (select product from tbl_cpe_cve)

select * from tbl_cpe_dict where product not in (select product from tbl_cpe_cve)
