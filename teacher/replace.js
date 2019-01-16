var fs = require("fs");  

var asmjs = fs.readFileSync("./target/asmjs-unknown-emscripten/release/teacher.js", "utf-8");

//注释掉两个 Module["arguments"]=arguments
var new_content = asmjs.replace('Module["arguments"]=arguments', '');
new_content = new_content.replace('ENVIRONMENT_IS_WORKER=typeof importScripts==="function";', '');
new_content = new_content.replace('Module["arguments"]=arguments', '');
//处理字体需要加大内存(2的整数倍)
new_content = new_content.replace('Module["TOTAL_STACK"]||5242880', 'Module["TOTAL_STACK"]||parseInt(5242880*4)');
new_content = new_content.replace('Module["TOTAL_MEMORY"]||16777216', 'Module["TOTAL_MEMORY"]||parseInt(16777216*4)');

fs.writeFileSync("../mpproj/teacher/pages/index/teacher.js", new_content);