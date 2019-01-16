var parser = require('xml2json');
var fs = require("fs");  

var xml = fs.readFileSync("./handwriting-zh_CN-gb2312.xml", "utf-8");

// xml to json
var json = parser.toJson(xml);

fs.writeFileSync("./handwriting-zh_CN-gb2312.json", json);