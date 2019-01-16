//微信小程序实现
use crate::platform::*;
use base64::decode;
use std::io::Write;
use stdweb::web::ArrayBuffer;
use std::collections::HashMap;
use crate::teacher::Word;
use bincode::deserialize;

fn log(s: &str){
    js!(console.log(@{s}));
}

fn current_timestamp() -> i64{
    js!(return Date.now()).try_into().unwrap()
}

fn get_timezone_offset() -> i32{
    js!(return new Date().getTimezoneOffset()/60;).try_into().unwrap()
}

//------注册事件-------------
//查询字典
fn register_query_dict_listener(callback: &'static Fn()){
    js!(getApp().query_dict = @{callback});
}
//页面加载完
fn register_on_ready_listener(callback: &'static Fn()){
    js!(getApp().on_ready = @{callback});
}
//动画演示
fn register_stroke_anim_listener(callback: &'static Fn()){
    js!(getApp().stroke_anim = @{callback});
}
//橡皮擦
fn register_eraser_listener(callback: &'static Fn()){
    js!(getApp().eraser = @{callback});
}
//历史记录
fn register_history_listener(callback: &'static Fn()){
    js!(getApp().histories = @{callback});
}
//内容输入框
fn register_show_input(callback: &'static Fn()){
    js!(getApp().show_input = @{callback});
}
//内容选择
fn register_show_choose(callback: &'static Fn()){
    js!(getApp().show_choose = @{callback});
}
//回退上一个字
fn register_previous(callback: &'static Fn()){
    js!(getApp().previous = @{callback});
}
//搜索列表
fn register_search_title(callback: &'static Fn(String)){
    js!(getApp().search = @{callback});
}
//----------触摸事件--------------
fn register_on_touchstart(callback: &'static Fn(f64, f64, f64, f64)){
    js!(getApp().on_touchstart = @{callback};);
}
//反馈
fn register_add_feedback(callback: &'static Fn()){
    js!(getApp().add_feedback = @{callback});
}
fn add_feedback(feedback:&str, callback: &'static Fn(bool)){
    js!(getApp().page.addFeedback(@{feedback}, @{callback}));
}

fn register_on_touchmove(callback: &'static Fn(f64, f64, f64, f64)){
    js!(getApp().on_touchmove = @{callback};);
}
fn register_on_touchend(callback: &'static Fn()){
    js!(getApp().on_touchend = @{callback};);
}
fn register_on_touchcancel(callback: &'static Fn()){
    js!(getApp().on_touchcancel = @{callback};);
}
//--------------------------------
//读取笔画数据
fn load_strokes(callback: &mut FnMut(Option<Vec<u8>>)){
    let data:String = js!(return getApp().globalData.strokes;).try_into().unwrap();
    let data = decode(&data).unwrap();
    let mut decompressor = bzip2::write::BzDecoder::new(vec![]);
    decompressor.write_all(&data).unwrap();
    let data = decompressor.finish().unwrap();
    callback(Some(data));
}

//读取字典文件
fn load_dict(callback: &'static Fn(Result<HashMap<String, Word>, String>)){
    let scb = move |value: Value| {
        match value{
            Value::Undefined | Value::Null => callback(Err(format!("Undefined/Null {:?}", value))),
            Value::String(e) => callback(Err(e)),
            Value::Bool(b) => callback(Err(format!("Bool {}", b))),
            Value::Number(n) => callback(Err(format!("Number {:?}", n))),
            Value::Symbol(s) => callback(Err(format!("Symbol {:?}", s))),
            Value::Reference(n) => {
                let buffer:ArrayBuffer = n.try_into().unwrap();
                let buffer = Vec::from(buffer);
                let data: HashMap<String, Word> = deserialize(&buffer).unwrap();
                callback(Ok(data));
            }
        };
    };
    js!{
        var scb = @{scb};
        //字典解压的完整文件路径
        var path = getApp().globalData.userDataPath+"DICT";
        try{
            scb(wx.getFileSystemManager().readFileSync(path));
        }catch(e){
            //下载字典
            wx.cloud.downloadFile({
                fileID: "cloud://pub-ee5927.7075-pub-ee5927/DICT.zip",
                success: function(res){
                    //解压文件
                    wx.getFileSystemManager().unzip({
                        zipFilePath:res.tempFilePath,
                        //解压到userData根目录
                        targetPath: getApp().globalData.userDataPath,
                        success: function(){
                            scb(wx.getFileSystemManager().readFileSync(path));
                        },
                        fail: function(res){
                            console.log(res);
                            scb("字典文件解压失败");
                        },
                    });
                },
                fail: function (res) {
                    scb("字典下载失败");
                }
            });
        }
    };
}
// fn load_dict(callback: &'static Fn(Result<Vec<u8>, String>)){
//     let scb = move |value: Value| {
//         match value{
//             Value::Undefined | Value::Null => callback(Err(format!("Undefined/Null {:?}", value))),
//             Value::String(e) => callback(Err(e)),
//             Value::Bool(b) => callback(Err(format!("Bool {}", b))),
//             Value::Number(n) => callback(Err(format!("Number {:?}", n))),
//             Value::Symbol(s) => callback(Err(format!("Symbol {:?}", s))),
//             Value::Reference(n) => {
//                 let buffer:ArrayBuffer = n.try_into().unwrap();
//                 callback(Ok(Vec::from(buffer)));
//             }
//         };
//     };
//     js!{
//         var scb = @{scb};
//         var path = getApp().globalData.userDataPath+"DICT";
//         try{
//             scb(wx.getFileSystemManager().readFileSync(path));
//         }catch(e){
//             //下载字典
//             wx.cloud.downloadFile({
//                 fileID: "cloud://pub-ee5927.7075-pub-ee5927/DICT",
//                 success: function(res){
//                     //复制文件
//                     try{
//                         wx.getFileSystemManager().saveFileSync(res.tempFilePath, path);
//                         scb(wx.getFileSystemManager().readFileSync(path));
//                     }catch(e){
//                         scb("字典读取失败:"+JSON.stringify(e));
//                     }
//                 },
//                 fail: function (res) {
//                     scb("字典下载失败:"+JSON.stringify(res));
//                 }
//             });
//         }
//     };
// }

//提示相关
fn alert(title:&str, message: &str, callback: &'static Fn()){
    js!{
        getApp().page.showModal({
            showCancel: false,
            confirmText: "确定",
            title: @{title},
            content: @{message},
            success: function(res) {
                @{callback}();
            }
        });
    };
}
fn show_modal(title:&str, content: &str, show_cancel: bool, ok:&'static Fn(), cancel:&'static Fn()){
    js!{
        wx.showModal({
            title: @{title},
            content: @{content},
            showCancel: @{show_cancel},
            success: function(res) {
                if (res.confirm) {
                    @{ok}();
                } else if (res.cancel) {
                    @{cancel}();
                }
            }
        });
    };
}

//确认字符串数据
fn confirm_str(title:&str, content: &str, val:&str, ok:&'static Fn(String), cancel:&'static Fn()){
    js!{
        var val = @{val};
        wx.showModal({
            title: @{title},
            content: @{content},
            showCancel: true,
            success: function(res) {
                if (res.confirm) {
                    @{ok}(val);
                } else if (res.cancel) {
                    @{cancel}();
                }
            }
        });
    };
}

fn show_toast(title:&str, duration: i32, mask: bool, image: Option<String>){
    js!{
        var image = @{image};
        var data = {
            title: @{title},
            icon: "none",
            duration: @{duration},
            mask: @{mask},
        };
        if (image){
            data.image = image;
        }
        wx.showToast(data);
    };
}

fn show_loading(title:&str){
    js!{
        wx.showLoading({
        title: @{title},
        mask: true
        });
    };
}

fn hide_loading(){
    js!{wx.hideLoading();};
}

//显示输入对话框
fn show_input(text: &str, placeholder:&str, callback: &'static Fn(String)){
    js!{
        getApp().page.setData({ inputPlaceholder:@{placeholder}, inputContent: @{text}, showInput: true, showCanvasPlace: true});
        getApp().handle_input = @{callback};
    };
}

//显示选择对话框(列表1，当前项，列表2，当前项，callback(当前项1，当前项2))
fn show_choose(groups: Option<Vec<String>>, groud_id:i32, names: Option<Vec<String>>, name_id:i32, handle: Option<(&'static Fn(i32, i32), &'static Fn(i32, i32, i32, i32))>){
    js!{
        getApp().page.hideMenu();
        getApp().page.setData({
            exerciseValue:[@{groud_id}, @{name_id}], //当前选项
            showChoose: true,
            showCanvasPlace: true, });
    };
    if let Some(groups) = groups{
        js!(getApp().page.setData({exerciseGroups: @{groups}}));
    }
    if let Some(names) = names{
        js!(getApp().page.setData({exerciseNames: @{names}}));
    }
    if let Some(handle) = handle{
        js!{
            //绑定事件
            getApp().handle_choose = @{handle.0};
            getApp().handle_choose_change = @{handle.1};
        }
    }
}

//隐藏输入对话框
fn hide_input(){
    js!(getApp().page.setData({ showInput: false, showCanvasPlace: false}));
}

//显示练习记录
fn show_history(history: History, index: i32){
    js!{
        var page = getApp().page;
        var history = @{history};
        var index = @{index};
        var text_array = history.text.split("");
        page.setData({
            showHistoryStroke: true,
            showCanvasPlace: true,
            historyStrokeText: text_array,
        });
        getApp().handle_history = function(index){
            //画笔画
            setTimeout(function(){
                page.drawStrokes(history, index);
            }, 200);
        };
        getApp().handle_history(0);
    };
}

// //View相关
// fn window_width() -> i32{
//     js!(return wx.getSystemInfoSync().windowWidth;).try_into().unwrap()
// }
// fn window_height() -> i32{
//     js!(return wx.getSystemInfoSync().windowHeight;).try_into().unwrap()
// }

fn show_custom_toast(title:String, image: String, duration: i32){
    js!(clearTimeout(getApp().showToastTimeOut););
    js!{
        getApp().page.setData({
            scoreToastAnimation: "showtoast "+@{duration}+"ms",
            hideScoreToast: false,
            scoreToastImage: @{image},
            scoreToastText: @{title}
        });
        getApp().showToastTimeOut = setTimeout(function(){
            getApp().page.setData({
                hideScoreToast: true
            });
        }, @{duration});
    };
}

fn save_file(name:&str, content:Vec<u8>){
    js!{
        var path = getApp().globalData.userDataPath+@{name};
        wx.getFileSystemManager().writeFile({
            filePath: path,
            data: new Uint8Array(@{content}).buffer,
            success: function (res) {
                // console.log("文件保存成功", res);
            },
            fail: function (res) {
                // console.log("文件保存失败!", res);
            }
        });
    };
}

fn read_file(name:&str, callback: &'static Fn(Result<Vec<u8>, String>)){
    let cb = move |value:Value|{
        match value{
            Value::Undefined | Value::Null => callback(Err(format!("Undefined/Null {:?}", value))),
            Value::String(e) => callback(Err(e)),
            Value::Bool(b) => callback(Err(format!("Bool {}", b))),
            Value::Number(n) => callback(Err(format!("Number {:?}", n))),
            Value::Symbol(s) => callback(Err(format!("Symbol {:?}", s))),
            Value::Reference(n) => {
                let buffer:ArrayBuffer = n.try_into().unwrap();
                callback(Ok(Vec::from(buffer)));
            }
        };
    };
    js!{
        var cb = @{cb};
        var path = getApp().globalData.userDataPath+@{name};
        wx.getFileSystemManager().readFile({
            filePath: path,
            success: function (res) {
                // console.log("文件读取成功", res);
                cb(res.data);
            },
            fail: function (res) {
                // console.log("文件读取失败!", res);
                cb(res.errMsg);
            }
        });
    };
}

//存储相关
fn set_storage(name: &str, content: Vec<u8>) -> Result<(), String>{
    let result:Value = js!(try{
            var content = @{content};
            var name = @{name};
            //console.log("set_storage>>>> 存储 ", name, content);
            // console.log("保存:", name, content, "类型:",  typeof(content));
            wx.setStorageSync(name, content);
            return null;
        }catch(e){
            return e+"";
        }
    ).try_into().unwrap();

    match result{
        Value::String(s) => Err(s),
        _ => Ok(())
    }
}

fn get_storage(name: &str) -> Result<Vec<u8>, String>{
    let data:Value = js!(
        try {
            var name = @{name};
            var data = wx.getStorageSync(name);
            //console.log("get_storage>>>> 读取 ", name, data);
            //Array转换成arrayBuffer
            if(data && data.length>0){
                return new Uint8Array(data).buffer;
            }else{
                return "数据为空！";
            }
        } catch (e) {
            return e+"";
        }
    ).try_into().unwrap();

    match data{
        Value::Undefined | Value::Null => Err(format!("get_storage 调用失败/Undefined/Null: {:?}", data)),
        Value::String(e) => Err(format!("get_storage 调用失败/String: {}", e)),
        Value::Bool(b) => Err(format!("get_storage 调用失败: 布尔值 {}", b)),
        Value::Number(n) => Err(format!("get_storage 调用失败: Number {:?}", n)),
        Value::Symbol(n) => Err(format!("get_storage 调用失败: Symbol {:?}", n)),
        _ => {
            let buffer:ArrayBuffer = data.try_into().unwrap();
            Ok(Vec::from(buffer))
        }
    }
}

//动画相关
fn set_timeout(tid:i32, callback: &'static Fn(), delay: u32){
    js!{
        var timeouts = getApp().timeouts;
        var tid = @{tid};
        if (timeouts.has(tid)){//清空之前的动画
            clearTimeout(timeouts.get(tid));
        }
        var timeout = setTimeout(@{callback}, @{delay});
        timeouts.set(tid, timeout);
    };
}

fn clear_timeout(tid: i32){
    js!{
        if (getApp().timeouts && getApp().timeouts.has(@{tid})){
            clearTimeout(getApp().timeouts.get(@{tid}));
        }
    };
}

//提示相关
// fn alert(&self, title:&str, message: &str, ok: &'static Fn());
// fn show_modal(&self, title:&str, content: &str, show_cancel: bool, ok:&'static Fn(), cancel:&'static Fn());
// fn show_toast(&self, title:&str, duration: i32, mask: bool, image: Option<String>, );
// fn show_loading(&self, title:&str);
// fn hide_loading(&self);

struct MMPCanvasContext{}

impl CanvasContext for MMPCanvasContext{
    fn set_fill_style_color(&self, color: &str) {
        js!(getApp().canvasContext.setFillStyle(@{color}));
    }
    fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        js!(getApp().canvasContext.fillRect(@{x}, @{y}, @{width}, @{height}));
    }
    fn set_stroke_style_color(&self, color: &str) {
        js!(getApp().canvasContext.setStrokeStyle(@{color}));
    }
    fn set_line_width(&self, line_width: f64) {
        js!(getApp().canvasContext.setLineWidth(@{line_width}));
    }
    fn begin_path(&self) {
        js!(getApp().canvasContext.beginPath());
    }
    fn move_to(&self, x: f64, y: f64) {
        js!(getApp().canvasContext.moveTo(@{x},@{y}));
    }
    fn line_to(&self, x: f64, y: f64) {
        js!(getApp().canvasContext.lineTo(@{x},@{y}));
    }
    fn stroke(&self) {
        js!(getApp().canvasContext.stroke());
    }
    fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        js!(getApp().canvasContext.strokeRect(@{x}, @{y}, @{width}, @{height}));
    }
    fn fill_circle(&self, x: f64, y: f64, radius: f64) {
        js!{
            getApp().canvasContext.save();
            getApp().canvasContext.beginPath();
            getApp().canvasContext.arc(@{x}, @{y}, @{radius}, 0.0, 360.0, false);
            getApp().canvasContext.fill();
            getApp().canvasContext.restore();
        }
    }

    fn rotate(&self, angle: f64) {
        js!(getApp().canvasContext.rotate(@{angle}));
    }
    fn set_line_dash(&self, segments: Vec<f64>) {
        js!(getApp().canvasContext.setLineDash(@{segments}));
    }
    fn save(&self) {
        js!(getApp().canvasContext.save());
    }
    fn restore(&self) {
        js!(getApp().canvasContext.restore());
    }
    fn scale(&self, x: f64, y: f64) {
        js!(getApp().canvasContext.scale(@{x}, @{y}));
    }
    fn translate(&self, x: f64, y: f64) {
        js!(getApp().canvasContext.translate(@{x}, @{y}));
    }
    fn draw_image(&self, path: &str, x: i32, y: i32, width: i32, height: i32) {
        js!{
            getApp().canvasContext.drawImage(@{path},@{x}, @{y}, @{width}, @{height});
        };
    }
    fn draw_image_at(&self, path: &str, x: f64, y: f64) {
        js!{ getApp().canvasContext.drawImage(@{path},@{x}, @{y})};
    }
    fn draw(&self, callback: &'static Fn()){
        js!{
            var cb = @{callback};
            getApp().canvasContext.draw(false, function(e){
                cb();
            })
        };
    }

    fn set_font_size(&self, font_size:f64){
        js!(getApp().canvasContext.setFontSize(@{font_size}));
    }
    fn fill_text(&self, text:&str, x:f64, y:f64){
        js!(getApp().canvasContext.fillText(@{text}, @{x}, @{y}));
    }
}

struct IndexPage{
    context: Box<CanvasContext>,
}

impl MainPage for IndexPage{
    //获取画布
    fn canvas(&self) -> &Box<CanvasContext>{
        &self.context
    }
    
    fn canvas_width(&self) -> f64{
        js!(
            return getApp().page.data.renderSize;
        ).try_into()
        .unwrap()
    }
    fn canvas_height(&self) -> f64{
        js!(
            return getApp().page.data.renderSize;
        ).try_into()
        .unwrap()
    }

    //设置显示的字符(页面标签) [//在小程序中，程序分包根据需要切换不同字体]
    fn set_character(&self, c: char){
        let val = c.escape_unicode().to_string().replace("\\", "").replace("u{", "").replace("}", "");
        let z = i64::from_str_radix(&format!("{}", val), 16).unwrap();
        let class = if z<=28431{ "KaiTi_GB2312_0" }else{"KaiTi_GB2312_1"};
        js!{
            //置空、更新字体
            getApp().page.setData({ character: "", fontName: @{class}});
            //更新文字
            getApp().page.setData({ character: @{format!("{}", c)}});
        };
    }

    //作业完成进度预览
    fn update_homework(&self, content:&str, index: i32){
        let unicodes:Vec<String> = content.chars().map(|c|{
            c.escape_unicode().to_string().replace("\\", "").replace("u{", "").replace("}", "")
        }).collect();
        
        js!{
            var unicodes = @{unicodes};
            var content = @{content};
            var index = @{index};
            var words = [];
            for (var i = 0; i < content.length; i++) {
                words.push({ id: 'x'+i+unicodes[i], style: index > i ? "a" : "", text: content.charAt(i) });
            }
            //大概滚动到中间位置
            var i = index-5;
            if(i<0){
                i = 0;
            }
            getApp().page.setData({ words: words, scrollId: 'x' + i + unicodes[i]});
        };
    }

    //笔刷按钮闪烁动画
    fn start_brush_blink(&self){
        js!(getApp().page.setData({ brushDotHidden: false}););
    }
    fn stop_brush_blink(&self){
        js!(getApp().page.setData({ brushDotHidden: true}););
    }
}

fn get_page_context() ->Box<MainPage>{
    Box::new(IndexPage{context: Box::new(MMPCanvasContext{})})
}

fn platform_init(){
    js_serializable!( History );
    js!(getApp().timeouts = new Map());
}