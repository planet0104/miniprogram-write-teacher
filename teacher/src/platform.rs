//系统功能接口
//页面、UI的API接口
use crate::pdollarplus::Point;

pub trait CanvasContext {
    fn set_fill_style_color(&self, color: &str);
    fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64);
    fn set_stroke_style_color(&self, color: &str);
    fn set_line_width(&self, line_width: f64);
    fn begin_path(&self);
    fn move_to(&self, x: f64, y: f64);
    fn line_to(&self, x: f64, y: f64);
    fn stroke(&self);
    fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64);
    fn fill_circle(&self, x: f64, y: f64, radius: f64);
    fn draw_image(&self, path: &str, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_at(&self, path: &str, x: f64, y: f64);
    fn set_line_dash(&self, segments: Vec<f64>);
    fn save(&self);
    fn restore(&self);
    fn scale(&self, x: f64, y: f64);
    fn translate(&self, x: f64, y: f64);
    fn rotate(&self, angle: f64);
    fn draw(&self, callback: &'static Fn()) {
        callback();
    }
    fn set_font_size(&self, font_size:f64);
    fn fill_text(&self, text:&str, x:f64, y:f64);
}

pub trait MainPage {
    //获取画布
    fn canvas(&self) -> &Box<CanvasContext>;
    fn canvas_width(&self) -> f64;
    fn canvas_height(&self) -> f64;
    //设置显示的字符(页面标签) [//在小程序中，程序分包根据需要切换不同字体]
    fn set_character(&self, c: char);
    //作业完成进度预览
    fn update_homework(&self, content: &str, index: i32);

    //笔刷按钮闪烁动画
    fn start_brush_blink(&self);
    fn stop_brush_blink(&self);
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct History {
    pub date: String,
    pub text: String,
    pub strokes: Vec<Vec<Vec<Point>>>, //存储每个字得笔画
    pub scores: Vec<f64>,              //存储每个字得得分
    pub avg: String,                   //平均分
}
