# cpe数据分析
## 分析目的
分析nvd官方的cpe-dictionary文件，统计厂商个数，应用个数

## 流程说明
- [x] 下载cpe字典文件，地址：https://nvd.nist.gov/feeds/xml/cpe/dictionary/official-cpe-dictionary_v2.3.xml.gz
- [x] 读取cpe xml压缩字典文件，解析cpe数据
- [x] 入库SQLite
- [x] 统计part，输出统计文件
- [x] 统计vendor，输出统计文件
- [x] 统计product，输出统计文件
