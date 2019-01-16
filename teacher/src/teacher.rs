use crate::pdollarplus::{resample, PDollarPlusRecognizer, Point};
use crate::platform::*;
use bincode::{deserialize, serialize};
use chrono::offset::FixedOffset;
use chrono::{DateTime, NaiveDateTime};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
//图标集 https://www.iconfinder.com/icons/1054952/chart_graph_trends_icon

const KEY_INPUTS: &str = "inputs";
const KEY_HOMEWORK: &str = "homework";
const KEY_HISTORY: &str = "history";
const KEY_CHOOSE: &str = "choose";
use crate::log;

static POEMS: &[u8] = include_bytes!("../POEMS");
static ARTICLS: &[u8] = include_bytes!("../ARTICLS");
// static PINYIN: &[u8] = include_bytes!("../PINYIN");


const DELAY_ANIM:u32 = 30; //动画模式
const DELAY_IDLE:u32 = 9000; //空闲模式

thread_local! {
    static CONTROLLER: RefCell<Controller> = RefCell::new(Controller::new());
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Word {
    pub strokes: String,
    pub pinyin: String,
    pub radicals: String,
    pub explanation: String,
}

//保存的练习记录
#[derive(Serialize, Deserialize, Debug)]
pub struct Homework {
    group: i32,      //分组:0自定义,1古诗,2课文
    title: String,   //古诗、课文对应的标题(索引)
    content: String, //练习内容
    progress: usize, //练习进度
}

pub struct Controller {
    pinyin: HashMap<char, String>,
    page: Box<MainPage>,
    poems: HashMap<String, String>,
    poems_map: Vec<(String, String)>, //Vec<(Name, Key)>
    articls: HashMap<String, String>,
    articls_map: Vec<(String, String)>, //Vec<(Name, Key)>
    strokes_map: HashMap<char, Vec<Vec<(u16, u16)>>>,
    dict: Option<HashMap<String, Word>>,
    stroke_anim: Vec<Point>, //当前动画的指示器，每当新的笔画开始，数组清空，每一帧的时候放入一个当前笔画的点，直到数组长度和当前笔画点数相等。
    pub character: usize,    //当前第几个字
    drawing_ch: Option<char>, //当前绘制的字
    strokes: Vec<Vec<Point>>,
    stroke_index: usize,
    all_user_strokes: Vec<Vec<Vec<Point>>>, //用户所有字的笔画数据
    all_user_strokes_score: Vec<f64>,       //用户每个字的得分
    user_strokes: Vec<Vec<Point>>,          //当前字用户的笔画
    stroke_scores: Vec<f64>,                //当前字的笔画得分
    writing: bool,                          //正在写入
    complete: bool,                         //当前文字已写完,
    pub homework: Vec<char>,
    pub homework_str: String,
    error_icon: Option<Point>,
    last_point: Option<(f64, f64)>, //draw()方法正在执行
    delay: u32, //连续绘制间隔
    stroke_animation: Option<i32>, //笔画动画是否在执行 0-错误提示动画, 1-整字动画
    last_render_time: i64,

    /*
        更新逻辑:
        默认帧率 1FPS
        切换字符、笔画演示、错误提示、用户触摸(start,touchmove) 时， 帧率切为20FPS (间隔50ms)
        触摸结束(touchend,touchcancel)、动画演示结束、错误提示结束 时，帧率降低为 1FPS (间隔1000ms)
    */
}

impl Controller {
    pub fn new() -> Controller{
        //反序列化
        let poems: HashMap<String, String> = deserialize(POEMS).unwrap();
        //提取所有古诗的名字列表，以及对应的key
        let mut keys: Vec<String> = poems.keys().map(|s| String::from(s.as_str())).collect();
        keys.sort();
        let poems_map = keys
            .iter()
            .map(|s| (s.split("-").last().unwrap().to_string(), s.clone()))
            .collect();

        //提取所有文章的名字列表，以及对应的key
        //解压
        let mut decompressor = bzip2::write::BzDecoder::new(vec![]);
        decompressor.write_all(&ARTICLS).unwrap();
        let slice = decompressor.finish().unwrap();
        let articls: HashMap<String, String> = deserialize(&slice).unwrap();
        let mut keys: Vec<String> = articls.keys().map(|s| String::from(s.as_str())).collect();
        keys.sort();
        let articls_map = keys
            .iter()
            .map(|s| (s.split("-").last().unwrap().to_string(), s.clone()))
            .collect();

        //------------ 拼音列表 --------------------
        //解压
        // let mut decompressor = bzip2::write::BzDecoder::new(vec![]);
        // decompressor.write_all(&PINYIN).unwrap();
        // let slice = decompressor.finish().unwrap();
        // let pinyin: HashMap<char, String> = deserialize(&slice).unwrap();
        let pinyin = HashMap::new();

        Controller {
            pinyin: pinyin,
            page: crate::get_page_context(),
            poems,
            poems_map,
            articls,
            articls_map,
            strokes_map: HashMap::new(),
            dict: None,
            stroke_anim: vec![],
            character: 0,
            stroke_index: 0,
            strokes: vec![],
            user_strokes: vec![],
            all_user_strokes: vec![],
            all_user_strokes_score: vec![],
            writing: false,
            complete: false,
            homework: vec![],
            homework_str: String::new(),
            delay: 1000,
            drawing_ch: None,
            stroke_scores: vec![],
            error_icon: None,
            last_point: None,
            stroke_animation: None,
            last_render_time: 0
        }
    }

    //立即绘制并开始以指定的时间间隔连续绘制
    fn reset_delay(&mut self, delay:u32, render:bool){
        if self.delay != delay || render{
            self.delay = delay;
            crate::clear_timeout(0);
            self.render();
        }
    }

    fn on_ready(&mut self) {
        //js!(console.log("on_ready>>>", new Date(), Date.now()));
        crate::show_loading("加载笔画");
        //加载笔画
        crate::load_strokes(&mut |data| {
            let decoded: HashMap<char, Vec<Vec<(u16, u16)>>> =
                deserialize(data.as_ref().unwrap()).unwrap();
                
            self.strokes_map = decoded;
            //加载练习内容
            self.load_homework();
        });
    }

    fn load_homework(&mut self) {
        //js!(console.log("load_homework>>>", new Date(), Date.now()));
        //读取被保存的练习内容
        let homework: Homework = if let Ok(saved) = &crate::get_storage(KEY_HOMEWORK) {
            deserialize(&saved).unwrap()
        } else {
            let mut keys: Vec<String> = self
                .articls
                .keys()
                .map(|s| String::from(s.as_str()))
                .collect();
            keys.sort();
            Homework {
                group: 2,
                title: keys[0].clone(),
                content: self.articls.get(&keys[0]).unwrap().clone(),
                progress: 0,
            }
        };
        //设置练习内容显示文本
        self.set_homework(&homework.content);
        //加载笔画
        self.create_strokes();
        crate::hide_loading();
    }

    //更新动画
    pub fn update(&mut self){
        // log("update>>>");
        if let Some(anim_type) = self.stroke_animation{
            let anim_len = self.stroke_anim.len();
            if anim_len < self.strokes[self.stroke_index].len() {
                //添加下一个笔画点
                self.stroke_anim
                    .push(self.strokes[self.stroke_index][anim_len].clone());
                self.stroke_anim
                    .push(self.strokes[self.stroke_index][anim_len + 1].clone());
            } else {
                self.stroke_anim.clear();
                //如果是整笔动画演示, 每个笔画结束以后自动切换到下一笔画
                if anim_type==1 && self.stroke_index < self.strokes.len() - 1 {
                    self.stroke_index += 1;
                } else {
                    //动画结束
                    if anim_type==1 {
                        self.page.stop_brush_blink();
                        //恢复到第一笔
                        self.stroke_index = 0;
                    }
                    //如果是单笔动画演示, 笔画结束以后清空动画
                    //self.animating = false;
                    // log(&format!("第{}笔动画结束", self.stroke_index));
                    //log("笔画动画结束，切换为慢速帧率.");

                    self.stroke_animation = None;

                    //如果没有触摸，恢复慢速动画
                    if !self.writing{
                        self.reset_delay(DELAY_IDLE, true); //切换到慢速帧率
                    }
                }
            }
        }
    }

    //绘制
    pub fn render(&mut self) {
        //log("render..");
        self.last_render_time = crate::current_timestamp();
        let page = &self.page;
        let context = page.canvas();

        let (width, _height) = (page.canvas_width(), page.canvas_height());

        //--------------------- 画拼音 ----------------------------------
        if let Some(pinyin) = self.pinyin.get(&self.homework[self.character]){
            context.set_font_size(20.0);
            context.set_fill_style_color("#959595");
            context.fill_text(pinyin, 120.0, 20.0);
        }

        //------------------ 绘制用户的笔画 -----------------------------
        context.set_stroke_style_color("#333");
        context.set_fill_style_color("#333");
        let line_width = width * 0.035;
        context.set_line_width(line_width);

        for points in &self.user_strokes {
            let slen = points.len();
            if slen > 1 {
                context.fill_circle(points[0].x, points[0].y, line_width / 2.0);
            }

            context.begin_path();
            context.move_to(points[0].x, points[0].y);
            for point in points {
                context.line_to(point.x, point.y);
            }
            let len = points.len();
            context.stroke();

            if slen > 1 {
                context.fill_circle(points[len - 1].x, points[len - 1].y, line_width / 2.0);
            }
        }

        //--------------------- 画笔动画 --------------------------
        context.set_stroke_style_color("#000088");
        context.begin_path();
        context.set_line_dash(vec![]);
        context.set_line_width(1.0);
        context.save();
        //测试笔画
        //笔画路径
        //原始宽高 900x900, dx=180,dy=85
        //计算比例
        let scale = width as f64 / 1000.0;
        //platform.translate(scale*88.0, scale*48.0);
        context.scale(scale, scale);

        //------------ 正确笔画 ------------------------------
        /*
        context.begin_path();//
        let strokes:&Vec<Vec<(u16,u16)>> = self.strokes_map.get(&self.homework[self.character]).unwrap();
        for points in strokes{
            // if points.len()>6{
            //     platform.stroke_rect(points[6][0] as f64, points[6][1] as f64, (10) as f64, (10)as f64);
            // }
            context.move_to(points[0].0 as f64, points[0].1 as f64);
            for i in 1..points.len(){
                context.line_to(points[i].0 as f64, points[i].1 as f64);
            }
        }
        for points in &self.strokes{
            context.move_to(points[0].x, points[0].y);
            for i in 1..points.len(){
                context.line_to(points[i].x, points[i].y);
            }
        }
        context.stroke();
        */
        //------------------------------------------------------

        //如果需要，绘制错误提示
        if let Some(pos) = &self.error_icon {
            context.draw_image_at("/static/icons/pen.png", pos.x + 20.0, pos.y - 30.0);
        }

        //绘制画笔
        if self.stroke_animation.is_some(){
            if self.stroke_anim.len() > 0 {
                context.draw_image_at(
                    "/static/icons/hand.png",
                    self.stroke_anim.last().unwrap().x,
                    //self.stroke_anim.last().unwrap().y - platform.brush_height(),
                    self.stroke_anim.last().unwrap().y + 10.0,
                );
            }
        }

        context.restore();
        // if !self.render{
            
        // }
        //self.render = true;
        context.draw(&||{
            CONTROLLER.with(|ctrl| {
                //下一帧
                let ctrl = ctrl.borrow_mut();
                crate::set_timeout(
                    0,
                    &|| {
                        CONTROLLER.with(|c| {
                            c.borrow_mut().next_frame();
                        });
                    },
                    ctrl.delay,
                );
            });
        });
    }

    fn next_frame(&mut self){
        self.update();
        self.render();
    }

    pub fn set_homework(&mut self, content: &str) {
        let content = String::from(content.trim());
        //选择一个支持的字符
        let mut index = -1;
        for (i, ch) in content.chars().enumerate() {
            if self.strokes_map.contains_key(&ch) {
                index = i as i32;
                break;
            }
        }
        if index == -1 {
            //没有找到文字!
            if content.len() == 0 {
                crate::alert("提示", "请输入要练习的文字！", &|| {});
            }
        } else {
            self.complete = false;
            self.character = index as usize;
            self.stroke_index = 0;
            self.stroke_anim.clear();
            self.stroke_scores.clear();
            self.user_strokes.clear();
            self.all_user_strokes = vec![vec![]; content.len()];
            self.all_user_strokes_score = vec![0.0; content.len()];
            self.homework = content.chars().collect();
            self.homework_str = content;
            self.stroke_animation = None;
            self.create_strokes();

            self.page
                .update_homework(&self.homework_str, self.character as i32);
        }
        //保存历史记录
        let _ = crate::set_storage(
            KEY_HOMEWORK,
            serialize(&Homework {
                group: 0, //0自定义
                title: "自定义".into(),
                content: self.homework_str.clone(),
                progress: 0,
            })
            .unwrap(),
        );
    }

    //创建笔画数组
    pub fn create_strokes(&mut self) {
        //js!(console.log("create_strokes>>>", new Date(), Date.now()));
        //self.platform.log(&format!("create_strokes character={}", self.character));
        let ch = self.homework[self.character];
        self.drawing_ch = Some(ch);

        let strokes: &Vec<Vec<(u16, u16)>> = self.strokes_map.get(&ch).unwrap();
        //self.platform.log(&format!("一共{}笔", strokes.len()));
        self.strokes.clear();
        for i in 0..strokes.len() {
            //self.platform.log(&format!("第{}笔 {:?}", i, strokes[i]));
            self.strokes.push(resample(
                strokes[i]
                    .iter()
                    .map(|p| Point::new(p.0, p.1, i + 1))
                    .collect(),
                50,
            ));
        }

        //设置显示的字符
        self.page.set_character(ch);
        //js!(console.log("set_character ok.", new Date(), Date.now()));
    }

    fn clear_error_stroke(&mut self, remove_score: bool) {
        //删除当前笔画得分
        if remove_score {
            self.stroke_scores.remove(self.stroke_index);
        }
        //清空当前错误的笔画
        self.user_strokes.remove(self.stroke_index);
        //显示错误图标
        self.error_icon = Some(self.strokes[self.stroke_index][0].clone());
        //笔画提示
        log("笔画错误，切换到动画模式 reset_delay(50).");
        self.stroke_animation = Some(0); //笔画错误动画
        self.reset_delay(DELAY_ANIM, false);
    }

    //开/关 笔画演示控制
    pub fn stroke_anim(&mut self) {
        //如果正在错误图示，不进行操作
        if let Some(anim_type) = self.stroke_animation{
            if anim_type == 0{
                return;
            }

            self.page.stop_brush_blink();
            //结束动画
            self.stroke_anim.clear();
            self.stroke_animation = None;
            //重新渲染
            //log("关闭动画演示，切换为慢速帧率.");
            self.reset_delay(DELAY_IDLE, true);
            self.stroke_index = 0;
        }else{
            self.page.start_brush_blink();
            //清空用户笔画
            self.user_strokes.clear();
            self.stroke_animation = Some(1);
            //切换到第1笔动画
            self.stroke_index = 0;
            self.stroke_anim.clear();
            //开始动画
            self.reset_delay(DELAY_ANIM, false);
        }
    }

    //擦除/重写
    fn rewrite(&mut self) {
        self.stroke_index = 0;
        self.stroke_anim.clear();
        self.stroke_scores.clear();
        self.user_strokes.clear();
        //log("rewrite，切换为慢速帧率.");
        self.reset_delay(DELAY_IDLE, true);
    }

    /**
     * 低于50分不通过。
     * 笔画差距超过16不通过。
     */
    fn calculate_score(&mut self) {
        self.writing = false;
        // self.platform.log("计算当前笔画得分...");

        let mut recognizer = PDollarPlusRecognizer::new();

        //笔画少于2点重写
        if self.user_strokes[self.stroke_index].len() > 2 {
            //将当前笔画加入识别器
            recognizer.add_gesture("stroke", self.strokes[self.stroke_index].clone());
            //识别用户当前笔画
            let result = recognizer.recognize(self.user_strokes[self.stroke_index].clone());
            // self.platform.log_div(&format!(
            //     "笔画得分:{} > {}分",
            //     result.name, result.score
            // ));
            self.stroke_scores.push(result.score);

            //笔画距离超过4.5的不通过
            if self.stroke_scores[self.stroke_index] < 4.5 {
                //切换到下一笔画
                self.stroke_index += 1;
                //如果“笔画错误”处于显示中，将其隐藏
                if self.error_icon.is_some() {
                    self.error_icon = None;
                }

                //检查是否写完
                if self.user_strokes.len() == self.strokes.len() {
                    //检查整个汉字是否正确
                    recognizer.clear_gestures();
                    let mut points = vec![];
                    for stroke in &self.strokes {
                        points.append(&mut stroke.clone());
                    }
                    //self.platform.log(&format!("原始:{:?}", points));
                    recognizer.add_gesture(&self.homework[self.character].to_string(), points);

                    let mut user_points = vec![];
                    for i in 0..self.user_strokes.len() {
                        for point in &self.user_strokes[i] {
                            user_points.push(point.clone());
                        }
                    }
                    //self.platform.log(&format!("用户:{:?}", user_points));
                    let result = recognizer.recognize(user_points);
                    // self.platform
                    //     .log(&format!("整字识别结果:{}", result.score));

                    let mut total_score = 0.0;
                    for score in &self.stroke_scores {
                        total_score += score;
                    }
                    // self.platform
                    //     .log_div(&format!("整字得分:{}/{}", result.score, total_score));
                    self.stroke_scores.clear();
                    //log("笔画完成，整字完成，切换为慢速帧率.");
                    self.reset_delay(DELAY_IDLE, false);


                    //整字得分大于xx分，重写 2.7~6.5 分别代表 10.0~6.2分, 最终整篇计算平均分 8.5~9.0(3星),7.5~8.5(2星),6.0~7.5(1星)
                    if result.score > 6.5 {
                        //提示用户重写
                        crate::alert("哎呦", "字写得太丑了，重新写吧！", &|| {
                            CONTROLLER.with(|c| {
                                c.borrow_mut().rewrite();
                            });
                        });
                        return;
                    } else {
                        //将当前字的笔画数据保存到数组
                        self.all_user_strokes[self.character] = self.user_strokes.clone();

                        //提示用户单字得分
                        //6.5-2.7 = 3.8
                        //得分=result.score-2.7;
                        let mut num = result.score - 2.7;
                        if num < 0.0 {
                            num = 0.0;
                        }
                        let score = 10.0 - num;
                        //保存得分到数组
                        self.all_user_strokes_score[self.character] = score;

                        //切换到下一个字
                        if self.next_character(1) {
                            crate::show_loading("保存记录");
                            //完成作业
                            self.page
                                .update_homework(&self.homework_str, self.character as i32);

                            //计算平均分
                            let sum: f64 = self.all_user_strokes_score.iter().sum();
                            let avg = sum / self.all_user_strokes_score.len() as f64;

                            //格式化时间
                            let timestamp = crate::current_timestamp();
                            //new Date().getTimezoneOffset()/60
                            let timezone_offset = crate::get_timezone_offset();
                            let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
                            //3600为1小时
                            let datetime: DateTime<FixedOffset> = DateTime::from_utc(
                                naive,
                                FixedOffset::west(timezone_offset * 3600),
                            );
                            //let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
                            //保存练习记录(1条)
                            let history = History {
                                avg: format!("{:.1}", avg),
                                scores: self.all_user_strokes_score.clone(),
                                date: format!("{}", datetime.format("%Y-%m-%d %H:%M")),
                                text: self.homework_str.clone(),
                                strokes: self.all_user_strokes.clone(),
                            };

                            //保存历史记录
                            let _ = serialize(&history)
                                .and_then(|data| Ok(crate::save_file(KEY_HISTORY, data)));
                            //重新开始
                            let new_str = self.homework_str.clone();
                            self.set_homework(&new_str);
                            crate::hide_loading();
                            //刷新
                            self.reset_delay(DELAY_IDLE, true);
                            crate::alert("恭喜", "完成任务了！", &|| {});
                        } else {
                            // 不是最后一个字，提示用户得分！！
                            let star = if score >= 8.5 {
                                3
                            } else if score >= 7.5 {
                                2
                            } else {
                                1
                            };

                            //提示用户得星
                            crate::show_custom_toast(
                                format!("{:.1}分", score),
                                String::from(match star {
                                    3 => "/static/star_3.png",
                                    2 => "/static/star_2.png",
                                    _ => "/static/star_1.png",
                                }),
                                1000,
                            );
                        }
                    }
                }else{
                    //log("笔画完成，整字未完成，切换为慢速帧率.");
                    self.reset_delay(DELAY_IDLE, false);
                }
            } else {
                self.clear_error_stroke(true);
            }
        } else {
            log("笔画太短!");
            self.clear_error_stroke(false);
        }
    }

    /**
     * step: 0 不变，1下一个字符，2上一个字符
     */
    pub fn next_character(&mut self, step: i32) -> bool {
        if step == 0 {
            return false;
        }
        if step > 0 && self.character >= self.homework.len() {
            self.complete = true;
            return true;
        }
        if step < 0 && self.character == 0 {
            self.complete = false;
            return true;
        }
        if step > 0 {
            self.character += 1;
        } else {
            self.character -= 1;
        }
        {
            let mut ch = self.homework.get(self.character);
            //跳过标点符号
            while !ch.is_none() && self.strokes_map.get(&ch.unwrap()).is_none() {
                // log(&format!("跳过{}", ch.unwrap()));

                if step > 0 {
                    self.character += 1;
                } else {
                    self.character -= 1;
                }
                if self.character < self.homework.len() {
                    ch = Some(&self.homework[self.character]);
                } else {
                    ch = None;
                }
            }
            if ch.is_none() {
                //|| self.strokes_map.get(&ch.unwrap()).is_none()
                self.complete = true;
                return true;
            }
        }

        //下一个字
        self.stroke_index = 0;
        self.create_strokes();
        self.user_strokes.clear();
        self.error_icon = None;
        //关闭动画
        if self.stroke_animation.is_some() {
            self.stroke_animation = None;
        }

        //清空笔画
        //log("下一个字，切换为慢速帧率.");
        self.reset_delay(DELAY_IDLE, true);
        // log("已切换到下一个字");
        self.page
            .update_homework(&self.homework_str, self.character as i32);
        false
    }

    pub fn on_pointer_down(
        &mut self,
        client_x: f64,
        client_y: f64,
        _offset_x: f64,
        _offset_y: f64,
    ) {
        //log(&format!("on_pointer_down {},{}", client_x, client_y));
        if self.stroke_animation.is_some() {
            // log("on_pointer_down>正在动画，不处理");
            return;
        }
        if self.complete {
            // log("on_pointer_down>已完成，不处理");
            return;
        }
        //创建新的笔画
        self.user_strokes
            .push(vec![Point::new(client_x, client_y, self.stroke_index + 1)]);
        self.writing = true;
    }

    pub fn on_pointer_up(&mut self) {
        if self.writing {
            self.calculate_score();
        }
        if self.stroke_animation.is_none(){
            //log("on_pointer_up，切换为慢速帧率.");
            self.reset_delay(DELAY_IDLE, false);
        }
    }

    pub fn on_pointer_move(
        &mut self,
        client_x: f64,
        client_y: f64,
        _offset_x: f64,
        _offset_y: f64,
    ) {
        //log(&format!("on_pointer_move {},{}", client_x, client_y));
        if let Some((x1, y1)) = self.last_point{
            let (x2,y2) = (client_x, client_y);
            let distance = (((x1-x2)*(x1-x2))+((y1-y2)*(y1-y2))).sqrt();
            if distance<5.0{
                //log("距离小于5.0");
                return;
            }
        }
        if self.writing {
            self.last_point = Some((client_x, client_y));
            //加入画点
            self.user_strokes[self.stroke_index].push(Point::new(
                client_x,
                client_y,
                self.stroke_index + 1,
            ));
            // if crate::current_timestamp() - self.last_render_time >= 35{
                //self.reset_delay(DELAY_IDLE, true);
            // }
            self.reset_delay(DELAY_ANIM, false);
        }else{
            //log(&format!("on_pointer_move 没有writing，不处理 {},{}", client_x, client_y));
        }
    }
}

//检查字典是否已下载
pub fn check_dict() {
    let loaded = CONTROLLER.with(|c| {
        c.borrow().dict.is_some()
    });
    //检查字典是否加载
    if loaded {
        search_dict();
    } else {
        //读取字典
        crate::show_loading("加载字典");
        crate::load_dict(&|data| {
            crate::hide_loading();
            match data {
                Err(err) => crate::alert("提示", &format!("{:?}", err), &|| {}),
                Ok(data) => {
                    //保存
                    CONTROLLER.with(|c| {
                        c.borrow_mut().dict = Some(data);
                    });
                    search_dict();
                }
            }
        });
        // crate::load_dict(&|data| {
        //     crate::hide_loading();
        //     match data {
        //         Err(err) => crate::alert("提示", &format!("{:?}", err), &|| {}),
        //         Ok(data) => {
        //             //保存
        //             let decoded: HashMap<String, Vec<u8>> = deserialize(&data).unwrap();
        //             CONTROLLER.with(|c| {
        //                 c.borrow_mut().dict = Some(decoded);
        //             });
        //             search_dict();
        //         }
        //     }
        // });
    }
}

//读取保存的选择内容
fn get_choose_index() -> Vec<i32>{
    match crate::get_storage(KEY_CHOOSE) {
        Ok(choose) => deserialize(&choose).unwrap_or(vec![0, 0]),
        Err(err) => {
            log(&format!("KEY_CHOOSE读取失败 {:?}", err));
            vec![0, 0]
        }
    }
}

//根据练习内容的分组类型，读取分组的文章标题列表
fn get_choose_names(group_index:i32) -> Vec<String>{
    CONTROLLER.with(|ctrl| {
        let ctrl = ctrl.borrow();
        if group_index == 0 {
            //读取课文
            ctrl.articls_map.iter().map(|v| v.0.clone()).collect()
        } else {
            //读取古诗
            ctrl.poems_map.iter().map(|v| v.0.clone()).collect()
        }
    })
}

//查询字典
fn search_dict() {
    CONTROLLER.with(|ctrl| {
        let ctrl = ctrl.borrow();
        let ch = format!("{}", ctrl.homework[ctrl.character]);
        if let Some(word) = ctrl.dict.as_ref().unwrap().get(&ch) {
            let title = format!("“{}”的字意", ch);
            let content = format!(
                "{}\r\n笔画:{} 部首:{}\r\n{}",
                word.pinyin, word.strokes, word.radicals, word.explanation
            );
            crate::show_modal(&title, &content, false, &|| {}, &|| {});
        } else {
            crate::alert("提示", "没有这个字", &|| {});
        }
    });
}

pub fn init(){

    crate::register_add_feedback(&||{
        crate::show_input("", "笔画错误可在此填写", &|result| {
            if result.trim().len() == 0 {
                crate::show_toast("请输入反馈内容", 1000, false, None);
                return;
            }

            crate::confirm_str("提交确认", "确定要提交反馈内容吗？", &result, &|result:String|{
                crate::show_loading("正在提交");
                crate::add_feedback(result.trim(), &|success|{
                    crate::hide_loading();
                    if success{
                        crate::show_toast("提交成功", 1500, false, None);
                    }else{
                        crate::show_modal("提示", "提交失败", false, &|| {}, &|| {});
                    }
                });
            }, &||{
                
            });
        });
    });

    crate::register_query_dict_listener(&|| {
        check_dict();
    });

    crate::register_previous(&|| {
        CONTROLLER.with(|c| {
            let is_first = c.borrow_mut().next_character(-1);
            if is_first {
                crate::alert("提示", "已经到第一个字了", &|| {});
            }
        });
    });

    crate::register_stroke_anim_listener(&|| {
        CONTROLLER.with(|c| {
            c.borrow_mut().stroke_anim();
        });
    });

    crate::register_on_touchstart(&|client_x: f64,
                                    client_y: f64,
                                    offset_x: f64,
                                    offset_y: f64| {
        CONTROLLER.with(|c| {
            c.borrow_mut().on_pointer_down(client_x, client_y, offset_x, offset_y);
        });
    });

    crate::register_on_touchmove(&|client_x: f64,
                                    client_y: f64,
                                    offset_x: f64,
                                    offset_y: f64| {
        CONTROLLER.with(|c| {
            c.borrow_mut().on_pointer_move(client_x, client_y, offset_x, offset_y);
        });
    });

    crate::register_history_listener(&|| {
        //读取历史记录
        crate::show_loading("正在加载");
        crate::read_file(KEY_HISTORY, &|res| {
            crate::hide_loading();
            if let Ok(data) = res {
                match deserialize(&data) {
                    Ok(history) => {
                        crate::show_history(history, 0);
                    }
                    Err(err) => {
                        log(&format!("历史记录读取失败 {:?}", err));
                        crate::show_toast("读取失败", 1000, false, None);
                    }
                }
            } else {
                crate::alert("提示", "暂无练习记录", &|| {});
                //crate::show_toast("暂无记录", 1000, false, None);
            }
        });
    });

    crate::register_show_input(&|| {
        let input: String = match &crate::get_storage(KEY_INPUTS) {
            Ok(saved) => deserialize(&saved).unwrap(),
            Err(err) => {
                log(&format!(
                    ">>>>>>>>register_show_input get_storage {:?}",
                    err
                ));
                String::new()
            }
        };
        crate::show_input(&input, "输入练习内容", &|result| {
            crate::hide_input();
            if result.trim().len() == 0 {
                return;
            }
            //保存输入历史和作业内容
            if let Err(err) = crate::set_storage(KEY_INPUTS, serialize(&result).unwrap()) {
                log(&format!("inputs保存失败!{:?}", err));
            }
            CONTROLLER.with(|c| {
                c.borrow_mut().set_homework(&result);
            });
        });
    });

    crate::register_search_title(&|text|{
        //读取保存得选项记录
        let mut choose = get_choose_index();
        let names: Vec<String> = get_choose_names(choose[0]);
        for j in 0..names.len(){
            if names[j].contains(&text){
                choose[1] = j as i32;
                break;
            }
        }
        crate::show_choose(None, choose[0], None, choose[1], None);
    });

    crate::register_show_choose(&|| {
        //读取保存得选项记录
        let choose = get_choose_index();

        let groups = vec!["课文".into(), "古诗".into()];

        let names: Vec<String> = get_choose_names(choose[0]);

        crate::show_choose(
            Some(groups),
            choose[0],
            Some(names),
            choose[1],
            //选择完成
            Some((
                &|group_id, name_id| {
                    CONTROLLER.with(|c| {
                        let mut ctrl = c.borrow_mut();
                        //设置选择的练习内容
                        let homework = if group_id == 0 {
                            ctrl.articls
                                .get(&ctrl.articls_map[name_id as usize].1)
                                .unwrap()
                                .clone()
                        } else {
                            ctrl.poems
                                .get(&ctrl.poems_map[name_id as usize].1)
                                .unwrap()
                                .clone()
                        };
                        ctrl.set_homework(&homework);
                    });

                    //保存选择内容
                    let _ = serialize(&vec![group_id, name_id])
                        .and_then(|data| Ok(crate::set_storage(KEY_CHOOSE, data)));
                },
                &|old_group_id, _old_name_id, new_group_id, new_name_id| {
                    let groups: Vec<String> = vec!["课文".into(), "古诗".into()];

                    let names = if old_group_id == new_group_id {
                        None
                    } else {
                        Some(get_choose_names(new_group_id))
                    };
                    crate::show_choose(Some(groups), new_group_id, names, new_name_id, None);
                    //保存选择内容
                    let _ = serialize(&vec![new_group_id, new_name_id])
                        .and_then(|data| Ok(crate::set_storage(KEY_CHOOSE, data)));
                },
            )),
        );
    });

    crate::register_on_touchend(&|| {
        CONTROLLER.with(|c| {
            c.borrow_mut().on_pointer_up();
        });
    });

    crate::register_on_touchcancel(&|| {
        CONTROLLER.with(|c| {
            c.borrow_mut().on_pointer_up();
        });
    });

    crate::register_on_ready_listener(&|| {
        CONTROLLER.with(|c| {
            c.borrow_mut().on_ready();
        });
    });

    crate::register_eraser_listener(&|| {
        CONTROLLER.with(|c| {
            let mut ctrl = c.borrow_mut();
            if ctrl.stroke_animation.is_some() || ctrl.user_strokes.len() == 0 {
                //提示橡皮擦出文字
                crate::show_toast("橡皮擦", 1000, false, None);
            } else {
                ctrl.rewrite();
            }
        });
    });
}