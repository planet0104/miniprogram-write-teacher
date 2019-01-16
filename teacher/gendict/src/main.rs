extern crate bincode;
use bincode::{serialize, deserialize};
extern crate bzip2;
use bzip2::Compression;
extern crate serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use serde_json::Value;
use std::io::prelude::*;
extern crate base64;
use base64::{encode, decode};
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Word {
    pub strokes: String,
    pub pinyin: String,
    pub radicals: String,
    pub explanation: String,
}

enum Status{
    AppendLine,
    WaitStart,
}

enum LineType{
    Normal,
    Blank,
    Start
}

fn main111(){
    let mut sarr = vec![String::from("010"), String::from("100"), String::from("012")];
    sarr.sort();
    println!("{:?}", sarr);
}

fn gen_articls(){
    let mut file = File::open("课文.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let num:Vec<char> = "0123456789".chars().collect();

    let mut articls = HashMap::new();

    let mut article_title = String::new();
    let mut article_content = String::new();
    let mut status = Status::WaitStart;

    let mut titles = vec![];

    for line in contents.lines(){
        let line_type = {
            let s = line.trim();
            let chars = s.chars();
            let count = chars.count();
            if count == 0{
                LineType::Blank
            }else{
                let ch = line.chars().next().unwrap();
                if let Ok(_idx) = num.binary_search(&ch){
                    LineType::Start
                }else{
                    LineType::Normal
                }
            }
        };

        match line_type{
            LineType::Start => {
                if let Status::AppendLine = status{
                    //保存当前poem
                    articls.insert(String::from(article_title.as_str()), String::from(article_content.as_str()));
                    println!("添加:{} total:{}", article_title, articls.len());
                }
                //作者和诗名
                let mut title_str = String::from(line);
                titles.push(line);
                println!("----------------------------------------------------{} 总共:{}", line, titles.len());
                let mut title_key = String::from(line);
                title_key.retain(|ch|{
                    num.binary_search(&ch).is_ok()
                });

                title_str.retain(|ch|{
                    num.binary_search(&ch).is_err()
                });
                //保存标题
                article_title = title_key+"-"+&title_str;
                //清空内容
                article_content = String::new();
                //修改状态
                status = Status::AppendLine;
            }
            LineType::Normal => {
                if let Status::AppendLine = status{
                    article_content.push_str(line);
                }
            },
            LineType::Blank => {
                if let Status::AppendLine = status{
                    status = Status::WaitStart;
                    //保存当前poem
                    articls.insert(String::from(article_title.as_str()), String::from(article_content.as_str()));
                    println!("添加:{} total:{}", article_title, articls.len());
                }
            }
        }
    }
    // for (key, val) in &poems{
    //     println!("{}", key);
    // }
    println!("总数:{} titles={}", articls.len(), titles.len());

    //序列化
    let encoded: Vec<u8> = serialize(&articls).unwrap();

    //压缩
    let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
    zip.write_all(&encoded).unwrap();
    let result = zip.finish().unwrap();

    //写入文件
    let mut file = File::create("ARTICLS").unwrap();
    file.write_all(&result).unwrap();
}

fn gen_strokes(){
    //bzip压缩
    let strokes = include_bytes!("../gb2312.data");
    let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
    zip.write_all(strokes).unwrap();
    let result = zip.finish().unwrap();
    let mut file = File::create("strokes.bzip").unwrap();
    file.write_all(&result).unwrap();

    let base64 = encode(&result);
    let mut file = File::create("strokes.bzip.txt").unwrap();
    file.write_all(base64.as_bytes()).unwrap();
}

fn main(){
    //gen_dict();
    gen_dict_v2();
    // gen_strokes();
    // gen_articls();
    // gen_pinyin();
}

fn maintttt(){
    let mut file = File::open("唐诗三百首.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let num:Vec<char> = "0123456789".chars().collect();
    let mut total = 0;

    //let mut author = String::new();
    //let mut title = String::new();

    enum Status{
        AppendLine,
        WaitStart,
    }

    enum LineType{
        Normal,
        Blank,
        Start
    }

    let mut poems = HashMap::new();

    let mut poem_title = String::new();
    let mut poem_content = String::new();
    let mut status = Status::WaitStart;

    let mut titles = vec![];

    for line in contents.lines(){
        //如果是空行，并且状态是AppendLine，那么切换成WaitStart，【保存当前行】
        //如果是空行，并且状态是WaitStart，不做更改。
        //如果是正常行，并且状态是WaitStart，不做更改。
        //如果是正常行，并且状态时AppendLine，【poem_content加入当前行】。状态不做更改。
        //如果是起始，并且状态是WaitStart，那么【创建新的poem_title】，切换成AppendLine
        //如果时起始，并且状态是AppendLine，【保存当前行，创建新的poem_title】

        let line_type = {
            let s = line.trim();
            let chars = s.chars();
            let count = chars.count();
            if count == 0{
                LineType::Blank
            }else{
                let ch = line.chars().next().unwrap();
                if let Ok(_idx) = num.binary_search(&ch){
                    LineType::Start
                }else{
                    LineType::Normal
                }
            }
        };

        match line_type{
            LineType::Start => {
                if let Status::AppendLine = status{
                    //保存当前poem
                    poems.insert(String::from(poem_title.as_str()), String::from(poem_content.as_str()));
                    println!("添加:{} total:{}", poem_title, poems.len());
                }
                //作者和诗名
                let mut title_str = String::from(line);
                titles.push(line);
                println!("----------------------------------------------------{} 总共:{}", line, titles.len());
                let mut title_key = String::from(line);
                title_key.retain(|ch|{
                    num.binary_search(&ch).is_ok()
                });

                title_str.retain(|ch|{
                    num.binary_search(&ch).is_err()
                });
                //保存标题
                poem_title = title_key+"-"+&title_str;
                //清空内容
                poem_content = String::new();
                //修改状态
                status = Status::AppendLine;
            }
            LineType::Normal => {
                if let Status::AppendLine = status{
                    poem_content.push_str(line);
                }
            },
            LineType::Blank => {
                if let Status::AppendLine = status{
                    status = Status::WaitStart;
                    //保存当前poem
                    poems.insert(String::from(poem_title.as_str()), String::from(poem_content.as_str()));
                    println!("添加:{} total:{}", poem_title, poems.len());
                }
            }
        }
    }
    // for (key, val) in &poems{
    //     println!("{}", key);
    // }
    println!("总数:{} titles={}", poems.len(), titles.len());

    //序列化
    let encoded: Vec<u8> = serialize(&poems).unwrap();
    //写入文件
    let mut file = File::create("POEMS").unwrap();
    file.write_all(&encoded).unwrap();
}

// fn main(){
//     let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
//     zip.write_all(gb2312).unwrap();
//     let result = zip.finish().unwrap();
//     let mut file = File::create("gb2312.data.bzip").unwrap();
//     file.write_all(&result).unwrap();

//     let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
//     zip.write_all(ttf).unwrap();
//     let result = zip.finish().unwrap();
//     let mut file = File::create("KaiTi_GB2312.ttf.bzip").unwrap();
//     file.write_all(&result).unwrap();

//     let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
//     zip.write_all(word).unwrap();
//     let result = zip.finish().unwrap();
//     let mut file = File::create("WORD.bzip").unwrap();
//     file.write_all(&result).unwrap();
// }

fn gen_pinyin() {
    let mut file = File::open("word.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let v: Value = serde_json::from_str(&contents).unwrap();
    let array = v.as_array().unwrap();

    //填充
    let mut data:HashMap<char, &str> = HashMap::new();
    for character in array{
        let info = character.as_object().unwrap();
        let word = info["word"].as_str().unwrap();
        let pinyin = info["pinyin"].as_str().unwrap();
        data.insert(word.chars().next().unwrap(), pinyin);
    }

    //序列化整个字典
    let pinyin: Vec<u8> = serialize(&data).unwrap();

    //压缩
    let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
    zip.write_all(&pinyin).unwrap();
    let result = zip.finish().unwrap();

    //写入文件
    let mut file = File::create("PINYIN").unwrap();
    file.write_all(&result).unwrap();
}

fn gen_dict_v2() {
    let strokes = include_bytes!("../gb2312.data");
    let strokes_map:HashMap<char, Vec<Vec<(u16,u16)>>> = deserialize(strokes).unwrap();

    let mut file = File::open("word.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let v: Value = serde_json::from_str(&contents).unwrap();
    let array = v.as_array().unwrap();

    //搜索字典中不存在的词
    let mut all = vec![];
    for character in array{
        let info = character.as_object().unwrap();
        let word = info["word"].as_str().unwrap();
        all.push(word);
    }
    let mut count = 0;
    for (ch, _) in strokes_map{
        if all.contains(&format!("{}", ch).as_str()){

        }else{
            count += 1;

            let mut data = r#"    {
                    "word": "_word_",
                    "oldword": "",
                    "strokes": "",
                    "pinyin": "",
                    "radicals": "",
                    "explanation": "",
                    "more": ""
                },"#;
            let data = data.replace("_word_", &format!("{}", ch));
            
            println!("{}", data);
        }
    }
    println!("总共缺少{}个字的解释", count);

    //填充
    let mut data:HashMap<String, Word> = HashMap::new();
    for character in array{
        let info = character.as_object().unwrap();
        let word = info["word"].as_str().unwrap();
        let strokes  = info["strokes"].as_str().unwrap();
        let pinyin = info["pinyin"].as_str().unwrap();
        let radicals = info["radicals"].as_str().unwrap();
        let explanation = info["explanation"].as_str().unwrap();

        let word_data = Word{
            strokes: String::from(strokes),
            pinyin: String::from(pinyin), 
            radicals: String::from(radicals), 
            explanation: String::from(explanation)
        };

        data.insert(word.to_string(), word_data);
    }

    //序列化整个字典
    let dict: Vec<u8> = serialize(&data).unwrap();

    //写入文件
    let mut file = File::create("DICT").unwrap();
    file.write_all(&dict).unwrap();
}

fn gen_dict() {

    let strokes = include_bytes!("../gb2312.data");
    let strokes_map:HashMap<char, Vec<Vec<(u16,u16)>>> = deserialize(strokes).unwrap();

    let mut file = File::open("word.json").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    //map中保存压缩的数据Hashmap<String, Vec<Word>>, key=笔画数
    let mut data:HashMap<String, Vec<u8>> = HashMap::new();

    let v: Value = serde_json::from_str(&contents).unwrap();
    let array = v.as_array().unwrap();

    //搜索字典中不存在的词
    let mut all = vec![];
    for character in array{
        let info = character.as_object().unwrap();
        let word = info["word"].as_str().unwrap();
        all.push(word);
    }
    let mut count = 0;
    for (ch, _) in strokes_map{
        if all.contains(&format!("{}", ch).as_str()){

        }else{
            count += 1;

            let mut data = r#"    {
                    "word": "_word_",
                    "oldword": "",
                    "strokes": "",
                    "pinyin": "",
                    "radicals": "",
                    "explanation": "",
                    "more": ""
                },"#;
            let data = data.replace("_word_", &format!("{}", ch));
            
            println!("{}", data);
        }
    }
    println!("总共缺少{}个字的解释", count);

    //填充
    let mut tmp_data:HashMap<String, HashMap<char, Word>> = HashMap::new();
    for character in array{
        let info = character.as_object().unwrap();
        let word = info["word"].as_str().unwrap();
        let strokes  = info["strokes"].as_str().unwrap();
        let pinyin = info["pinyin"].as_str().unwrap();
        let radicals = info["radicals"].as_str().unwrap();
        let explanation = info["explanation"].as_str().unwrap();

        let word_data = Word{
            strokes: String::from(strokes),
            pinyin: String::from(pinyin), 
            radicals: String::from(radicals), 
            explanation: String::from(explanation)
        };

        if !tmp_data.contains_key(strokes){
            tmp_data.insert(String::from(strokes), HashMap::new());
        }
        tmp_data.get_mut(strokes).as_mut().unwrap().insert(word.chars().next().unwrap(), word_data);
    }

    for (key, map) in tmp_data{
        println!("笔画数:{} 汉字个数:{}", key, map.len());
        //序列化
        let encoded: Vec<u8> = serialize(&map).unwrap();
        //压缩
        let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Fastest);
        zip.write_all(&encoded).unwrap();
        let result = zip.finish().unwrap();
        data.insert(key, result);
    }

    //序列化整个字典
    let dict: Vec<u8> = serialize(&data).unwrap();

    //写入文件
    let mut file = File::create("DICT").unwrap();
    file.write_all(&dict).unwrap();
}

fn main1() {

    // let mut file = File::open("../handwriting-zh_CN-gb2312.json").unwrap();
    // let mut contents = String::new();
    // file.read_to_string(&mut contents).unwrap();

    // let mut data:HashMap<char, Vec<Vec<(u16,u16)>>> = HashMap::new();

    // let v: Value = serde_json::from_str(&contents).unwrap();
    // //所有字符数组
    // let characters = v["dictionary"]["character"].as_array().unwrap();
    // for character in characters{
    //     //let ch = character.as_object().unwrap();
    //     //strokes - 对象
    //     //stroke - 对象/对象数组
    //     let stroke = &character["strokes"]["stroke"];
    //     let mut c_strokes = vec![];
    //     if stroke.is_object(){
    //         //单笔画
    //         let points = stroke["point"].as_array().unwrap();
    //         c_strokes.push(points.iter().map(|v| {
    //             (FromStr::from_str(v["x"].as_str().unwrap()).unwrap(), FromStr::from_str(v["y"].as_str().unwrap()).unwrap())
    //         }).collect());
    //     }else{
    //         //多笔画
    //         let strokes = stroke.as_array().unwrap();
    //         for stroke in strokes{
    //             //单笔画
    //             let points = stroke["point"].as_array().unwrap();
    //             c_strokes.push(points.iter().map(|v| {
    //                 (FromStr::from_str(v["x"].as_str().unwrap()).unwrap(), FromStr::from_str(v["y"].as_str().unwrap()).unwrap())
    //             }).collect());
    //         }
    //     }
    //     data.insert(character["utf8"].as_str().unwrap().chars().next().unwrap(), c_strokes);
    // }

    //存储文件-----------------------------------------------

    //序列化
    // let encoded: Vec<u8> = serialize(&data).unwrap();

    let font_data = include_bytes!("../KaiTi_GB2312.ttf");

    println!(">>11 {}", font_data.len());

    //压缩
    let mut zip = bzip2::write::BzEncoder::new(vec![], Compression::Best);
    zip.write_all(font_data).unwrap();
    let result = zip.finish().unwrap();

    println!(">>22 {}", result.len());

    //写入文件
    // let mut file = File::create("../KaiTi_GB2312.bzip").unwrap();
    // file.write_all(&result).unwrap();

    let base64 = encode(&result);
    
    let sp4 = base64.len()/7;

    let part1 = &base64[0..sp4*6];
    let part2 = &base64[sp4*6..];

    let mut file = File::create("KaiTi_GB2312.bzip.base64.part1.txt").unwrap();
    file.write_all(part1.as_bytes()).unwrap();

    let mut file = File::create("KaiTi_GB2312.bzip.base64.part2.txt").unwrap();
    file.write_all(part2.as_bytes()).unwrap();

    //--------------------------------------------------------
}